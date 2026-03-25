use anyhow::Context;
use ndarray::Array2;
use ort::ep::CoreMLExecutionProvider;
use ort::inputs;
use ort::session::{Session, SessionOutputs, builder::GraphOptimizationLevel};
use ort::value::TensorRef;
use sensorvault_core::models::SensorData;
use std::path::Path;
use std::sync::Mutex;
use tracing::{debug, warn};

pub struct GrillFeatures {
    pub temperature: f32,
    pub target_temp: f32,
    pub temp_remaining: f32,
    pub time_elapsed: f32,
    pub avg_rate: f32,
    pub recent_rate: f32,
    pub rate_trend: f32,
    pub progress_ratio: f32,
}

impl GrillFeatures {
    pub fn to_vec(&self) -> Vec<f32> {
        vec![
            self.temperature,
            self.target_temp,
            self.temp_remaining,
            self.time_elapsed,
            self.avg_rate,
            self.recent_rate,
            self.rate_trend,
            self.progress_ratio,
        ]
    }
}

pub fn compute_grill_features(
    history: &[SensorData],
    current_temp: f32,
    target_temp: f32,
) -> Option<GrillFeatures> {
    if history.len() < 3 {
        return None;
    }

    let first_time = history.first()?.time;
    let last_time = history.last()?.time;

    let time_elapsed = (last_time - first_time).num_seconds() as f32 / 60.0;
    if time_elapsed < 1.0 {
        warn!(
            time_elapsed = %time_elapsed,
            "cannot compute grill features because time_elapsed too small"
        );
        return None;
    }

    let first_temp = history.first()?.value as f32;
    let temp_rise = current_temp - first_temp;
    let avg_rate = temp_rise / time_elapsed;

    // Recent rate from last 5 readings
    let recent_window = &history[history.len().saturating_sub(5)..];
    let recent_rate = compute_rate(recent_window);

    // Rate trend from last 10 readings
    let trend_window = &history[history.len().saturating_sub(10)..];
    let rate_trend = compute_rate_trend(trend_window);

    let temp_remaining = target_temp - current_temp;
    let progress_ratio = 1.0 - (temp_remaining / (target_temp - 20.0).max(1.0));

    debug!(
        temp_rise = %temp_rise,
        avg_rate = %avg_rate,
        recent_rate = %recent_rate,
        rate_trend = %rate_trend,
        temp_remaining = %temp_remaining,
        progress_ratio = %progress_ratio,
      "computed grill features"
    );

    Some(GrillFeatures {
        temperature: current_temp,
        target_temp,
        temp_remaining,
        time_elapsed,
        avg_rate,
        recent_rate,
        rate_trend,
        progress_ratio,
    })
}

fn compute_rate(readings: &[SensorData]) -> f32 {
    if readings.len() < 2 {
        return 0.0;
    }
    let first = readings.first().unwrap();
    let last = readings.last().unwrap();
    let dt = (last.time - first.time).num_seconds() as f32 / 60.0;
    if dt < 0.01 {
        return 0.0;
    }
    (last.value as f32 - first.value as f32) / dt
}

fn compute_rate_trend(readings: &[SensorData]) -> f32 {
    if readings.len() < 4 {
        return 0.0;
    }
    // Compute instantaneous rates then fit slope
    let rates: Vec<f32> = readings
        .windows(2)
        .map(|w| {
            let dt = (w[1].time - w[0].time).num_seconds() as f32 / 60.0;
            if dt < 0.01 {
                0.0
            } else {
                (w[1].value as f32 - w[0].value as f32) / dt
            }
        })
        .collect();

    // Slope of rates over time
    let n = rates.len() as f32;
    let x_mean = (n - 1.0) / 2.0;
    let y_mean = rates.iter().sum::<f32>() / n;
    let num: f32 = rates
        .iter()
        .enumerate()
        .map(|(i, r)| (i as f32 - x_mean) * (r - y_mean))
        .sum();
    let den: f32 = rates
        .iter()
        .enumerate()
        .map(|(i, _)| (i as f32 - x_mean).powi(2))
        .sum();

    if den.abs() < 1e-8 { 0.0 } else { num / den }
}

pub struct GrillPredictorModel {
    session: Mutex<Session>,
}

impl GrillPredictorModel {
    pub fn load(model_path: &Path) -> anyhow::Result<Self> {
        let coreml = CoreMLExecutionProvider::default().build();
        let session = Session::builder()?
            .with_execution_providers([coreml])
            .map_err(|e| anyhow::anyhow!("execution provider: {e}"))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| anyhow::anyhow!("opt level: {e}"))?
            .with_intra_threads(4)
            .map_err(|e| anyhow::anyhow!("threads: {e}"))?
            .commit_from_file(model_path)
            .map_err(|e| anyhow::anyhow!("load: {e}"))?;

        Ok(Self {
            session: Mutex::new(session),
        })
    }

    pub fn predict(&self, features: &GrillFeatures) -> anyhow::Result<f32> {
        let input = Array2::from_shape_vec(
            (1, 8), // 1 sample, 8 features
            features.to_vec(),
        )
        .context("Failed to create input array")?;

        // Get session
        let mut session = self
            .session
            .lock()
            .map_err(|_| anyhow::anyhow!("poisened lock to session"))?;

        // Run inference
        let outputs: SessionOutputs =
            session.run(inputs!["float_input" => TensorRef::from_array_view(&input)?])?;
        // temporarily added to see output names and shapes
        let outputs_debug = outputs
            .iter()
            .map(|(output, _)| output.to_string())
            .collect::<Vec<String>>();

        let predictions = outputs["variable"]
            .try_extract_array::<f32>()
            .context("Failed to extract prediction")?;
        debug!(
            outputs = %format!("{:?}", outputs_debug),
            shape = %format!("{:?}", predictions.shape()),
            "outputs and shape"
        );
        let minutes_remaining = predictions[[0, 0]].max(0.0); // clamp to non-negative

        Ok(minutes_remaining)
    }
}
