use serial_test::serial;
use sword::grpc::*;
use sword::prelude::*;
use sword_grpc::prelude::{GrpcResponse, GrpcResult};
use tokio::time::{Duration, sleep};
use tonic::{Request, Status};

pub mod proto {
    tonic::include_proto!("secure");
}

use proto::secure_service_server::{SecureService, SecureServiceServer};
use proto::{PingReply, PingRequest};

const AUTH_METADATA_KEY: &str = "authorization";
const TOKEN: &str = "Bearer test-token";

#[derive(Interceptor)]
struct RequireAuthHeaderInterceptor;

impl OnRequest for RequireAuthHeaderInterceptor {
    async fn on_request(&self, req: Request<()>) -> GrpcInterceptorResult {
        let metadata = req.metadata();

        if metadata.get(AUTH_METADATA_KEY).is_some() {
            Ok(req)
        } else {
            Err(Status::unauthenticated(
                "missing authorization metadata header",
            ))
        }
    }
}

#[derive(Interceptor)]
struct AuthWithConfigInterceptor;

impl OnRequestWithConfig<&'static str> for AuthWithConfigInterceptor {
    async fn on_request(&self, expected: &'static str, req: Request<()>) -> GrpcInterceptorResult {
        let metadata = req.metadata();
        let token = metadata
            .get(AUTH_METADATA_KEY)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if token == expected {
            Ok(req)
        } else {
            Err(Status::permission_denied(
                "authorization token does not match config",
            ))
        }
    }
}

#[controller(kind = Controller::Grpc, service = SecureServiceServer)]
#[interceptor(RequireAuthHeaderInterceptor)]
#[interceptor(AuthWithConfigInterceptor, config = "Bearer test-token")]
struct SecureController;

#[sword::grpc::async_trait]
impl SecureService for SecureController {
    async fn ping(&self, req: Request<PingRequest>) -> GrpcResult<PingReply> {
        let msg = req.into_inner().message;
        Ok(GrpcResponse::message(PingReply {
            message: format!("pong: {msg}"),
        }))
    }
}

struct SecureGrpcModule;

impl Module for SecureGrpcModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<SecureController>();
    }
}

async fn start_server_for_test() -> tokio::task::JoinHandle<()> {
    let config_path = format!("{}/config/config.toml", env!("CARGO_MANIFEST_DIR"));
    let app = Application::from_config_path(&config_path)
        .with_module::<SecureGrpcModule>()
        .build();

    let handle = tokio::spawn(async move {
        app.run().await;
    });

    sleep(Duration::from_millis(150)).await;

    for _ in 0..50 {
        if handle.is_finished() {
            match handle.await {
                Ok(()) => panic!("gRPC server task exited before accepting connections"),
                Err(err) => panic!("gRPC server task failed: {err}"),
            }
        }

        if tonic::transport::Channel::from_static("http://127.0.0.1:50051")
            .connect()
            .await
            .is_ok()
        {
            return handle;
        }

        sleep(Duration::from_millis(30)).await;
    }

    panic!("gRPC server did not become ready on 127.0.0.1:50051");
}

#[tokio::test]
#[serial]
async fn interceptor_rejects_without_metadata() {
    let server = start_server_for_test().await;

    let mut client =
        proto::secure_service_client::SecureServiceClient::connect("http://127.0.0.1:50051")
            .await
            .expect("client must connect");

    let result = client
        .ping(Request::new(PingRequest {
            message: "hello".to_string(),
        }))
        .await;

    let status = result.expect_err("request should be rejected");
    assert_eq!(status.code(), tonic::Code::Unauthenticated);

    server.abort();
}

#[tokio::test]
#[serial]
async fn interceptor_allows_with_metadata_and_config() {
    let server = start_server_for_test().await;

    let mut client =
        proto::secure_service_client::SecureServiceClient::connect("http://127.0.0.1:50051")
            .await
            .expect("client must connect");

    let mut request = Request::new(PingRequest {
        message: "hello".to_string(),
    });

    let metadata_value = tonic::metadata::MetadataValue::try_from(TOKEN)
        .expect("token must be valid metadata value");
    request
        .metadata_mut()
        .insert(AUTH_METADATA_KEY, metadata_value);

    let response = client
        .ping(request)
        .await
        .expect("request should succeed")
        .into_inner();

    assert_eq!(response.message, "pong: hello");

    server.abort();
}

#[tokio::test]
#[serial]
async fn interceptor_with_config_rejects_mismatched_token() {
    let server = start_server_for_test().await;

    let mut client =
        proto::secure_service_client::SecureServiceClient::connect("http://127.0.0.1:50051")
            .await
            .expect("client must connect");

    let mut request = Request::new(PingRequest {
        message: "hello".to_string(),
    });

    let metadata_value = tonic::metadata::MetadataValue::try_from("Bearer wrong-token")
        .expect("token must be valid metadata value");
    request
        .metadata_mut()
        .insert(AUTH_METADATA_KEY, metadata_value);

    let result = client.ping(request).await;

    let status = result.expect_err("request should fail on config mismatch");
    assert_eq!(status.code(), tonic::Code::PermissionDenied);
    assert_eq!(
        status.message(),
        "authorization token does not match config"
    );

    server.abort();
}
