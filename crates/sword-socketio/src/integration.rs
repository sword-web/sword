use crate::prelude::{
    HandlerRegistrar, SocketIoHandlerRegistrar, SocketIoParser, SocketIoServerConfig,
    SocketIoServerLayer,
};

use axum::{Router, extract::Request, middleware::Next};
use socketioxide::SocketIo;
use socketioxide::layer::SocketIoLayer;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use sword_core::diagnostic::{DiagnosticPhase, Error};
use sword_core::{Config, ConfigSection, Controller, ControllerMap, State, sword_diagnostic};
use sword_layers::DisplayConfig;
use sword_web::application::{
    WebApplicationConfig, WebExtension, WebExtensionContext, WebExtensionRegistrar,
};

struct SocketIoWebExtension;

impl SocketIoWebExtension {
    fn get_config(config: &Config) -> SocketIoServerConfig {
        config.get_or_default::<SocketIoServerConfig>()
    }

    fn register_handlers(state: &State, controller_map: &ControllerMap) -> Result<(), Error> {
        let Some(handlers) = controller_map.get(&Controller::SocketIo) else {
            return Ok(());
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
                    return Err(sword_diagnostic! {
                        code: "sword.build.socketio",
                        phase: DiagnosticPhase::Build,
                        target: "Socket.IO handler registry",
                        title: "Controller has handlers but no setup function",
                        reason: "SocketIoHandlerRegistrar is missing for controller",
                        context: {
                            "handler_id" => format!("{handler_id:?}"),
                        },
                        hints: ["Verify #[controller(kind = Controller::SocketIo, namespace = \"...\")] and #[on(...)] annotations are applied correctly"],
                    });
                }

                continue;
            };

            (setup.setup_fn)(state)?;
        }

        Ok(())
    }
}

impl WebExtension for SocketIoWebExtension {
    fn name(&self) -> &'static str {
        "socketio"
    }

    fn init_state(&self, ctx: &WebExtensionContext) -> Result<(), Error> {
        let state = &ctx.state;

        let mut socketio_config = ConfigSection::load::<SocketIoServerConfig>(
            &ctx.config,
            ctx.config_document.as_deref(),
        )?
        .unwrap_or_default();

        if state.get::<SocketIo>().is_ok() && state.get::<SocketIoLayer>().is_ok() {
            return Ok(());
        }

        let web_config: WebApplicationConfig = ctx.config.get_or_default();

        if let Some(prefix) = &web_config.router_prefix
            && socketio_config.req_path.is_none()
        {
            socketio_config.req_path = Some(format!("{prefix}/socket.io"));
        }

        let (layer, io) = SocketIoServerLayer::new(&socketio_config);

        state.insert(io);
        state.insert(layer);

        socketio_config.display();

        Ok(())
    }

    fn extend_router(
        &self,
        ctx: &WebExtensionContext,
        mut router: Router<State>,
    ) -> Result<Router<State>, Error> {
        let state = &ctx.state;
        let socketio_config = Self::get_config(&ctx.config);

        let socketio_layer = state.get::<SocketIoLayer>().map_err(|err| {
            sword_diagnostic! {
                code: "sword.build.socketio",
                phase: DiagnosticPhase::Build,
                target: "socketio layer",
                title: "Socket.IO layer not found in application state",
                reason: err,
                source: "SocketIoWebExtension::extend_router",
                hints: ["Ensure Socket.IO runtime bootstrap ran before router extension"],
            }
        })?;

        router = router.layer(socketio_layer);

        router = router.layer(axum::middleware::from_fn(
            move |mut req: Request, next: Next| async move {
                req.extensions_mut()
                    .insert::<SocketIoParser>(socketio_config.parser);

                next.run(req).await
            },
        ));

        Self::register_handlers(state, &ctx.controller_map)?;

        Ok(router)
    }
}

inventory::submit! {
    WebExtensionRegistrar {
        extension: &SocketIoWebExtension,
    }
}
