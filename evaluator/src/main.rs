use crate::model::{Features, SensorHealthModel};
use evaluation::evaluation_service_server::{EvaluationService, EvaluationServiceServer};
use evaluation::{EvaluationRequest, EvaluationReply};
use std::path::Path;
use std::sync::Arc;
use tonic::transport::Server;
use tonic::{Code, Request, Response, Status};

mod model;
pub mod evaluation {
    tonic::include_proto!("evaluation");
}

pub struct SensorEvaluationService {
    model: Arc<SensorHealthModel>,
}

impl SensorEvaluationService {
    pub fn new(model: SensorHealthModel) -> Self {
        Self {
            model: Arc::new(model),
        }
    }
}

#[tonic::async_trait]
impl EvaluationService for SensorEvaluationService {
    async fn eval_temperature(
        &self,
        request: Request<EvaluationRequest>,
    ) -> Result<Response<EvaluationReply>, Status> {
        let request: EvaluationRequest = request.into_inner();
        println!("Received {:?}", request);
        let normal = Features {
            value: request.temperature as f32,
            mean: 21.1,
            std_dev: 1.2,
            min: 19.5,
            max: 23.0,
            slope: 0.01,
            z_score: 0.17,
        };
        let result = self
            .model
            .predict(&normal)
            .map_err(|e| Status::new(Code::Internal, format!("{:?}", e)))?;
        let reply = EvaluationReply {
            health_score: result.health_score as u32,
            is_anomaly: result.is_anomaly,
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let onnx_path = Path::new("evaluator/onnx/sensor_health.onnx");
    let model = SensorHealthModel::load(onnx_path).expect("failed to load sensor health model");

    let addr = "[::1]:50051".parse()?;
    let grpc_service = SensorEvaluationService::new(model);

    println!("gRPC server listening on {addr}");

    Server::builder()
        .add_service(EvaluationServiceServer::new(grpc_service))
        .serve(addr)
        .await?;

    Ok(())
}
