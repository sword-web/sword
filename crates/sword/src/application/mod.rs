mod builder;
mod config;

use std::path::Path;
use sword_core::{Config, sword_error};

pub use builder::ApplicationBuilder;
pub use config::{ApplicationConfig, ApplicationEngine};

/// The main application struct that holds the runtime(s) and configuration.
///
/// `Application` is the core component of the Sword framework that manages
/// the web server, routing, and application configuration. It provides a
/// builder pattern for configuration and methods to run the application.
pub struct Application {
    engine: ApplicationEngine,
    pub config: Config,
}

impl Application {
    pub(crate) fn new(engine: ApplicationEngine, config: Config) -> Self {
        Self { engine, config }
    }

    /// Creates a new application builder for configuring the application.
    ///
    /// This is the starting point for creating a new Sword application.
    /// The builder pattern allows you to configure various aspects of the
    /// application before building the final `Application` instance.
    ///
    /// This function will panic if:
    /// - The configuration file `config/config.toml` cannot be found
    /// - The configuration file contains invalid TOML syntax
    /// - Environment variable interpolation fails
    /// - The configuration cannot be loaded for any other reason
    pub fn builder() -> ApplicationBuilder {
        ApplicationBuilder::new()
    }

    /// Creates a new application builder from an existing configuration.
    pub fn from_config(config: Config) -> ApplicationBuilder {
        ApplicationBuilder::from_config(config)
    }

    /// Creates a new application builder by loading configuration from a custom path.
    pub fn from_config_path<P: AsRef<Path>>(path: P) -> ApplicationBuilder {
        let config_path = path.as_ref().display().to_string();

        ApplicationBuilder::from_config(
            Config::builder()
                .add_required_file(path.as_ref())
                .build()
                .unwrap_or_else(|err| {
                    sword_error! {
                        title: "Failed to load configuration from custom path",
                        reason: err,
                        context: {
                            "path" => config_path,
                            "source" => "Application::from_config_path",
                        },
                        hints: ["Ensure the file exists and contains valid TOML"],
                    }
                }),
        )
    }

    /// Runs the application server.
    ///
    /// This method starts the web server and begins listening for incoming
    /// requests. It will bind to the host and port specified in the
    /// server configuration.
    pub async fn run(&self) {
        let app_config = self.config.get_or_default::<ApplicationConfig>();

        tracing::info!(
            target: "sword.startup.app",
            name = app_config.name.as_deref().unwrap_or("unknown"),
            environment = app_config.environment.as_deref().unwrap_or("unknown"),
            graceful_shutdown = app_config.graceful_shutdown,
            "Starting Sword application"
        );

        match &self.engine {
            #[cfg(any(feature = "web-controllers", feature = "socketio-controllers"))]
            ApplicationEngine::Web(app) => app.start().await,
            #[cfg(feature = "grpc-controllers")]
            ApplicationEngine::Grpc(app) => app.start().await,
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }

    #[cfg(any(feature = "web-controllers", feature = "socketio-controllers"))]
    pub fn router(&self) -> axum::Router {
        match &self.engine {
            ApplicationEngine::Web(app) => app.router(),
            #[cfg(feature = "grpc-controllers")]
            ApplicationEngine::Grpc(_) => {
                sword_error! {
                    title: "Router API is not available for gRPC engine",
                    reason: "Application::router() is only valid for web/socketio applications",
                    context: {
                        "source" => "Application::router",
                    },
                    hints: ["Use Application::run() to start the gRPC server"],
                }
            }
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        }
    }
}
