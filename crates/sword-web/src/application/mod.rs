mod config;
mod extensions;
mod router;
#[cfg(feature = "scalar-ui")]
mod scalar;

use std::net::SocketAddr;

use axum::Router;
use sword_core::*;
use tokio::net::TcpListener;

use self::router::WebApplicationRouter;
use crate::layers::body_limit::BodyLimitValue;

#[cfg(any(feature = "swagger-ui", feature = "scalar-ui"))]
pub use self::config::OpenApiConfig;
pub use self::config::WebApplicationConfig;
pub use self::extensions::{WebExtension, WebExtensionContext, WebExtensionRegistrar};

pub struct WebApplication {
    state: State,
    router: Router<State>,
    config: WebApplicationConfig,
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

        #[cfg(any(feature = "swagger-ui", feature = "scalar-ui"))]
        {
            if let Some(cfg) = &self.config.openapi {
                let display_host = if self.config.host == "0.0.0.0" {
                    "localhost"
                } else {
                    &self.config.host
                };

                // With both UIs enabled Swagger UI keeps `/docs` and Scalar moves to `/scalar`.
                let docs_route = if cfg!(feature = "scalar-ui") && cfg!(feature = "swagger-ui") {
                    "/docs and /scalar"
                } else {
                    "/docs"
                };

                let docs_url = format!(
                    "http://{}:{}{}{}",
                    display_host,
                    self.config.port,
                    self.config.router_prefix.as_deref().unwrap_or(""),
                    docs_route
                );

                tracing::info!(
                    target: "sword.startup.app",
                    docs_url = docs_url.as_str(),
                    "Loaded {} OpenAPI spec file(s) for the API docs UI",
                    cfg.spec_file_paths.len()
                );
            }
        }

        // The router is built as Router<State>; state is attached just before serving.
        let app = self.router.clone().with_state(self.state.clone());

        let bind_addr: SocketAddr = bind.parse::<SocketAddr>().map_err(|err| {
            sword_error! {
                code: "sword.runtime.web",
                phase: DiagnosticPhase::Startup,
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
                phase: DiagnosticPhase::Startup,
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

        // A signal stops accepting new connections and drops the server.
        tokio::select! {
            res = axum::serve(listener, app) => {
                res.map_err(|err| {
                    sword_error! {
                        code: "sword.runtime.web",
                        phase: DiagnosticPhase::Runtime,
                        target: "WebApplication::run",
                        title: "HTTP server stopped with an internal error",
                        reason: err,
                        context: {
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

    /// Returns the router with the application state attached.
    ///
    /// It is the router the engine serves, so use it to embed the application
    /// in another server or to build a test server.
    pub fn router(&self) -> Router {
        self.router.clone().with_state(self.state.clone())
    }

    /// Replaces the web router with the result of `f`.
    ///
    /// The closure receives the router before the state is attached, so any
    /// layer, service or fallback added there becomes part of the served
    /// application.
    pub fn with_router<F>(&mut self, f: F)
    where
        F: FnOnce(Router<State>) -> Router<State>,
    {
        let router = std::mem::take(&mut self.router);

        self.router = f(router);
    }
}

impl TryFrom<EngineBuildContext> for WebApplication {
    type Error = Error;

    fn try_from(ctx: EngineBuildContext) -> Result<Self, Error> {
        let EngineBuildContext {
            state,
            config,
            controllers,
        } = ctx;

        let web_config = config.get_or_default::<WebApplicationConfig>();

        // The extractor reads the limit from State so it always matches the value
        // enforced by BodyLimitLayer.
        state.insert(BodyLimitValue::from(web_config.body_limit.clone()));

        let mut router = WebApplicationRouter {
            state: state.clone(),
            config,
            controller_registry: controllers,
            web_config: web_config.clone(),
        };

        Ok(Self {
            state,
            config: web_config,
            router: router.build()?,
        })
    }
}
