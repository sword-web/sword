mod config;
mod extensions;
mod router;

use axum::Router;
use router::WebApplicationRouter;
use std::net::SocketAddr;
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
    pub async fn start(&self) {
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

        let app = self.router.clone().with_state(self.state.clone());

        let bind_addr: SocketAddr = bind.parse::<SocketAddr>().unwrap_or_else(|err| {
            sword_error! {
                title: "Invalid web bind address",
                reason: err,
                context: {
                    "bind" => bind,
                    "source" => "WebApplication::start",
                },
                hints: ["Ensure host and port values are valid"],
            }
        });

        let listener = TcpListener::bind(bind_addr).await.unwrap_or_else(|err| {
            sword_error! {
                title: "Failed to bind HTTP listener",
                reason: err,
                context: {
                    "host" => self.config.host.clone(),
                    "port" => self.config.port.to_string(),
                },
                hints: ["Ensure the host/port is available and not already in use"],
            }
        });

        if self.graceful_shutdown {
            axum::serve(listener, app)
                .with_graceful_shutdown(shutdown_signal())
                .await
                .unwrap_or_else(|err| {
                    sword_error! {
                        title: "HTTP server stopped with an internal error",
                        reason: err,
                        context: {
                            "mode" => "graceful_shutdown",
                            "host" => self.config.host.clone(),
                            "port" => self.config.port.to_string(),
                        },
                    }
                });

            return;
        }

        axum::serve(listener, app).await.unwrap_or_else(|err| {
            sword_error! {
                title: "HTTP server stopped with an internal error",
                reason: err,
                context: {
                    "mode" => "normal",
                    "host" => self.config.host.clone(),
                    "port" => self.config.port.to_string(),
                },
            }
        });
    }

    pub fn router(&self) -> axum::Router {
        self.router.clone().with_state(self.state.clone())
    }
}

impl From<EngineBuildContext> for WebApplication {
    fn from(ctx: EngineBuildContext) -> Self {
        let EngineBuildContext {
            state,
            config,
            controllers,
            layer_stack,
        } = ctx;

        let web_config = config.get_or_default::<WebApplicationConfig>();
        let graceful_shutdown = config
            .get_or_default::<ApplicationConfig>()
            .graceful_shutdown;

        let mut router = WebApplicationRouter {
            state: state.clone(),
            config,
            layer_stack,
            controller_registry: controllers,
            web_config: web_config.clone(),
        };

        Self {
            state,
            config: web_config,
            graceful_shutdown,
            router: router.build(),
        }
    }
}
