use crate::config::WebApplicationConfig;
use crate::router::WebRouter;
use axum::Router;
use std::net::SocketAddr;
use sword_core::{Config, ControllerRegistry, State, sword_error};
use sword_layers::layer_stack::LayerStack;
use tokio::net::TcpListener;

pub struct WebApplication {
    state: State,
    router: Router<State>,
    web_config: WebApplicationConfig,
    graceful_shutdown: bool,
}

impl WebApplication {
    pub fn new(
        state: State,
        config: &Config,
        web_config: WebApplicationConfig,
        graceful_shutdown: bool,
        layers: LayerStack<State>,
        controllers: &ControllerRegistry,
    ) -> Self {
        let router = WebRouter {
            state: state.clone(),
            config,
            layer_stack: layers,
            controller_registry: controllers,
            web_config: web_config.clone(),
        };

        Self {
            state,
            web_config,
            graceful_shutdown,
            router: router.build(),
        }
    }

    pub async fn start(&self) {
        let bind = format!("{}:{}", self.web_config.host, self.web_config.port);

        tracing::info!(
            target: "sword.startup.web",
            bind,
            web_router_prefix = self
                .web_config
                .web_router_prefix
                .as_deref()
                .unwrap_or("none"),
            "Starting application listener"
        );

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
                    "host" => self.web_config.host.clone(),
                    "port" => self.web_config.port.to_string(),
                },
                hints: ["Ensure the host/port is available and not already in use"],
            }
        });

        if self.graceful_shutdown {
            axum::serve(listener, app)
                .with_graceful_shutdown(Self::graceful_signal())
                .await
                .unwrap_or_else(|err| {
                    sword_error! {
                        title: "HTTP server stopped with an internal error",
                        reason: err,
                        context: {
                            "mode" => "graceful_shutdown",
                            "host" => self.web_config.host.clone(),
                            "port" => self.web_config.port.to_string(),
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
                    "host" => self.web_config.host.clone(),
                    "port" => self.web_config.port.to_string(),
                },
            }
        });
    }

    pub fn router(&self) -> axum::Router {
        self.router.clone().with_state(self.state.clone())
    }

    async fn graceful_signal() {
        let ctrl_c = async {
            tokio::signal::ctrl_c().await.unwrap_or_else(|err| {
                sword_error! {
                    title: "Failed to install Ctrl+C handler",
                    reason: err,
                    context: {
                        "source" => "WebApplication::graceful_signal",
                    },
                }
            });
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .unwrap_or_else(|err| {
                    sword_error! {
                        title: "Failed to install SIGTERM handler",
                        reason: err,
                        context: {
                            "signal" => "SIGTERM",
                            "source" => "WebApplication::graceful_signal",
                        },
                    }
                })
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                tracing::info!(
                    target: "sword.startup.signal",
                    signal = "SIGINT",
                    "Shutdown signal received, starting graceful shutdown"
                );
            },
            _ = terminate => {
                tracing::info!(
                    target: "sword.startup.signal",
                    signal = "SIGTERM",
                    "Shutdown signal received, starting graceful shutdown"
                );
            },
        }
    }
}
