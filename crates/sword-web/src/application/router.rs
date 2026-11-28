use std::any::TypeId;
use std::collections::HashMap;

use axum::Router;
use axum::middleware::{from_fn, from_fn_with_state};

use sword_core::*;
use sword_extra::request_id::RequestIdLayer;

use crate::application::*;
use crate::controller::{RouteRegistrar, WebControllerRegistrar};

use crate::layers::body_limit::{RequestBodyLimitLayer, body_limit_response_mw};
use crate::layers::cookies::CookieManagerLayer;
use crate::layers::logger::web_logger_mw;
use crate::layers::not_found::{method_not_allowed, not_found};
use crate::layers::timeout::request_timeout_mw;

use crate::response::JsonResponse;

pub struct WebApplicationRouter {
    pub state: State,
    pub config: Config,
    pub controller_registry: ControllerRegistry,
    pub web_config: WebApplicationConfig,
}

impl WebApplicationRouter {
    pub fn build(&mut self) -> Result<Router<State>, Error> {
        // Collect extensions once and let each seed State before controllers and routes are built.
        let extensions = inventory::iter::<WebExtensionRegistrar>()
            .map(|entry| entry.extension)
            .collect::<Vec<&'static dyn WebExtension>>();

        let extension_ctx = WebExtensionContext {
            state: self.state.clone(),
            config: self.config.clone(),
            controller_map: self.controller_registry.snapshot(),
        };

        for extension in &extensions {
            extension.init_state(&extension_ctx)?;
        }

        // Build the API router: controllers, mandatory layers, health, OpenAPI, and services
        // Everything goes into api_router so it's consistently under prefix when configured
        let mut api_router = Router::new();

        // Health is registered before the mandatory layers so it goes through them too.
        api_router = api_router.route(
            "/health",
            axum::routing::get(|| async { JsonResponse::Ok().message("healthy") }),
        );

        api_router = self.apply_web_controllers(api_router)?;

        // Unsupported methods on registered routes answer with the JSON envelope; axum adds
        // the `Allow` header.
        api_router = api_router.method_not_allowed_fallback(method_not_allowed);

        api_router = self.apply_web_layers(api_router);

        #[cfg(any(feature = "swagger-ui", feature = "scalar-ui"))]
        let (mut api_router, openapi_urls) = self.apply_openapi_specs(api_router);

        // Static file serving is mounted before the prefix is applied, like the rest of the
        // API router.
        #[cfg(feature = "servedir")]
        {
            use crate::layers::servedir::{ServeDirConfig, ServeDirLayer};

            let servedir_config = self.config.get_or_default::<ServeDirConfig>();
            let servedir_path = servedir_config.router_path.clone();

            servedir_config.display();

            let layer = ServeDirLayer::from(servedir_config);

            api_router = api_router.nest_service(&servedir_path, layer);
        }

        api_router = api_router.fallback(not_found);

        // Nest the API router under the global prefix, or use it directly
        let mut router = if let Some(prefix) = &self.web_config.router_prefix {
            Router::new().nest(prefix, api_router)
        } else {
            api_router
        };

        // Swagger UI must be merged at the outer router level (after prefix nesting)
        // because utoipa_swagger_ui creates redirects (e.g. /docs → /docs/) with hardcoded
        // Location headers that don't account for Axum's prefix nesting
        #[cfg(feature = "swagger-ui")]
        {
            router = self.apply_openapi_swagger_ui(router, &openapi_urls);
        }

        // Scalar is mounted at the same outer level for consistency with Swagger UI.
        #[cfg(feature = "scalar-ui")]
        {
            router = self.apply_openapi_scalar_ui(router, &openapi_urls);
        }

        // Paths outside the router prefix also get the JSON not-found response. It is set
        // before the extensions so mounted layers such as Socket.IO still wrap it.
        router = router.fallback(not_found);

        for extension in extensions {
            router = extension.extend_router(&extension_ctx, router)?;
        }

        // Cookie parsing first, then user layers. The built-in response layers sit outside them
        // so they also see responses produced by mounted services, and NotFound is outermost so
        // it only rewrites unmatched requests.
        router = router.layer(CookieManagerLayer::new());

        #[cfg(feature = "cors")]
        {
            use crate::layers::cors::{CorsConfig, CorsLayer};

            let cors_config = self.config.get_or_default::<CorsConfig>();
            cors_config.display();

            router = router.layer(CorsLayer::from(cors_config));
        }

        #[cfg(feature = "compression")]
        {
            use crate::layers::compression::{CompressionConfig, CompressionLayer};

            let compression_config = self.config.get_or_default::<CompressionConfig>();
            compression_config.display();

            router = router.layer(CompressionLayer::from(compression_config));
        }

        // Request id and the access logger sit outside the user layers and the built-in
        // response layers, so every response carries an id and every request is logged,
        // including unmatched routes and mounted services.
        router = router.layer(RequestIdLayer::new());

        if let Some(logger_config) = &self.web_config.logger
            && logger_config.enabled
        {
            router = router.layer(from_fn_with_state(logger_config.clone(), web_logger_mw));
        }

        Ok(router)
    }

    fn apply_web_controllers(&mut self, mut router: Router<State>) -> Result<Router<State>, Error> {
        let controller_registrars = inventory::iter::<WebControllerRegistrar>()
            .map(|reg| (reg.controller_id, reg))
            .collect::<HashMap<TypeId, &WebControllerRegistrar>>();

        // inventory has no guaranteed order, so routes are bucketed by controller_id instead of
        // trusting iteration order.
        let mut routes_by_controller: HashMap<TypeId, Vec<&RouteRegistrar>> = HashMap::new();

        for route in inventory::iter::<RouteRegistrar>() {
            routes_by_controller
                .entry(route.controller_id)
                .or_default()
                .push(route);
        }

        for controller_id in self.controller_registry.get_by_kind(Controller::Web) {
            let Some(controller_registrar) = controller_registrars.get(&controller_id).copied()
            else {
                return Err(sword_error! {
                    code: "sword.build.router",
                    phase: DiagnosticPhase::Build,
                    target: "controller registry",
                    title: "Controller metadata not found",
                    reason: "No WebControllerRegistrar entry was found for controller",
                    context: {
                        "controller_id" => format!("{controller_id:?}"),
                    },
                    hints: ["This usually indicates a controller macro expansion issue"],
                });
            };

            // Build the controller into State before resolving its route handlers.
            (controller_registrar.build)(&self.state)?;

            let controller_routes = routes_by_controller
                .get(&controller_id)
                .cloned()
                .unwrap_or_default();

            if controller_routes.is_empty() {
                return Err(sword_error! {
                    code: "sword.build.router",
                    phase: DiagnosticPhase::Build,
                    target: "controller registry",
                    title: "Controller has no registered routes",
                    reason: "No RouteRegistrar entries were found for controller",
                    context: {
                        "controller_id" => format!("{controller_id:?}"),
                    },
                    hints: ["This usually indicates a controller macro expansion issue"],
                });
            }

            let mut controller_router = Router::new();

            for route in controller_routes {
                // Each route handler closes over State and produces its own MethodRouter.
                let route_handler = (route.handler)(self.state.clone())?;
                controller_router = controller_router.route(route.path, route_handler);
            }

            // "/" merges into the app root; any other path nests routes under the controller's
            // declared base path.
            match controller_registrar.controller_path {
                "/" => {
                    router = router.merge(controller_router);
                }
                _ => {
                    router = router.nest(controller_registrar.controller_path, controller_router);
                }
            }
        }

        Ok(router)
    }

    /// Apply mandatory web layers.
    ///
    /// These are applied BEFORE the SocketIO layer, so SocketIO traffic bypasses
    /// HTTP controller timeout semantics.
    fn apply_web_layers(&self, mut router: Router<State>) -> Router<State> {
        router = router.layer(RequestBodyLimitLayer::new(
            self.web_config.body_limit.max_size.parsed,
        ));
        router = router.layer(from_fn(body_limit_response_mw));

        // The timeout is applied before the response layers so it can cancel the whole chain.
        if self.web_config.request_timeout.enabled {
            self.web_config.request_timeout.display();
            router = router.layer(from_fn_with_state(
                self.web_config.request_timeout.clone(),
                request_timeout_mw,
            ));
        }

        router
    }

    #[cfg(any(feature = "swagger-ui", feature = "scalar-ui"))]
    fn apply_openapi_specs(&self, mut router: Router<State>) -> (Router<State>, Vec<String>) {
        let Some(openapi) = &self.web_config.openapi else {
            return (router, vec![]);
        };

        let mut urls: Vec<String> = Vec::new();
        let prefix = self.web_config.router_prefix.as_deref().unwrap_or("");

        for spec_path in &openapi.spec_file_paths {
            let file_extension = spec_path
                .rsplit('.')
                .next()
                .unwrap_or_default()
                .to_lowercase();

            let spec_file_content_type = match file_extension.as_str() {
                "yaml" | "yml" => "application/x-yaml",
                "json" => "application/json",
                ext => {
                    Diagnostic::builder()
                        .code("sword.build.web")
                        .severity(Severity::Warning)
                        .phase(DiagnosticPhase::Build)
                        .target(spec_path.clone())
                        .title("Unsupported OpenAPI spec file type")
                        .reason(format!("extension `{ext}` is not supported"))
                        .context("file_extension", ext.to_string())
                        .hint("Supported extensions are .yaml, .yml, and .json")
                        .build()
                        .report();

                    continue;
                }
            };

            let file_content = match std::fs::read_to_string(spec_path) {
                Ok(c) => c,
                Err(err) => {
                    Diagnostic::builder()
                        .code("sword.build.web")
                        .severity(Severity::Warning)
                        .phase(DiagnosticPhase::Build)
                        .target(spec_path.clone())
                        .title("Failed to read OpenAPI spec file")
                        .reason(err.to_string())
                        .hint("Ensure the file exists and is readable")
                        .build()
                        .report();

                    continue;
                }
            };

            let filename = spec_path.rsplit('/').next().unwrap_or("openapi.yaml");

            let route_path = format!("/openapi/{filename}");

            router = router.route(
                &route_path,
                axum::routing::get(move || async move {
                    (
                        [("Content-Type", spec_file_content_type)],
                        file_content.clone(),
                    )
                }),
            );

            // Full URL with prefix so the docs UI fetches from the correct path
            urls.push(format!("{prefix}{route_path}"));
        }

        (router, urls)
    }

    /// Serves the Swagger UI. It lives at `{prefix}/docs`, or at `{prefix}/docs/swagger-ui`
    /// when `scalar-ui` is also enabled so both UIs can coexist.
    #[cfg(feature = "swagger-ui")]
    fn apply_openapi_swagger_ui(&self, router: Router<State>, urls: &[String]) -> Router<State> {
        use utoipa_swagger_ui::{Config, SwaggerUi};

        if urls.is_empty() {
            return router;
        }

        let prefix = self.web_config.router_prefix.as_deref().unwrap_or("");
        let docs_path = if cfg!(feature = "scalar-ui") {
            format!("{prefix}/docs/swagger-ui")
        } else {
            format!("{prefix}/docs")
        };

        router.merge(SwaggerUi::new(docs_path).config(Config::new(urls.to_vec())))
    }

    /// Serves the Scalar API reference. It lives at `{prefix}/docs`, or at `{prefix}/docs/scalar-ui`
    /// when `swagger-ui` is also enabled so both UIs can coexist.
    #[cfg(feature = "scalar-ui")]
    fn apply_openapi_scalar_ui(&self, router: Router<State>, urls: &[String]) -> Router<State> {
        use scalar_api_reference::axum::router as scalar_router;
        use serde_json::json;

        if urls.is_empty() {
            return router;
        }

        let sources = urls
            .iter()
            .map(|url| json!({ "url": url }))
            .collect::<Vec<_>>();

        let config = json!({ "sources": sources });

        let prefix = self.web_config.router_prefix.as_deref().unwrap_or("");
        let docs_path = if cfg!(feature = "swagger-ui") {
            format!("{prefix}/docs/scalar-ui")
        } else {
            format!("{prefix}/docs")
        };
        let trailing_slash = format!("{docs_path}/");
        let redirect_target = docs_path.clone();

        router
            .merge(scalar_router(&docs_path, &config).with_state::<State>(()))
            .route(
                &trailing_slash,
                axum::routing::get(move || async move {
                    axum::response::Redirect::permanent(&redirect_target)
                }),
            )
    }
}
