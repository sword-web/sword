use crate::config::WebApplicationConfig;
use crate::router::WebRouter;
use axum::Router;
use std::net::SocketAddr;
use sword_core::*;
use tokio::net::TcpListener;

pub struct WebApplication {
    pub state: State,
    pub router: Router<State>,
    pub web_config: WebApplicationConfig,
    pub graceful_shutdown: bool,
}

impl WebApplication {
    pub async fn start(&self) {
        let bind = format!("{}:{}", self.web_config.host, self.web_config.port);

        tracing::info!(
            target: "sword.startup.web",
            bind,
            router_prefix = self
                .web_config
                .router_prefix
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
                .with_graceful_shutdown(shutdown_signal())
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

        let router = WebRouter {
            state: state.clone(),
            config: &config,
            layer_stack,
            controller_registry: &controllers,
            web_config: web_config.clone(),
        };

        Self {
            state,
            web_config,
            graceful_shutdown,
            router: router.build(),
        }
    }
}
