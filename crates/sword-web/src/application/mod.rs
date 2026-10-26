mod config;
mod extensions;
mod logger;
mod router;

use axum::Router;
use router::WebApplicationRouter;
use std::net::SocketAddr;
use sword_core::diagnostic::Error;
use sword_core::sword_error;
use sword_core::*;
use tokio::net::TcpListener;

pub use config::WebApplicationConfig;
pub use extensions::{WebExtension, WebExtensionContext, WebExtensionRegistrar};

pub struct WebApplication {
    pub state: State,
    pub router: Router<State>,
    pub config: WebApplicationConfig,
    pub graceful_shutdown: bool,
}

impl WebApplication {
    pub async fn run(self, shutdown: Shutdown) -> Result<(), Error> {
        let bind = format!("{}:{}", self.config.host, self.config.port);
        let router_prefix = self
            .config
            .router_prefix
            .as_deref()
            .unwrap_or("none")
            .to_string();

        tracing::info!(
            target: "sword.startup.web",
            bind,
            router_prefix,
            "Starting application listener"
        );

        #[cfg(feature = "swagger-ui")]
        {
            if let Some(cfg) = &self.config.openapi {
                let display_host = if self.config.host == "0.0.0.0" {
                    "localhost"
                } else {
                    &self.config.host
                };

                let docs_url = format!(
                    "http://{}:{}{}/docs",
                    display_host,
                    self.config.port,
                    self.config.router_prefix.as_deref().unwrap_or("")
                );

                tracing::info!(
                    target: "sword.startup.app",
                    docs_url = docs_url.as_str(),
                    "Loaded {} OpenAPI spec file(s) for Swagger UI",
                    cfg.spec_file_paths.len()
                );
            }
        }

        // The router is built as Router<State>; state is attached just before serving.
        let app = self.router.clone().with_state(self.state.clone());

        let bind_addr: SocketAddr = bind.parse::<SocketAddr>().map_err(|err| {
            sword_error! {
                code: "sword.runtime.web",
                phase: sword_core::diagnostic::DiagnosticPhase::Runtime,
                target: "WebApplication::run",
                title: "Invalid web bind address",
                reason: err,
                context: {
                    "bind" => bind,
                },
                hints: ["Ensure host and port values are valid"],
            }
        })?;

        let listener = TcpListener::bind(bind_addr).await.map_err(|err| {
            sword_error! {
                code: "sword.runtime.web",
                phase: sword_core::diagnostic::DiagnosticPhase::Runtime,
                target: "WebApplication::run",
                title: "Failed to bind HTTP listener",
                reason: err,
                context: {
                    "host" => self.config.host.clone(),
                    "port" => self.config.port.to_string(),
                },
                hints: ["Ensure the host/port is available and not already in use"],
            }
        })?;

        // Graceful shutdown waits for a signal and drains in-flight requests.
        if self.graceful_shutdown {
            axum::serve(listener, app)
                .with_graceful_shutdown(shutdown.cancelled())
                .await
                .map_err(|err| {
                    sword_error! {
                        code: "sword.runtime.web",
                        phase: sword_core::diagnostic::DiagnosticPhase::Runtime,
                        target: "WebApplication::run",
                        title: "HTTP server stopped with an internal error",
                        reason: err,
                        context: {
                            "mode" => "graceful_shutdown",
                            "host" => self.config.host.clone(),
                            "port" => self.config.port.to_string(),
                        },
                        hints: ["Check the web server logs and listener configuration"],
                    }
                })?;

            return Ok(());
        }

        tokio::select! {
            res = axum::serve(listener, app) => {
                res.map_err(|err| {
                    sword_error! {
                        code: "sword.runtime.web",
                        phase: sword_core::diagnostic::DiagnosticPhase::Runtime,
                        target: "WebApplication::run",
                        title: "HTTP server stopped with an internal error",
                        reason: err,
                        context: {
                            "mode" => "normal",
                            "host" => self.config.host.clone(),
                            "port" => self.config.port.to_string(),
                        },
                        hints: ["Check the web server logs and listener configuration"],
                    }
                })?;
            }
            _ = shutdown.cancelled() => {}
        }

        Ok(())
    }

    pub fn router(&self) -> axum::Router {
        self.router.clone().with_state(self.state.clone())
    }
}

impl TryFrom<EngineBuildContext> for WebApplication {
    type Error = Error;

    fn try_from(ctx: EngineBuildContext) -> Result<Self, Error> {
        let EngineBuildContext {
            state,
            config,
            config_document,
            controllers,
            layer_stack,
        } = ctx;

        let web_config =
            ConfigSection::load::<WebApplicationConfig>(&config, config_document.as_deref())?
                .unwrap_or_default();

        let graceful_shutdown = config
            .get_or_default::<ApplicationConfig>()
            .graceful_shutdown;

        let mut router = WebApplicationRouter {
            state: state.clone(),
            config,
            config_document,
            layer_stack,
            controller_registry: controllers,
            web_config: web_config.clone(),
        };

        Ok(Self {
            state,
            config: web_config,
            graceful_shutdown,
            router: router.build()?,
        })
    }
}
