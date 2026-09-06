mod builder;
mod config;
mod env;

use std::path::Path;
use sword_core::{Config, sword_error};

pub use builder::ApplicationBuilder;
pub use config::{ApplicationConfig, ApplicationEngine};
pub use env::Environment;

/// The main application struct that holds the runtime(s) and configuration.
///
/// `Application` is the core component of the Sword framework that manages
/// the web server, routing, and application configuration. It provides a
/// builder pattern for configuration and methods to run the application.
pub struct Application {
    engine: ApplicationEngine,
    pub config: Config,

    #[cfg(feature = "events-in-memory")]
    event_shutdown_tx: Option<tokio::sync::watch::Sender<bool>>,
}

impl Application {
    #[cfg(any(feature = "web", feature = "socketio", feature = "grpc"))]
    pub(crate) fn new(
        engine: ApplicationEngine,
        config: Config,
        #[cfg(feature = "events-in-memory")] event_shutdown_tx: Option<
            tokio::sync::watch::Sender<bool>,
        >,
    ) -> Self {
        Self {
            engine,
            config,

            #[cfg(feature = "events-in-memory")]
            event_shutdown_tx,
        }
    }

    /// Creates a new application builder for configuring the application.
    ///
    /// This is the starting point for creating a new Sword application.
    /// The builder pattern allows you to configure various aspects of the
    /// application before building the final `Application` instance.
    ///
    /// The default configuration file is selected from the `SWORD_ENV`
    /// environment variable:
    /// - `SWORD_ENV=dev` loads `config/config.dev.toml`
    /// - `SWORD_ENV=prod` loads `config/config.prod.toml`
    /// - `SWORD_ENV=test` loads `config/config.test.toml`
    /// - If `SWORD_ENV` is not set, the legacy `config/config.toml` is used.
    ///
    /// This function will panic if:
    /// - The selected configuration file cannot be found
    /// - The configuration file contains invalid TOML syntax
    /// - `SWORD_ENV` is set to an invalid value
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
    ///
    /// When the server shuts down, the event subscriber is signaled to stop
    /// processing new events.
    pub async fn run(&self) {
        let app_config = self.config.get_or_default::<ApplicationConfig>();

        tracing::info!(
            target: "sword.startup.app",
            name = app_config.name.as_deref().unwrap_or("none"),
            environment = app_config.environment.as_deref().unwrap_or("none"),
            graceful_shutdown = app_config.graceful_shutdown,
            "Starting Sword application"
        );

        match &self.engine {
            #[cfg(any(feature = "web", feature = "socketio"))]
            ApplicationEngine::Web(app) => app.start().await,

            #[cfg(feature = "grpc")]
            ApplicationEngine::Grpc(app) => app.start().await,

            #[allow(unreachable_patterns)]
            _ => unreachable!(
                "Invalid application engine configuration. Enable the appropriate feature flag to use the desired engine."
            ),
        }

        #[cfg(feature = "events-in-memory")]
        if let Some(tx) = &self.event_shutdown_tx {
            tracing::info!(target: "sword.events", "Signaling event subscriber shutdown");
            let _ = tx.send(true);
        }
    }

    #[allow(irrefutable_let_patterns)]
    #[cfg(any(feature = "web", feature = "socketio"))]
    pub fn router(&self) -> sword_web::internal::AxumRouter {
        #[cfg(any(feature = "web", feature = "socketio"))]
        if let ApplicationEngine::Web(app) = &self.engine {
            return app.router();
        }

        sword_error! {
            title: "Router API is only available for web based applications",
            reason: "Application::router() is only valid for web/socketio applications",
            context: {
                "source" => "Application::router",
            }
        }
    }
}
