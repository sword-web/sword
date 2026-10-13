use std::sync::Arc;

use serial_test::serial;
use sword::grpc::*;
use sword::prelude::*;
use sword_core::{DependencyContainer, State};

use crate::users::{proto, start_server_for_test};

#[injectable]
pub struct UsersClientService {
    client: Arc<proto::user_service_client::UserServiceClient<Channel>>,
}

impl UsersClientService {
    pub async fn list(&self) -> Result<Vec<proto::UserItem>, Status> {
        let mut client = self.client.as_ref().clone();
        Ok(client
            .list_users(Request::new(proto::ListUsersRequest {}))
            .await?
            .into_inner()
            .users)
    }
}

struct ClientModule;

impl Module for ClientModule {
    async fn register_providers(config: &Config, providers: &ProviderRegistry) {
        let grpc = config.get_or_default::<GrpcApplicationConfig>();
        let channel = Channel::from_config(grpc.expect_client("users"))
            .await
            .expect("channel must be built from config");

        providers.register(proto::user_service_client::UserServiceClient::new(channel));
    }

    fn register_components(components: &ComponentRegistry) {
        components.register::<UsersClientService>();
    }
}

fn load_config() -> Config {
    let path = format!("{}/config/config.toml", env!("CARGO_MANIFEST_DIR"));

    Config::builder()
        .add_required_file(&path)
        .build()
        .expect("config must load")
}

async fn build_state(config: &Config) -> State {
    let state = State::new();
    let container = DependencyContainer::new();

    ClientModule::register_providers(config, container.provider_registry()).await;
    ClientModule::register_components(container.component_registry());
    container.build_all(&state).expect("container must build");

    state
}

#[tokio::test]
async fn client_is_injected_into_component() {
    let config = load_config();
    let state = build_state(&config).await;

    assert!(
        state.borrow::<UsersClientService>().is_ok(),
        "the client must be injected into the component"
    );
}

#[tokio::test]
#[serial]
async fn client_calls_server_end_to_end() {
    let server = start_server_for_test().await;
    let config = load_config();
    let state = build_state(&config).await;

    let service = state
        .borrow::<UsersClientService>()
        .expect("component must be available");

    let users = service.list().await.expect("list must succeed");

    assert_eq!(users.len(), 2);
    assert_eq!(users[0].username, "alice");
    assert_eq!(users[1].username, "bob");

    server.abort();
}
