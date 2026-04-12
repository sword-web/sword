use serde::{Deserialize, Serialize};
use sword_core::{ConfigItem, ConfigRegistrar, inventory_submit};

pub enum ApplicationEngine {
    #[cfg(any(feature = "web", feature = "socketio"))]
    Web(sword_web::application::WebApplication),

    #[cfg(feature = "grpc")]
    Grpc(sword_grpc::application::GrpcApplication),
}

/// Configuration structure for the Sword application.
///
/// This struct contains only global application configuration.
///
/// Engine-specific settings live in their own sections such as `[web]`, `[grpc]`,
/// and `[socketio]`.
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
