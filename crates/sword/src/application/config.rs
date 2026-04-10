use serde::{Deserialize, Serialize};
use sword_core::{ConfigItem, ConfigRegistrar, inventory_submit};
#[cfg(feature = "grpc-controllers")]
use sword_grpc::config::GrpcApplicationConfig;
use sword_web::config::WebApplicationConfig;

pub enum ApplicationEngine {
    #[cfg(any(feature = "web-controllers", feature = "socketio-controllers"))]
    Web(sword_web::application::WebApplication),

    #[cfg(feature = "grpc-controllers")]
    Grpc(sword_grpc::application::GrpcApplication),
}

/// Configuration structure for the Sword application.
///
/// This struct contains all the configuration options that can be specified
/// in the `config/config.toml` file under the `[application]` section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApplicationConfig {
    /// Optional name of the application. Defaults `None`.
    /// This can be used for logging or display purposes.
    pub name: Option<String>,

    /// Optional environment name (e.g., "development", "production").
    /// This can be used to alter behavior based on the environment. Defaults `None`.
    pub environment: Option<String>,

    /// Whether to enable graceful shutdown of the server.
    /// If true, the server will finish processing ongoing requests
    /// before shutting down when a termination signal is received.
    /// Defaults `false`
    #[serde(rename = "graceful-shutdown")]
    pub graceful_shutdown: bool,

    /// Web server settings flattened into `[application]`.
    ///
    /// This keeps the TOML ergonomic while still using a dedicated struct
    /// for web-specific configuration.
    #[serde(flatten)]
    pub web: WebApplicationConfig,

    /// gRPC server settings flattened into `[application]`.
    #[cfg(feature = "grpc-controllers")]
    #[serde(flatten)]
    pub grpc: GrpcApplicationConfig,
}

impl ConfigItem for ApplicationConfig {
    fn key() -> &'static str {
        "application"
    }
}

inventory_submit! {[
    ConfigRegistrar::new(|state, config| {
        state.insert(config.get_or_default::<ApplicationConfig>());
    })
]}
