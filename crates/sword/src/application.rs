mod builder;
mod engine;
mod env;

use std::path::Path;
use sword_core::diagnostic::{DiagnosticPhase, Error};
use sword_core::{Config, Shutdown, sword_diagnostic, sword_error};

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
    pub async fn run(self) -> Result<(), Error> {
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
            // Without engines there is nothing to run, but the process should
            // still wait for the shutdown signal instead of exiting.
            sword_core::shutdown_signal().await;
            return Ok(());
        }

        let mut first_error = None;

        tokio::select! {
            _ = sword_core::shutdown_signal() => {}
            result = tasks.join_next() => {
                if let Some(result) = result {
                    first_error = Self::task_error(result);
                }
            }
        }

        shutdown.cancel();

        while let Some(result) = tasks.join_next().await {
            if first_error.is_none() {
                first_error = Self::task_error(result);
            }
        }

        match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    /// Turns the end of an engine task into a diagnostic when it failed.
    ///
    /// A `JoinError` means the task panicked. Tokio only surfaces it when the
    /// task is joined, so without this the process could exit successfully
    /// after a failed startup.
    fn task_error(result: Result<Result<(), Error>, tokio::task::JoinError>) -> Option<Error> {
        match result {
            Ok(Ok(())) => None,
            Ok(Err(error)) => Some(error),
            Err(error) => Some(sword_diagnostic! {
                code: "sword.runtime.application",
                phase: DiagnosticPhase::Runtime,
                target: "Application::run",
                title: "Engine task failed",
                reason: error,
                source: "Application::run",
                hints: ["Check the engine logs and the underlying cause"],
                cause: error,
            }),
        }
    }

    #[cfg(any(feature = "web", feature = "socketio"))]
    #[allow(irrefutable_let_patterns)]
    pub fn router(&self) -> Result<sword_web::internal::AxumRouter, Error> {
        for engine in &self.engines {
            if let Engine::Web(web) = engine {
                return Ok(web.router());
            }
        }

        Err(sword_diagnostic! {
            code: "sword.runtime.router",
            phase: DiagnosticPhase::Runtime,
            target: "Application::router",
            title: "Router API is only available for web based applications",
            reason: "Application::router() is only valid for web/socketio applications",
            hints: ["Enable the web or socketio feature and call Application::router() there"],
        })
    }
}
