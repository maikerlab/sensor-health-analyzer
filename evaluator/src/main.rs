use crate::evaluation::{GrillEvalRequest, GrillEvalResponse};
use crate::models::grill::GrillPredictorModel;
use clap::Parser;
use evaluation::evaluation_service_server::{EvaluationService, EvaluationServiceServer};
use infra::persistence::SensorDataRepository;
use infra::persistence::postgres::PostgresDatabase;
use std::path::Path;
use std::sync::Arc;
use tonic::transport::Server;
use tonic::{Code, Request, Response, Status};
use tracing::info;

pub mod models;

pub mod evaluation {
    tonic::include_proto!("evaluation");
}

pub struct SensorEvaluationService<D: SensorDataRepository> {
    model: Arc<GrillPredictorModel>,
    db: Arc<D>,
}

impl<R> SensorEvaluationService<R>
where
    R: SensorDataRepository + Send + Sync,
{
    pub fn new(model: GrillPredictorModel, db: R) -> Self {
        Self {
            model: Arc::new(model),
            db: Arc::new(db),
        }
    }
}

#[tonic::async_trait]
impl<R> EvaluationService for SensorEvaluationService<R>
where
    R: SensorDataRepository + Send + Sync + 'static,
{
    async fn eval_grill_session(
        &self,
        request: Request<GrillEvalRequest>,
    ) -> Result<Response<GrillEvalResponse>, Status> {
        let request: GrillEvalRequest = request.into_inner();
        info!("Received {:?}", request);

        let readings = self
            .db
            .find_readings_by_sensor_id_since_minutes(&request.sensor_id, request.minutes_elapsed)
            .await
            .map_err(|e| Status::new(Code::Internal, format!("{:?}", e)))?;
        info!("{:?}", readings);
        if readings.len() < 3 {
            return Ok(Response::new(GrillEvalResponse {
                minutes_remaining: -1.0,
                confidence: 0.0,
                message: "Not enough data yet – need at least 3 readings".into(),
            }));
        }

        // ── 2. Compute features ───────────────────────────────────────────
        let features = match models::grill::compute_grill_features(
            &readings,
            request.current_temp,
            request.target_temp,
        ) {
            Some(f) => f,
            None => {
                return Ok(Response::new(GrillEvalResponse {
                    minutes_remaining: 0.0,
                    confidence: 0.0,
                    message: "Could not compute features from history".into(),
                }));
            }
        };

        // ── 3. Run inference ──────────────────────────────────────────────
        let minutes_remaining = self
            .model
            .predict(&features)
            .map_err(|e| Status::internal(e.to_string()))?;

        // ── 4. Confidence based on data quality ──────────────────────────
        let confidence = derive_confidence(readings.len(), features.rate_trend);

        // ── 5. Human-readable message ─────────────────────────────────────
        let message = format_message(minutes_remaining, request.target_temp, confidence);

        Ok(Response::new(GrillEvalResponse {
            minutes_remaining,
            confidence,
            message,
        }))
    }
}

fn format_message(minutes: f32, target: f32, confidence: f32) -> String {
    if confidence < 0.5 {
        return format!(
            "Estimated {:.0} min remaining to reach {:.0}°C (low confidence – still early)",
            minutes, target
        );
    }
    match minutes {
        m if m < 2.0 => format!(
            "Almost done! {:.0}°C target reached in ~{:.0} min",
            target, m
        ),
        m if m < 10.0 => format!("{:.0} min remaining to reach {:.0}°C", m, target),
        m => format!("About {:.0} min remaining to reach {:.0}°C", m, target),
    }
}

fn derive_confidence(n_readings: usize, rate_trend: f32) -> f32 {
    // More readings = more confident
    let data_confidence = match n_readings {
        0..=3 => 0.2,
        4..=10 => 0.5,
        11..=20 => 0.7,
        _ => 0.9,
    };

    // Unstable rate trend = less confident
    let stability = if rate_trend.abs() > 0.5 { 0.7 } else { 1.0 };

    data_confidence * stability
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// ML model to load for running predictions
    model_name: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // init logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let model_path = format!("evaluator/onnx/{}.onnx", cli.model_name.clone());
    let onnx_path = Path::new(model_path.as_str());
    let model = GrillPredictorModel::load(onnx_path).expect("failed to load sensor health model");

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let db = PostgresDatabase::connect(db_url, 3).await?;
    info!("Connected to database");

    let grpc_service = SensorEvaluationService::new(model, db);

    let addr = "[::1]:50051".parse()?;
    info!("gRPC server listening on {addr}");

    Server::builder()
        .add_service(EvaluationServiceServer::new(grpc_service))
        .serve(addr)
        .await?;

    Ok(())
}
