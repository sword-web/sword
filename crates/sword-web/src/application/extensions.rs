use axum::routing::Router;
use std::sync::Arc;
use sword_core::diagnostic::Error;
use sword_core::{Config, ConfigDocument, ControllerMap, State};

#[derive(Clone)]
pub struct WebExtensionContext {
    pub state: State,
    pub config: Config,
    pub config_document: Option<Arc<ConfigDocument>>,
    pub controller_map: ControllerMap,
}

pub trait WebExtension: Send + Sync {
    fn name(&self) -> &'static str;
    fn init_state(&self, _: &WebExtensionContext) -> Result<(), Error> {
        Ok(())
    }

    fn extend_router(
        &self,
        _: &WebExtensionContext,
        router: Router<State>,
    ) -> Result<Router<State>, Error> {
        Ok(router)
    }
}

pub struct WebExtensionRegistrar {
    pub extension: &'static dyn WebExtension,
}

inventory::collect!(WebExtensionRegistrar);
