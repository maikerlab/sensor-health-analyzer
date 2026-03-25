use ort::inputs;
use ort::session::{Session, SessionOutputs, builder::GraphOptimizationLevel};
use ort::value::TensorRef;
use std::path::Path;
use std::sync::Mutex;
use ndarray::Array2;
use tracing::{debug, info};

/// Input features – order must match Python training exactly
#[derive(Debug)]
pub struct Features {
    pub value: f32,
    pub mean: f32,
    pub std_dev: f32,
    pub min: f32,
    pub max: f32,
    pub slope: f32,
    pub z_score: f32,
}

impl Features {
    fn to_vec(&self) -> Vec<f32> {
        vec![
            self.value,
            self.mean,
            self.std_dev,
            self.min,
            self.max,
            self.slope,
            self.z_score,
        ]
    }
}

/// Output of the model
#[derive(Debug)]
pub struct Prediction {
    pub health_score: u8, // 0–100, derived from anomaly score
    pub is_anomaly: bool, // true if IsolationForest says -1
    pub raw_score: f32,   // raw decision function score
}

pub struct SensorHealthModel {
    session: Mutex<Session>,
}

impl SensorHealthModel {
    pub fn load(model_path: &Path) -> anyhow::Result<Self> {
        let session = Session::builder()?
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

    pub fn predict(&self, features: &Features) -> anyhow::Result<Prediction> {
        // Shape: [1, 7] – one sample, seven features
        let input: Array2<f32> = Array2::from_shape_vec((1, 7), features.to_vec())?;

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
        debug!(
            outputs = %format!("{:?}", outputs_debug),
            "predict outputs"
        );

        // IsolationForest outputs:
        // "label"  → i64 tensor, shape [1],    values: 1 (normal) or -1 (anomaly)
        // "scores" → f32 tensor, shape [1, 2], anomaly score per class
        let labels = outputs["label"].try_extract_array::<i64>()?;
        let scores = outputs["scores"].try_extract_array::<f32>()?;
        debug!(
            label_shape = %format!("{:?}", labels.shape()),
            scores_shape = %format!("{:?}", scores.shape()),
            "IsolationForest outputs"
        );

        let label = labels[[0, 0]]; // 1 or -1
        // TODO: should be scores[[0, 0]], but model only has [1,1] shape - why??
        let raw_score = scores[[0, 0]]; // normal class score
        debug!(
            raw_label = %label.to_string(),
            raw_score = %raw_score.to_string(),
            "raw"
        );

        let prediction = Prediction {
            is_anomaly: label == -1,
            health_score: self.score_to_health(raw_score),
            raw_score,
        };
        info!(
            health_score = %prediction.health_score,
            is_anomaly = %prediction.is_anomaly,
            raw_score = %prediction.raw_score,
            "Prediction result ready"
        );
        Ok(prediction)
    }

    fn score_to_health(&self, score: f32) -> u8 {
        // IsolationForest decision_function:
        // positive → normal (further from boundary)
        // negative → anomaly
        // typical range: -0.12 (anomaly) to +0.17 (normal)
        let clamped = score.clamp(-0.12, 0.17);
        let normalized = (clamped + 0.12) / (0.17 + 0.12); // 0.0–1.0
        (normalized * 100.0).round() as u8
    }
}
