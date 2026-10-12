use crate::{
    Event, EventControllerRegistrar, EventQueueConfig, EventRouteRegistrar,
    in_memory::{EventKeyHandlerMap, EventPublisher, EventSubscriber},
};

use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;
use sword_core::{Config, Controller, ControllerRegistry, EventSource, Shutdown, State};
use tokio::sync::mpsc::{self, Receiver};

/// Creates the in-memory event channel.
///
/// The [`EventPublisher`] is a provider (registered before DI, so components can
/// depend on it); the [`EventSubscriberHandle`] becomes the events engine once
/// the registered controllers are known.
pub struct EventChannel;

impl EventChannel {
    pub fn create(config: &Config) -> (EventPublisher, EventSubscriberHandle) {
        let queue_config = config.get_or_default::<EventQueueConfig>();
        let (tx, rx) = mpsc::channel::<Arc<dyn Event>>(queue_config.buffer_size);

        (
            EventPublisher::new(tx),
            EventSubscriberHandle {
                rx,
                config: queue_config,
            },
        )
    }
}

/// Owns the receiving side of the channel. Turns into an [`EventEngine`] once
/// the registered controllers are known (post-DI).
pub struct EventSubscriberHandle {
    rx: Receiver<Arc<dyn Event>>,
    config: EventQueueConfig,
}

impl EventSubscriberHandle {
    /// Builds the handlers for the registered event controllers and returns the
    /// runnable engine.
    pub fn into_engine(self, state: &State, controllers: &ControllerRegistry) -> EventEngine {
        EventEngine {
            handlers: build_handlers(state, controllers),
            config: self.config,
            rx: self.rx,
        }
    }
}

/// Background engine that consumes published events and dispatches them to the
/// registered handlers.
pub struct EventEngine {
    handlers: EventKeyHandlerMap,
    config: EventQueueConfig,
    rx: Receiver<Arc<dyn Event>>,
}

impl EventEngine {
    /// Consumes events until `shutdown` is cancelled.
    pub async fn run(self, shutdown: Shutdown) {
        let EventEngine {
            handlers,
            config,
            rx,
        } = self;

        EventSubscriber {
            handlers,
            receiver: rx,
            config,
        }
        .run(shutdown)
        .await;
    }
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
