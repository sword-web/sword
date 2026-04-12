use serial_test::serial;
use sword::grpc::*;
use sword::prelude::*;
use thiserror::Error;
use tokio::time::{Duration, sleep};
use tonic::{Code, Request, transport::Channel};

pub mod proto {
    tonic::include_proto!("users");
}

#[derive(Debug, Error, GrpcError)]
#[grpc_error(code = "internal", tracing = error)]
enum TestGrpcError {
    #[error("Not found")]
    #[grpc(code = "not_found", tracing = info)]
    NotFound,

    #[error("Validation failure: {detail}")]
    #[grpc(code = "invalid_argument", message = client_message)]
    Validation {
        client_message: String,
        detail: String,
    },

    #[error("Unavailable service")]
    Unavailable,
}

#[controller(kind = Controller::Grpc, service = proto::user_service_server::UserServiceServer)]
struct ErrorController;

#[sword::grpc::async_trait]
impl proto::user_service_server::UserService for ErrorController {
    type StreamUsersStream = GrpcStream<proto::UserItem>;

    async fn list_users(
        &self,
        _: Request<proto::ListUsersRequest>,
    ) -> GrpcResult<proto::ListUsersReply> {
        Ok(GrpcResponse::message(proto::ListUsersReply {
            users: vec![],
        }))
    }

    async fn stream_users(
        &self,
        _: Request<proto::StreamUsersRequest>,
    ) -> GrpcResult<Self::StreamUsersStream> {
        let stream = tokio_stream::iter(Vec::<proto::UserItem>::new().into_iter().map(Ok));
        Ok(GrpcResponse::stream(stream))
    }

    async fn get_user(&self, _: Request<proto::GetUserRequest>) -> GrpcResult<proto::UserReply> {
        Ok(GrpcResponse::message(proto::UserReply { user: None }))
    }

    async fn get_error(
        &self,
        request: Request<proto::GetErrorRequest>,
    ) -> GrpcResult<proto::GetErrorReply> {
        let id = request.into_inner().id;

        match id.as_str() {
            "not-found" => Err(TestGrpcError::NotFound.into()),
            "validation" => Err(TestGrpcError::Validation {
                client_message: "Field validation failed".into(),
                detail: "missing name".into(),
            }
            .into()),
            "unavailable" => Err(TestGrpcError::Unavailable.into()),
            _ => Ok(GrpcResponse::message(proto::GetErrorReply {
                id,
                status: "ok".into(),
            })),
        }
    }
}

struct ErrorModule;

impl Module for ErrorModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<ErrorController>();
    }
}

async fn start_error_server() -> tokio::task::JoinHandle<()> {
    let config_path = format!("{}/config/config.toml", env!("CARGO_MANIFEST_DIR"));
    let app = Application::from_config_path(&config_path)
        .with_module::<ErrorModule>()
        .build();

    let handle = tokio::spawn(async move { app.run().await });

    sleep(Duration::from_millis(150)).await;

    for _ in 0..50 {
        if handle.is_finished() {
            match handle.await {
                Ok(()) => panic!("gRPC server task exited before accepting connections"),
                Err(err) => panic!("gRPC server task failed: {err}"),
            }
        }

        if Channel::from_static("http://127.0.0.1:50051")
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
async fn grpc_error_route_not_found_returns_status() {
    let server = start_error_server().await;
    let mut client =
        proto::user_service_client::UserServiceClient::connect("http://127.0.0.1:50051")
            .await
            .expect("client must connect");

    let result = client
        .get_error(Request::new(proto::GetErrorRequest {
            id: "not-found".into(),
        }))
        .await;

    let status = result.expect_err("request should fail with not found");
    assert_eq!(status.code(), Code::NotFound);
    assert_eq!(status.message(), "Not found");

    server.abort();
}

#[tokio::test]
#[serial]
async fn grpc_error_route_validation_returns_custom_message() {
    let server = start_error_server().await;
    let mut client =
        proto::user_service_client::UserServiceClient::connect("http://127.0.0.1:50051")
            .await
            .expect("client must connect");

    let result = client
        .get_error(Request::new(proto::GetErrorRequest {
            id: "validation".into(),
        }))
        .await;

    let status = result.expect_err("request should fail with invalid_argument");
    assert_eq!(status.code(), Code::InvalidArgument);
    assert_eq!(status.message(), "Field validation failed");

    server.abort();
}

#[tokio::test]
#[serial]
async fn grpc_error_route_enum_default_code_becomes_internal() {
    let server = start_error_server().await;
    let mut client =
        proto::user_service_client::UserServiceClient::connect("http://127.0.0.1:50051")
            .await
            .expect("client must connect");

    let result = client
        .get_error(Request::new(proto::GetErrorRequest {
            id: "unavailable".into(),
        }))
        .await;

    let status = result.expect_err("request should fail with internal");
    assert_eq!(status.code(), Code::Internal);
    assert_eq!(status.message(), "Unavailable service");

    server.abort();
}
