use serde::{Deserialize, Serialize};

use crate::{Config, ConfigItem, ConfigRegistrar, ControllerRegistry, State, inventory_submit};
use sword_layers::layer_stack::LayerStack;

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

/// Context passed from [`ApplicationBuilder`] to engine-specific builders.
///
/// Contains all shared state accumulated during the builder phase,
/// after DI resolution and interceptor registration.
pub struct EngineBuildContext {
    pub state: State,
    pub config: Config,
    pub controllers: ControllerRegistry,
    pub layer_stack: LayerStack<State>,
}
