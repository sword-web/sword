#[cfg(feature = "memory")]
use crate::memory::MemoryTransport;

#[cfg(feature = "nats")]
use crate::nats::NatsTransport;

use crate::{
    EventApplicationConfig, EventControllerRegistrar, EventHandlerFn, EventHandlerRegistrar,
    EventSource,
};

use std::{any::TypeId, collections::HashMap};
use sword_core::{Config, Controller, ControllerRegistry, ProviderRegistry, Shutdown, State};
use tokio::task::JoinSet;

/// Background engine that consumes published events and dispatches them to the
/// registered handlers.
pub struct EventApplication {
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
        {
            let (publisher, transport) = NatsTransport::new(&events_config);

            providers.register(publisher);
            transports.push(EventTransport::Nats(transport));
        }

        Self { transports }
    }

    pub fn with_controllers(&mut self, controllers: &ControllerRegistry, state: &State) {
        let mut subscriptions = EventSubscriptions::collect(state, controllers);

        for transport in &mut self.transports {
            transport.set_handlers(subscriptions.take(transport.source()));
        }
    }

    pub async fn run(self, shutdown: Shutdown) {
        let mut tasks = JoinSet::new();

        for transport in self.transports {
            let shutdown = shutdown.clone();
            tasks.spawn(async move { transport.run(shutdown).await });
        }

        shutdown.cancelled().await;

        while tasks.join_next().await.is_some() {}
    }
}

/// Possible event transports (backends) that can be used to publish and consume events.
enum EventTransport {
    #[cfg(feature = "memory")]
    Memory(MemoryTransport),

    #[cfg(feature = "nats")]
    Nats(NatsTransport),
}

impl EventTransport {
    fn source(&self) -> EventSource {
        match self {
            #[cfg(feature = "memory")]
            EventTransport::Memory(_) => EventSource::Memory,

            #[cfg(feature = "nats")]
            EventTransport::Nats(_) => EventSource::Nats,
        }
    }

    fn set_handlers(&mut self, handlers: Vec<HandlerEntry>) {
        match self {
            #[cfg(feature = "memory")]
            EventTransport::Memory(transport) => transport.set_handlers(handlers),

            #[cfg(feature = "nats")]
            EventTransport::Nats(transport) => transport.set_handlers(handlers),
        }
    }

    async fn run(self, shutdown: Shutdown) {
        match self {
            #[cfg(feature = "memory")]
            EventTransport::Memory(transport) => {
                transport.run(shutdown).await;
            }

            #[cfg(feature = "nats")]
            EventTransport::Nats(transport) => {
                transport.run(shutdown).await;
            }
        }
    }
}

pub struct HandlerEntry {
    pub key: &'static str,
    #[cfg_attr(not(feature = "nats"), allow(dead_code))]
    pub group: Option<&'static str>,
    pub handler: EventHandlerFn,
}

/// All the event handlers registered in the application,
/// grouped by event source and event key.
struct EventSubscriptions {
    by_source: HashMap<EventSource, Vec<HandlerEntry>>,
}

impl EventSubscriptions {
    /// Builds all the registered event controllers and its handlers,
    /// grouping them by their source kind
    fn collect(state: &State, controllers: &ControllerRegistry) -> Self {
        let mut by_source: HashMap<EventSource, Vec<HandlerEntry>> = HashMap::new();

        for type_id in controllers.get_by_kind(Controller::Events) {
            let Some(controller) = controller_registrar(&type_id) else {
                tracing::warn!(target: "sword.events", "No EventControllerRegistrar for {type_id:?}");
                continue;
            };

            (controller.build)(state); // Build the controller and register it in the state

            // iterate over all the handlers registered for this controller type

            for handler in handler_registrars(&type_id) {
                by_source
                    .entry(handler.source)
                    .or_default()
                    .push(HandlerEntry {
                        key: handler.key,
                        group: handler.group,
                        handler: (handler.handler_fn)(state),
                    });
            }
        }

        Self { by_source }
    }

    /// Returns the map of event handlers for a specific source, removing it from the subscriptions.
    fn take(&mut self, source: EventSource) -> Vec<HandlerEntry> {
        self.by_source.remove(&source).unwrap_or_default()
    }
}

fn controller_registrar(type_id: &TypeId) -> Option<&'static EventControllerRegistrar> {
    inventory::iter::<EventControllerRegistrar>().find(|controller| controller.ty == *type_id)
}

fn handler_registrars(type_id: &TypeId) -> impl Iterator<Item = &'static EventHandlerRegistrar> {
    inventory::iter::<EventHandlerRegistrar>()
        .filter(move |handler| handler.controller_ty == *type_id)
}
