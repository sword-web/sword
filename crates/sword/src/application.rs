mod builder;
mod engine;
mod env;

use std::path::Path;
use sword_core::{Config, Shutdown, sword_error};

pub use builder::ApplicationBuilder;
pub use env::Environment;
pub use sword_core::ApplicationConfig;

pub(crate) use engine::Engine;

/// The main application struct that holds the configured engines and the
/// application configuration.
///
/// `Application` runs every engine together and stops them all on shutdown.
pub struct Application {
    engines: Vec<Engine>,

    pub config: Config,
}

impl Application {
    pub(crate) fn new(engines: Vec<Engine>, config: Config) -> Self {
        Self { engines, config }
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

    /// Runs every configured engine until the application shuts down.
    ///
    /// All engines start together. The application stops when the OS shutdown
    /// signal arrives or when any engine finishes on its own; then it cancels
    /// the shared [`Shutdown`] and waits for every engine to drain.
    pub async fn run(self) {
        let app_config = self.config.get_or_default::<ApplicationConfig>();

        tracing::info!(
            target: "sword.startup.app",
            name = app_config.name.as_deref().unwrap_or("none"),
            environment = app_config.environment.as_deref().unwrap_or("none"),
            graceful_shutdown = app_config.graceful_shutdown,
            "Starting Sword application"
        );

        let shutdown = Shutdown::new();

        let mut tasks = tokio::task::JoinSet::new();

        for engine in self.engines {
            tracing::info!(
                target: "sword.startup.engine",
                engine = engine.name(),
                "Starting engine"
            );

            tasks.spawn(engine.run(shutdown.clone()));
        }

        if tasks.is_empty() {
            // No engines: stay alive until shutdown.
            sword_core::shutdown_signal().await;
            return;
        }

        tokio::select! {
            _ = sword_core::shutdown_signal() => {}
            _ = tasks.join_next() => {}
        }

        shutdown.cancel();

        while tasks.join_next().await.is_some() {}
    }

    #[cfg(any(feature = "web", feature = "socketio"))]
    #[allow(irrefutable_let_patterns)]
    pub fn router(&self) -> sword_web::internal::AxumRouter {
        for engine in &self.engines {
            if let Engine::Web(web) = engine {
                return web.router();
            }
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
