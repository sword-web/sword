use crate::{
    Event, EventControllerRegistrar, EventQueueConfig, EventRouteRegistrar,
    in_memory::{EventKeyHandlerMap, EventPublisher, EventSubscriber},
};

use std::{any::TypeId, collections::HashMap, sync::Arc};
use sword_core::{Config, Controller, ControllerRegistry, EventSource, State};
use tokio::sync::mpsc::{self, Receiver};

pub struct EventApplicationRuntime {
    state: State,
    config: EventQueueConfig,
    rx: Receiver<Arc<dyn Event>>,
}

impl EventApplicationRuntime {
    pub fn new(state: &State, config: &Config) -> Self {
        let queue_config = config.get_or_default::<EventQueueConfig>();

        let (tx, rx) = mpsc::channel::<Arc<dyn Event>>(queue_config.buffer_size);

        state.insert(EventPublisher::new(tx));

        Self {
            rx,
            state: state.clone(),
            config: queue_config,
        }
    }

    pub fn start(self, controllers: &ControllerRegistry) {
        let handlers = self.build_handlers(controllers);

        if handlers.is_empty() {
            return;
        }

        let subscriber = EventSubscriber {
            handlers,
            receiver: self.rx,
            config: self.config,
        };

        subscriber.run();
    }

    fn build_handlers(&self, controllers: &ControllerRegistry) -> EventKeyHandlerMap {
        let event_controllers = controllers.get_by_kind(Controller::EventHandler);

        if event_controllers.is_empty() {
            return HashMap::new();
        }

        let controller_registrars: HashMap<TypeId, &EventControllerRegistrar> =
            inventory::iter::<EventControllerRegistrar>()
                .map(|r| (r.handler_type_id, r))
                .collect();

        let mut route_map: HashMap<TypeId, Vec<&EventRouteRegistrar>> = HashMap::new();

        for route in inventory::iter::<EventRouteRegistrar>() {
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

            (registrar.build)(&self.state);

            let Some(routes) = route_map.get(type_id) else {
                tracing::warn!(
                    target: "sword.events",
                    "No event routes registered for handler type {:?}",
                    type_id,
                );

                continue;
            };

            for route in routes {
                let handle_fn = (route.build_and_handle)(&self.state);
                handlers.entry(route.event_key).or_default().push(handle_fn);
            }
        }

        handlers
    }
}
