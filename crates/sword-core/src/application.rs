use serde::{Deserialize, Serialize};

use crate::LayerStack;
use crate::diagnostic::DiagnosticsMode;
use crate::{Config, ConfigItem, ConfigRegistrar, ControllerRegistry, State, inventory_submit};

/// Configuration structure for the Sword application.
///
/// This struct contains only global application configuration.
///
/// Engine-specific settings live in their own sections such as `[web]`, `[grpc]`,
/// and `[socketio]`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ApplicationConfig {
    /// Optional name of the application. Defaults `None`.
    /// This can be used for logging or display purposes.
    pub name: Option<String>,

    /// Optional environment name (e.g., "development", "production").
    /// This can be used to alter behavior based on the environment. Defaults `None`.
    pub environment: Option<String>,

    /// Renderer used for process-level diagnostics. Defaults to `miette`.
    pub diagnostics: DiagnosticsMode,
}

impl ConfigItem for ApplicationConfig {
    fn key() -> &'static str {
        "application"
    }
}

inventory_submit! {[
    ConfigRegistrar::new(|state, config| {
        state.insert(config.try_get::<ApplicationConfig>().ok().flatten().unwrap_or_default());

        Ok(())
    })
]}

/// Context passed from 'ApplicationBuilder' to engine-specific builders.
///
/// Contains all shared state accumulated during the builder phase,
/// after DI resolution and interceptor registration.
#[derive(Clone)]
pub struct EngineBuildContext {
    pub state: State,
    pub config: Config,
    pub config_document: Option<std::sync::Arc<crate::ConfigDocument>>,
    pub controllers: ControllerRegistry,
    pub layer_stack: LayerStack<State>,
}
