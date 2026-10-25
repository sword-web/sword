#[cfg(feature = "memory")]
use crate::memory::MemoryTransport;

#[cfg(feature = "nats")]
use crate::nats::NatsTransport;

use crate::{
    EventApplicationConfig, EventControllerRegistrar, EventHandlerFn, EventHandlerRegistrar,
    EventRegistrar, EventSource,
};

use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
};

use sword_core::diagnostic::{DiagnosticPhase, Error};
use sword_core::{
    Config, ConfigDocument, ConfigSection, Controller, ControllerRegistry, ProviderRegistry,
    Shutdown, State, sword_diagnostic,
};

use tokio::task::JoinSet;

/// Background engine that consumes published events and dispatches them to the
/// registered handlers.
pub struct EventApplication {
    transports: Vec<EventTransport>,
}

impl EventApplication {
    pub fn new(
        config: &Config,
        providers: &ProviderRegistry,
        document: Option<&ConfigDocument>,
    ) -> Result<Self, Error> {
        check_event_key_uniqueness()?;

        let events_config =
            ConfigSection::load::<EventApplicationConfig>(config, document)?.unwrap_or_default();
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

        Ok(Self { transports })
    }

    /// Registers the event handlers for all the controllers in the application,
    /// grouping them by their source kind and event key.
    pub fn with_controllers(
        &mut self,
        controllers: &ControllerRegistry,
        state: &State,
    ) -> Result<(), Error> {
        let mut subscriptions = EventSubscriptions::collect(state, controllers)?;

        // Set the handlers for each transport based on its source kind
        //
        // MemoryTransport will only receive handlers for EventSource::Memory
        // NatsTransport will only receive handlers for EventSource::Nats

        for transport in &mut self.transports {
            transport.set_handlers(subscriptions.take(transport.source()));
        }

        Ok(())
    }

    pub async fn run(self, shutdown: Shutdown) -> Result<(), Error> {
        let mut tasks = JoinSet::new();

        for transport in self.transports {
            let shutdown = shutdown.clone();
            tasks.spawn(async move { transport.run(shutdown).await });
        }

        if tasks.is_empty() {
            shutdown.cancelled().await;
            return Ok(());
        }

        let mut first_error = None;

        tokio::select! {
            _ = shutdown.cancelled() => {}
            result = tasks.join_next() => {
                if let Some(result) = result {
                    first_error = Self::task_error(result);
                }

                shutdown.cancel();
            }
        }

        while let Some(result) = tasks.join_next().await {
            if first_error.is_none() {
                first_error = Self::task_error(result);
            }
        }

        match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    /// A transport that panics reaches us as a `JoinError`. Without this it
    /// would look like a clean stop.
    fn task_error(result: Result<Result<(), Error>, tokio::task::JoinError>) -> Option<Error> {
        match result {
            Ok(Ok(())) => None,
            Ok(Err(error)) => Some(error),
            Err(error) => Some(sword_diagnostic! {
                code: "sword.runtime.events",
                phase: DiagnosticPhase::Runtime,
                target: "EventApplication::run",
                title: "Event transport task failed",
                reason: error,
                source: "EventApplication::run",
                hints: ["Check the event transport logs and the underlying cause"],
                cause: error,
            }),
        }
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

    async fn run(self, shutdown: Shutdown) -> Result<(), Error> {
        #[allow(unreachable_patterns)]
        match self {
            #[cfg(feature = "memory")]
            EventTransport::Memory(transport) => {
                transport.run(shutdown).await;
            }

            #[cfg(feature = "nats")]
            EventTransport::Nats(transport) => {
                transport.run(shutdown).await;
            }

            _ => {}
        }

        Ok(())
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
    fn collect(state: &State, controllers: &ControllerRegistry) -> Result<Self, Error> {
        let mut events_by_source: HashMap<EventSource, Vec<HandlerEntry>> = HashMap::new();

        for type_id in controllers.get_by_kind(Controller::Events) {
            let Some(controller) = controller_registrar(&type_id) else {
                tracing::warn!(target: "sword.events", "No EventControllerRegistrar for {type_id:?}");
                continue;
            };

            (controller.build)(state)?; // Build the controller and register into the state

            // iterate over all the handlers registered for this controller type

            for handler in inventory::iter::<EventHandlerRegistrar>()
                .filter(|handler| handler.controller_ty == type_id)
            {
                let handler_entry = HandlerEntry {
                    key: handler.key,
                    group: handler.group,
                    handler: (handler.handler_fn)(state)?,
                };

                events_by_source
                    .entry(handler.source)
                    .or_default()
                    .push(handler_entry);
            }
        }

        Ok(Self {
            by_source: events_by_source,
        })
    }

    /// Returns the map of event handlers for a specific source, removing it from the subscriptions.
    fn take(&mut self, source: EventSource) -> Vec<HandlerEntry> {
        self.by_source.remove(&source).unwrap_or_default()
    }
}

fn controller_registrar(type_id: &TypeId) -> Option<&'static EventControllerRegistrar> {
    inventory::iter::<EventControllerRegistrar>().find(|controller| controller.ty == *type_id)
}

fn check_event_key_uniqueness() -> Result<(), Error> {
    let mut keys = HashSet::new();

    for event in inventory::iter::<EventRegistrar>() {
        if !keys.insert(event.key) {
            return Err(sword_diagnostic! {
                code: "sword.build.events",
                phase: DiagnosticPhase::Build,
                target: "event registry",
                title: "Duplicate event key",
                reason: format!(
                    "event key `{}` is registered more than once",
                    event.key,
                ),
                source: "check_event_key_uniqueness",
                context: {
                    "key" => event.key,
                },
                hints: [
                    "Each `#[event(key = ...)]` must use a unique key",
                ],
            });
        }
    }

    Ok(())
}
