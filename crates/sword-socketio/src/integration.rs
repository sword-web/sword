use crate::prelude::{
    HandlerRegistrar, SocketIoHandlerRegistrar, SocketIoParser, SocketIoServerConfig,
    SocketIoServerLayer,
};

use axum::{Router, extract::Request, middleware::Next};
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use sword_core::{Config, State, sword_error};
use sword_core::{Controller, ControllerIds, ControllerRegistry};
use sword_web::router::WebRouterExtension;

fn socketio_setup(
    state: &State,
    socketio_config: &SocketIoServerConfig,
) -> (Option<crate::SocketIoLayer>, SocketIoServerConfig) {
    let socketio_config = socketio_config.clone();

    let layer = socketio_config.enabled.then(|| {
        let (layer, io) = SocketIoServerLayer::new(&socketio_config);
        state.insert(io);
        layer
    });

    (layer, socketio_config)
}

fn apply_socketio_layer(
    mut router: Router<State>,
    layer: crate::SocketIoLayer,
    config: SocketIoServerConfig,
) -> Router<State> {
    router = router.layer(layer);

    router = router.layer(axum::middleware::from_fn(
        move |mut req: Request, next: Next| async move {
            req.extensions_mut().insert::<SocketIoParser>(config.parser);
            next.run(req).await
        },
    ));

    router
}

fn apply_socketio_controllers(state: &State, handlers: &ControllerIds) {
    let setup_fns: HashMap<TypeId, &SocketIoHandlerRegistrar> =
        inventory::iter::<SocketIoHandlerRegistrar>()
            .map(|setup| (setup.handler_type_id, setup))
            .collect();

    let handler_controllers: HashSet<TypeId> = inventory::iter::<HandlerRegistrar>()
        .map(|handler| handler.controller_type_id)
        .collect();

    for handler_id in handlers {
        if let Some(setup) = setup_fns.get(handler_id).copied() {
            (setup.setup_fn)(state);
        } else {
            let has_handlers = handler_controllers.contains(handler_id);

            if has_handlers {
                sword_error! {
                    title: "Controller has handlers but no setup function",
                    reason: "SocketIoHandlerRegistrar is missing for controller",
                    context: {
                        "handler_id" => format!("{handler_id:?}"),
                        "source" => "WebRouterExtension::apply_socketio_handlers",
                    },
                    hints: ["Verify #[controller(kind = Controller::SocketIo, namespace = \"...\")] and #[on(...)] annotations are applied correctly"],
                };
            }
        }
    }
}

fn apply_socketio_extension(
    state: &State,
    config: &Config,
    mut router: Router<State>,
    controller_registry: &ControllerRegistry,
) -> Router<State> {
    let socketio_config = config.get_or_default::<SocketIoServerConfig>();
    let (socketio_layer, socketio_config) = socketio_setup(state, &socketio_config);

    if let Some(layer) = socketio_layer {
        router = apply_socketio_layer(router, layer, socketio_config);
    }

    let controller_map = controller_registry.read();
    if let Some(handlers) = controller_map.get(&Controller::SocketIo) {
        apply_socketio_controllers(state, handlers);
    }

    router
}

inventory::submit! {
    WebRouterExtension {
        apply: apply_socketio_extension,
    }
}
