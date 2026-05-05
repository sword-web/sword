use axum::routing::Router;
use sword_core::{Config, ControllerMap, State};

#[derive(Clone)]
pub struct WebExtensionContext {
    pub state: State,
    pub config: Config,
    pub controller_map: ControllerMap,
}

pub trait WebExtension: Send + Sync {
    fn name(&self) -> &'static str;
    fn init_state(&self, _: &WebExtensionContext) {}

    fn extend_router(&self, _: &WebExtensionContext, router: Router<State>) -> Router<State> {
        router
    }
}

pub struct WebExtensionRegistrar {
    pub extension: &'static dyn WebExtension,
}

inventory::collect!(WebExtensionRegistrar);
