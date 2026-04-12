use crate::config::WebApplicationConfig;
use crate::controller::{RouteRegistrar, WebControllerRegistrar};
use crate::response::JsonResponse;

use axum::{Router, extract::Request, middleware::Next};
use std::any::TypeId;
use std::collections::HashMap;

use sword_core::*;
use sword_layers::{
    body_limit::{BodyLimitLayer, BodyLimitValue},
    cookies::CookieManagerLayer,
    layer_stack::LayerStack,
    not_found::NotFoundLayer,
    request_id::RequestIdLayer,
    timeout::{RequestTimeoutResponseLayer, TimeoutLayer},
};

pub(crate) struct WebRouter<'a> {
    pub state: State,
    pub config: &'a Config,
    pub layer_stack: LayerStack<State>,
    pub controller_registry: &'a ControllerRegistry,
    pub web_config: WebApplicationConfig,
}

pub struct WebRouterExtension {
    pub apply: fn(&State, &Config, Router<State>, &ControllerRegistry) -> Router<State>,
}

inventory::collect!(WebRouterExtension);

impl<'a> WebRouter<'a> {
    /// Build the complete HTTP router with all framework controllers and layers.
    pub(crate) fn build(self) -> Router<State> {
        let mut router = Router::new();

        router = Self::apply_controllers(&self.state, router, &self.controller_registry.read());
        router = Self::apply_web_layers(router, &self.web_config);

        for extension in inventory::iter::<WebRouterExtension>() {
            router = (extension.apply)(&self.state, self.config, router, self.controller_registry);
        }

        router = self.layer_stack.apply(router);

        if let Some(prefix) = &self.web_config.router_prefix {
            router = Router::new().nest(prefix, router);
        }

        router = router.route(
            "/health",
            axum::routing::get(|| async { JsonResponse::Ok().message("healthy") }),
        );

        router = router.layer(NotFoundLayer);

        router
    }

    /// Apply all controllers based on kind
    fn apply_controllers(
        state: &State,
        mut router: Router<State>,
        controllers: &ControllerMap,
    ) -> Router<State> {
        for (kind, ids) in controllers.iter() {
            if kind == &Controller::Web {
                router = Self::apply_web_controllers(state, router, ids);
            }
        }

        router
    }

    fn apply_web_controllers(
        state: &State,
        mut router: Router<State>,
        controllers: &ControllerIds,
    ) -> Router<State> {
        let controller_registrars: HashMap<TypeId, &WebControllerRegistrar> =
            inventory::iter::<WebControllerRegistrar>()
                .map(|reg| (reg.controller_id, reg))
                .collect();

        let mut routes_by_controller: HashMap<TypeId, Vec<&RouteRegistrar>> = HashMap::new();

        for route in inventory::iter::<RouteRegistrar>() {
            routes_by_controller
                .entry(route.controller_id)
                .or_default()
                .push(route);
        }

        for controller_id in controllers {
            let controller_registrar = controller_registrars
                .get(controller_id)
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

            (controller_registrar.build)(state);

            let controller_routes = routes_by_controller
                .get(controller_id)
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
                controller_router =
                    controller_router.route(route.path, (route.handler)(state.clone()));
            }

            if controller_registrar.controller_path == "/" {
                router = router.merge(controller_router);
            } else {
                router = router.nest(controller_registrar.controller_path, controller_router);
            }
        }

        router
    }

    /// Apply mandatory web layers.
    ///
    /// These are applied BEFORE the SocketIO layer, so SocketIO traffic bypasses
    /// HTTP controller timeout semantics.
    fn apply_web_layers(
        mut router: Router<State>,
        web_config: &WebApplicationConfig,
    ) -> Router<State> {
        let body_limit_config = web_config.body_limit.clone();

        router = router.layer(BodyLimitLayer::new(&body_limit_config));

        if web_config.request_timeout.enabled {
            let timeout_service: TimeoutLayer = web_config.request_timeout.clone().into();
            let response_mapper = RequestTimeoutResponseLayer::new();

            router = router.layer(timeout_service);
            router = router.layer(response_mapper);
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
}
