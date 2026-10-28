mod builder;
mod engine;
mod env;

use std::path::Path;
use sword_core::diagnostic::{DiagnosticPhase, Error};
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
    /// Starts a new application from the configuration selected by `SWORD_ENV`.
    ///
    /// `SWORD_ENV` can be `dev`, `prod` or `test`; when it is missing, the legacy
    /// `config/config.toml` is used. Loading the file, reading the environment
    /// and initializing tracing can all fail, so the builder is returned inside
    /// a `Result`.
    pub fn builder() -> Result<ApplicationBuilder, Error> {
        ApplicationBuilder::new()
    }

    /// Creates a new application builder from an existing configuration.
    pub fn from_config(config: Config) -> Result<ApplicationBuilder, Error> {
        ApplicationBuilder::from_config(config)
    }

    /// Creates a new application builder by loading configuration from a custom path.
    pub fn from_config_path<P: AsRef<Path>>(path: P) -> Result<ApplicationBuilder, Error> {
        let config_path = path.as_ref().display().to_string();

        let config = Config::builder()
            .add_required_file(path.as_ref())
            .build()
            .map_err(|err| {
                sword_error! {
                    code: "sword.build.config",
                    phase: DiagnosticPhase::Build,
                    target: config_path,
                    title: "Failed to load configuration from a custom path",
                    reason: err,
                    source: "Application::from_config_path",
                    hints: ["Ensure the file exists and contains valid TOML"],
                    cause: err,
                }
            })?;

        ApplicationBuilder::from_config(config)
    }

    /// Runs every configured engine until the application stops.
    ///
    /// All engines start together. A termination signal stops all of them
    /// immediately. If an engine fails or panics, the remaining engines are
    /// stopped immediately as well.
    pub async fn run(self) -> Result<(), Error> {
        let app_config = self.config.get_or_default::<ApplicationConfig>();

        tracing::info!(
            target: "sword.startup.app",
            name = app_config.name.as_deref().unwrap_or("none"),
            environment = app_config.environment.as_deref().unwrap_or("none"),
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

        let result = tokio::select! {
            _ = sword_core::shutdown_signal() => {
                shutdown.cancel();
                tasks.abort_all();
                return Ok(());
            }
            result = tasks.join_next() => result,
        };

        shutdown.cancel();
        tasks.abort_all();

        match result {
            Some(Ok(Ok(()))) => Err(engine_stopped()),
            Some(Ok(Err(error))) => Err(error),
            Some(Err(join_error)) => Err(engine_panicked(join_error)),
            None => Ok(()),
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

        Err(sword_error! {
            code: "sword.runtime.router",
            phase: DiagnosticPhase::Runtime,
            target: "Application::router",
            title: "Router API is only available for web based applications",
            reason: "The application has no web or Socket.IO engine",
            hints: ["Enable the web or socketio feature and call Application::router() there"],
        })
    }
}

fn engine_stopped() -> Error {
    sword_error! {
        code: "sword.runtime.engine",
        phase: DiagnosticPhase::Runtime,
        target: "Application::run",
        title: "Engine stopped before shutdown",
        reason: "an engine finished without a shutdown request",
        source: "Application::run",
        hints: ["Check the engine logs for the reason it stopped"],
    }
}

fn engine_panicked(error: tokio::task::JoinError) -> Error {
    sword_error! {
        code: "sword.runtime.panic",
        phase: DiagnosticPhase::Runtime,
        target: "Application::run",
        title: "Engine task panicked",
        reason: error,
        source: "Application::run",
        hints: ["Check the panic message in the logs"],
        cause: error,
    }
}
