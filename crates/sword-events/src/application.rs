#[cfg(feature = "memory")]
use crate::memory::MemoryTransport;

use crate::{
    EventApplicationConfig, EventControllerRegistrar, EventHandlerRegistrar, EventKeyHandlerMap,
};

use std::any::TypeId;
use std::collections::HashMap;
use sword_core::{
    Config, Controller, ControllerRegistry, EventSource, ProviderRegistry, Shutdown, State,
};

pub enum EventTransport {
    #[cfg(feature = "memory")]
    Memory(MemoryTransport),

    #[cfg(feature = "nats")]
    Nats(()),
}

/// Background engine that consumes published events and dispatches them to the
/// registered handlers.
pub struct EventApplication {
    config: EventApplicationConfig,
    transports: Vec<EventTransport>,
}

impl EventApplication {
    pub fn new(config: &Config, providers: &ProviderRegistry) -> Self {
        let events_config = config.get_or_default::<EventApplicationConfig>();

        let mut transports = Vec::new();

        #[cfg(feature = "memory")]
        {
            let (publisher, transport) = MemoryTransport::new(&events_config);

            providers.register(publisher);
            transports.push(EventTransport::Memory(transport));
        }

        #[cfg(feature = "nats")]
        {}

        Self {
            config: events_config,
            transports,
        }
    }

    pub async fn run(self, _: Shutdown) {}
}

fn build_handlers(state: &State, controllers: &ControllerRegistry) -> EventKeyHandlerMap {
    let event_controllers = controllers.get_by_kind(Controller::EventHandler);

    if event_controllers.is_empty() {
        return HashMap::new();
    }

    let controller_registrars: HashMap<TypeId, &EventControllerRegistrar> =
        inventory::iter::<EventControllerRegistrar>()
            .map(|r| (r.handler_type_id, r))
            .collect();

    let mut route_map: HashMap<TypeId, Vec<&EventHandlerRegistrar>> = HashMap::new();

    for route in inventory::iter::<EventHandlerRegistrar>() {
        route_map
            .entry(route.handler_type_id)
            .or_default()
            .push(route);
    }

    let mut handlers: EventKeyHandlerMap = HashMap::new();

    for type_id in &event_controllers {
        let Some(registrar) = controller_registrars.get(type_id) else {
            tracing::warn!(
                target: "sword.events",
                "No EventControllerRegistrar found for handler type {:?}",
                type_id,
            );

            continue;
        };

        if registrar.source != EventSource::Memory {
            tracing::warn!(
                target: "sword.events",
                source = ?registrar.source,
                "Event handler source is not supported yet, skipping handler type {:?}",
                type_id,
            );

            continue;
        }

        (registrar.build)(state);

        let Some(routes) = route_map.get(type_id) else {
            tracing::warn!(
                target: "sword.events",
                "No event routes registered for handler type {:?}",
                type_id,
            );

            continue;
        };

        for route in routes {
            let handle_fn = (route.build_and_handle)(state);
            handlers.entry(route.event_key).or_default().push(handle_fn);
        }
    }

    handlers
}
