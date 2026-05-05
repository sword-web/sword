use crate::prelude::{
    HandlerRegistrar, SocketIoHandlerRegistrar, SocketIoParser, SocketIoServerConfig,
    SocketIoServerLayer,
};

use axum::{Router, extract::Request, middleware::Next};
use socketioxide::SocketIo;
use socketioxide::layer::SocketIoLayer;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use sword_core::{Config, Controller, ControllerMap, State, sword_error};
use sword_layers::DisplayConfig;
use sword_web::application::{WebExtension, WebExtensionContext, WebExtensionRegistrar};

struct SocketIoWebExtension;

impl SocketIoWebExtension {
    fn get_config(config: &Config) -> SocketIoServerConfig {
        config.get_or_default::<SocketIoServerConfig>()
    }

    fn register_handlers(state: &State, controller_map: &ControllerMap) {
        let Some(handlers) = controller_map.get(&Controller::SocketIo) else {
            return;
        };

        let setup_fns = inventory::iter::<SocketIoHandlerRegistrar>()
            .map(|setup| (setup.handler_type_id, setup))
            .collect::<HashMap<TypeId, &SocketIoHandlerRegistrar>>();

        let handler_controllers = inventory::iter::<HandlerRegistrar>()
            .map(|handler| handler.controller_type_id)
            .collect::<HashSet<TypeId>>();

        for handler_id in handlers {
            let Some(setup) = setup_fns.get(handler_id) else {
                if handler_controllers.contains(handler_id) {
                    sword_error! {
                        title: "Controller has handlers but no setup function",
                        reason: "SocketIoHandlerRegistrar is missing for controller",
                        context: {
                            "handler_id" => format!("{handler_id:?}"),
                            "source" => "SocketIoWebExtension::register_handlers",
                        },
                        hints: ["Verify #[controller(kind = Controller::SocketIo, namespace = \"...\")] and #[on(...)] annotations are applied correctly"],
                    };
                }

                continue;
            };

            (setup.setup_fn)(state);
        }
    }
}

impl WebExtension for SocketIoWebExtension {
    fn name(&self) -> &'static str {
        "socketio"
    }

    fn init_state(&self, ctx: &WebExtensionContext) {
        let state = &ctx.state;

        if state.get::<SocketIo>().is_ok() && state.get::<SocketIoLayer>().is_ok() {
            return;
        }

        let socketio_config = Self::get_config(&ctx.config);
        let (layer, io) = SocketIoServerLayer::new(&socketio_config);

        state.insert(io);
        state.insert(layer);

        socketio_config.display();
    }

    fn extend_router(&self, ctx: &WebExtensionContext, mut router: Router<State>) -> Router<State> {
        let state = &ctx.state;
        let socketio_config = Self::get_config(&ctx.config);

        let socketio_layer = state.get::<SocketIoLayer>().unwrap_or_else(|err| {
            sword_error! {
                title: "Socket.IO layer not found in application state",
                reason: err,
                context: {
                    "source" => "SocketIoWebExtension::extend_router",
                },
                hints: ["Ensure Socket.IO runtime bootstrap ran before router extension"],
            }
        });

        router = router.layer(socketio_layer);

        router = router.layer(axum::middleware::from_fn(
            move |mut req: Request, next: Next| async move {
                req.extensions_mut()
                    .insert::<SocketIoParser>(socketio_config.parser);

                next.run(req).await
            },
        ));

        Self::register_handlers(state, &ctx.controller_map);

        router
    }
}

inventory::submit! {
    WebExtensionRegistrar {
        extension: &SocketIoWebExtension,
    }
}
