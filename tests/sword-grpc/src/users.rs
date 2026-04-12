use serial_test::serial;
use sword::grpc::*;
use sword::prelude::*;
use sword_grpc::prelude::{GrpcResponse, GrpcResult, GrpcStream};
use tokio::time::{Duration, sleep};

pub mod proto {
    tonic::include_proto!("users");
}

use proto::user_service_server::{UserService, UserServiceServer};
use proto::{
    GetUserRequest, ListUsersReply, ListUsersRequest, StreamUsersRequest, UserItem, UserReply,
};

#[derive(Clone)]
struct InMemoryUsersService;

impl InMemoryUsersService {
    fn users() -> Vec<UserItem> {
        vec![
            UserItem {
                id: "u1".to_string(),
                username: "alice".to_string(),
            },
            UserItem {
                id: "u2".to_string(),
                username: "bob".to_string(),
            },
        ]
    }
}

#[controller(kind = Controller::Grpc, service = UserServiceServer)]
struct UsersController;

#[sword::grpc::async_trait]
impl UserService for UsersController {
    type StreamUsersStream = GrpcStream<UserItem>;

    async fn list_users(&self, _: Request<ListUsersRequest>) -> GrpcResult<ListUsersReply> {
        Ok(GrpcResponse::message(ListUsersReply {
            users: InMemoryUsersService::users(),
        }))
    }

    async fn stream_users(
        &self,
        _: Request<StreamUsersRequest>,
    ) -> GrpcResult<Self::StreamUsersStream> {
        let users = InMemoryUsersService::users();
        let stream = tokio_stream::iter(users.into_iter().map(Ok));
        Ok(GrpcResponse::stream(stream))
    }

    async fn get_user(&self, req: Request<GetUserRequest>) -> GrpcResult<UserReply> {
        let id = req.into_inner().id;

        let maybe_user = InMemoryUsersService::users()
            .into_iter()
            .find(|u| u.id == id);

        let user = maybe_user.ok_or_else(|| Status::not_found("user not found"))?;

        Ok(GrpcResponse::message(UserReply { user: Some(user) }))
    }

    async fn get_error(
        &self,
        _: Request<proto::GetErrorRequest>,
    ) -> GrpcResult<proto::GetErrorReply> {
        Ok(GrpcResponse::message(proto::GetErrorReply {
            id: String::new(),
            status: "ok".into(),
        }))
    }
}

struct GrpcModule;

impl Module for GrpcModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<UsersController>();
    }
}

async fn start_server_for_test() -> tokio::task::JoinHandle<()> {
    let config_path = format!("{}/config/config.toml", env!("CARGO_MANIFEST_DIR"));
    let app = Application::from_config_path(&config_path)
        .with_module::<GrpcModule>()
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
async fn list_users_unary_ok() {
    let server = start_server_for_test().await;

    let mut client =
        proto::user_service_client::UserServiceClient::connect("http://127.0.0.1:50051")
            .await
            .expect("client must connect");

    let response = client
        .list_users(Request::new(ListUsersRequest {}))
        .await
        .expect("list_users should succeed")
        .into_inner();

    assert_eq!(response.users.len(), 2);
    assert_eq!(response.users[0].username, "alice");
    assert_eq!(response.users[1].username, "bob");

    server.abort();
}

#[tokio::test]
#[serial]
async fn get_user_not_found_returns_not_found() {
    let server = start_server_for_test().await;

    let mut client =
        proto::user_service_client::UserServiceClient::connect("http://127.0.0.1:50051")
            .await
            .expect("client must connect");

    let result = client
        .get_user(Request::new(GetUserRequest {
            id: "does-not-exist".to_string(),
        }))
        .await;

    let status = result.expect_err("request should fail with not found");
    assert_eq!(status.code(), tonic::Code::NotFound);
    assert_eq!(status.message(), "user not found");

    server.abort();
}

#[tokio::test]
#[serial]
async fn stream_users_receives_items() {
    let server = start_server_for_test().await;

    let mut client =
        proto::user_service_client::UserServiceClient::connect("http://127.0.0.1:50051")
            .await
            .expect("client must connect");

    let mut stream = client
        .stream_users(Request::new(StreamUsersRequest {}))
        .await
        .expect("stream_users should succeed")
        .into_inner();

    let first = stream
        .message()
        .await
        .expect("stream read should succeed")
        .expect("first item expected");
    let second = stream
        .message()
        .await
        .expect("stream read should succeed")
        .expect("second item expected");

    assert_eq!(first.username, "alice");
    assert_eq!(second.username, "bob");

    server.abort();
}
