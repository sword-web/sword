use crate::application::*;
use crate::controller::{RouteRegistrar, WebControllerRegistrar};
use crate::response::JsonResponse;

use axum::{Router, extract::Request, middleware::Next};
use std::any::TypeId;
use std::collections::HashMap;

use sword_core::*;
use sword_layers::prelude::*;

pub(crate) struct WebApplicationRouter {
    pub state: State,
    pub config: Config,
    pub layer_stack: LayerStack<State>,
    pub controller_registry: ControllerRegistry,
    pub web_config: WebApplicationConfig,
}

impl WebApplicationRouter {
    pub fn build(&mut self) -> Router<State> {
        let mut router = Router::new();

        let extensions = inventory::iter::<WebExtensionRegistrar>()
            .map(|entry| entry.extension)
            .collect::<Vec<&'static dyn WebExtension>>();

        let extension_ctx = WebExtensionContext {
            state: self.state.clone(),
            config: self.config.clone(),
            controller_map: self.controller_registry.snapshot(),
        };

        for extension in &extensions {
            extension.init_state(&extension_ctx);
        }

        router = self.apply_web_controllers(router);
        router = self.apply_web_layers(router);

        if let Some(prefix) = &self.web_config.router_prefix {
            router = Router::new().nest(prefix, router);
        }

        for extension in extensions {
            router = extension.extend_router(&extension_ctx, router);
        }

        router = self.layer_stack.apply(router);

        router = router.route(
            "/health",
            axum::routing::get(|| async { JsonResponse::Ok().message("healthy") }),
        );

        #[cfg(feature = "swagger-ui")]
        {
            router = self.apply_openapi(router);
        }

        router = router.layer(NotFoundLayer);

        router
    }

    fn apply_web_controllers(&mut self, mut router: Router<State>) -> Router<State> {
        let controller_registrars = inventory::iter::<WebControllerRegistrar>()
            .map(|reg| (reg.controller_id, reg))
            .collect::<HashMap<TypeId, &WebControllerRegistrar>>();

        let mut routes_by_controller: HashMap<TypeId, Vec<&RouteRegistrar>> = HashMap::new();

        for route in inventory::iter::<RouteRegistrar>() {
            routes_by_controller
                .entry(route.controller_id)
                .or_default()
                .push(route);
        }

        for controller_id in self.controller_registry.get_by_kind(Controller::Web) {
            let controller_registrar = controller_registrars
                .get(&controller_id)
                .copied()
                .unwrap_or_else(|| {
                    sword_error! {
                        title: "Controller metadata not found",
                        reason: "No WebControllerRegistrar entry was found for controller",
                        context: {
                            "controller_id" => format!("{controller_id:?}"),
                            "source" => "WebRouter::apply_http_controllers",
                        },
                        hints: ["This usually indicates a controller macro expansion issue"],
                    }
                });

            (controller_registrar.build)(&self.state);

            let controller_routes = routes_by_controller
                .get(&controller_id)
                .cloned()
                .unwrap_or_default();

            if controller_routes.is_empty() {
                sword_error! {
                    title: "Controller has no registered routes",
                    reason: "No RouteRegistrar entries were found for controller",
                    context: {
                        "controller_id" => format!("{controller_id:?}"),
                        "source" => "WebRouter::apply_http_controllers",
                    },
                    hints: ["This usually indicates a controller macro expansion issue"],
                }
            }

            let mut controller_router = Router::new();

            for route in controller_routes {
                let route_handler = (route.handler)(self.state.clone());
                controller_router = controller_router.route(route.path, route_handler);
            }

            match controller_registrar.controller_path {
                "/" => {
                    router = router.merge(controller_router);
                }
                _ => {
                    router = router.nest(controller_registrar.controller_path, controller_router);
                }
            }
        }

        router
    }

    /// Apply mandatory web layers.
    ///
    /// These are applied BEFORE the SocketIO layer, so SocketIO traffic bypasses
    /// HTTP controller timeout semantics.
    fn apply_web_layers(&self, mut router: Router<State>) -> Router<State> {
        let body_limit_config = self.web_config.body_limit.clone();

        router = router.layer(BodyLimitLayer::new(&body_limit_config));

        if self.web_config.request_timeout.enabled {
            router = router.layer(TimeoutLayer::from(self.web_config.request_timeout.clone()));
            router = router.layer(RequestTimeoutResponseLayer::new());
        }

        router = router.layer(axum::middleware::from_fn(
            move |mut req: Request, next: Next| async move {
                req.extensions_mut()
                    .insert(BodyLimitValue(body_limit_config.max_size.parsed));

                next.run(req).await
            },
        ));

        router = router.layer(RequestIdLayer::new());
        router = router.layer(CookieManagerLayer::new());

        router
    }

    #[cfg(feature = "swagger-ui")]
    fn apply_openapi(&self, mut router: Router<State>) -> Router<State> {
        use utoipa_swagger_ui::{Config, SwaggerUi};

        let Some(openapi) = &self.web_config.openapi else {
            return router;
        };

        let mut urls: Vec<String> = Vec::new();

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
                    sword_error! {
                        title: "Unsupported OpenAPI spec file type",
                        reason: "The OpenAPI spec file has an unsupported extension",
                        context: {
                            "spec_path" => spec_path.clone(),
                            "file_extension" => ext.to_string(),
                            "source" => "WebRouter::apply_openapi",
                        },
                        hints: ["Supported extensions are .yaml, .yml, and .json"],
                        fatal: false,
                    }
                    continue;
                }
            };

            let file_content = match std::fs::read_to_string(spec_path) {
                Ok(c) => c,
                Err(err) => {
                    sword_error! {
                        title: "Failed to read OpenAPI spec file",
                        reason: "An error occurred while reading the OpenAPI spec file",
                        context: {
                            "spec_path" => spec_path.clone(),
                            "error" => err.to_string(),
                            "source" => "WebRouter::apply_openapi",
                        },
                        hints: ["Ensure the file exists and is readable"],
                        fatal: false,
                    }
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

            urls.push(route_path);
        }

        if !urls.is_empty() {
            router = router.merge(SwaggerUi::new("/docs").config(Config::new(urls.clone())));
        }

        router
    }
}
