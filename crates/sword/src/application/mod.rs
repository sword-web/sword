mod builder;
mod config;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use sword_core::{Config, sword_error};

pub use builder::ApplicationBuilder;
pub use config::{ApplicationConfig, ApplicationEngine};

static BOOTSTRAP_CONFIG_PATH: OnceLock<Option<String>> = OnceLock::new();

#[doc(hidden)]
pub fn set_bootstrap_config_path(path: Option<String>) {
    let _ = BOOTSTRAP_CONFIG_PATH.set(path);
}

fn normalize_path(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn validate_metadata_path(path: &Path, source: &str) {
    let Some(expected) = BOOTSTRAP_CONFIG_PATH.get().and_then(|p| p.as_deref()) else {
        return;
    };

    let provided = normalize_path(path);
    let expected = normalize_path(Path::new(expected));

    if provided != expected {
        sword_error! {
            title: "Config path mismatch with Sword bootstrap metadata",
            reason: "Application config path does not match package.metadata.sword.config-path",
            context: {
                "source" => source,
                "expected_path" => expected.display().to_string(),
                "provided_path" => provided.display().to_string(),
            },
            hints: [
                "Use Application::builder() to load the declared config path",
                "Or pass the same path declared in [package.metadata.sword].config-path",
            ],
        }
    }
}

fn validate_config_sources(config: &Config, source: &str) {
    let Some(expected) = BOOTSTRAP_CONFIG_PATH.get().and_then(|p| p.as_deref()) else {
        return;
    };

    let expected = normalize_path(Path::new(expected));
    let has_expected_source = config
        .file_sources()
        .any(|path| normalize_path(path) == expected);

    if !has_expected_source {
        sword_error! {
            title: "Config sources mismatch with Sword bootstrap metadata",
            reason: "The Config provided to Sword does not include package.metadata.sword.config-path",
            context: {
                "source" => source,
                "expected_path" => expected.display().to_string(),
                "provided_sources" => format!("{:?}", config.sources()),
            },
            hints: [
                "Include the declared metadata path in Config::builder() sources",
                "Or use Application::builder()/Application::from_config_path with the declared path",
            ],
        }
    }
}

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
    #[cfg(any(feature = "web", feature = "socketio", feature = "grpc"))]
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
        validate_config_sources(&config, "Application::from_config");
        ApplicationBuilder::from_config(config)
    }

    /// Creates a new application builder by loading configuration from a custom path.
    pub fn from_config_path<P: AsRef<Path>>(path: P) -> ApplicationBuilder {
        validate_metadata_path(path.as_ref(), "Application::from_config_path");
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
        validate_config_sources(&self.config, "Application::run");
        let app_config = self.config.get_or_default::<ApplicationConfig>();

        tracing::info!(
            target: "sword.startup.app",
            name = app_config.name.as_deref().unwrap_or("unknown"),
            environment = app_config.environment.as_deref().unwrap_or("unknown"),
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
    }

    #[cfg(any(feature = "web", feature = "socketio"))]
    pub fn router(&self) -> axum::Router {
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
