use std::any::TypeId;
use std::collections::{HashMap, HashSet};

use sword_core::*;
use tokio::task::JoinSet;

use crate::EventTransport;
use crate::config::EventApplicationConfig;

use crate::registrar::{
    EventControllerRegistrar, EventHandlerFn, EventHandlerRegistrar, EventRegistrar,
};

#[cfg(feature = "memory")]
use crate::memory::MemoryTransport;

#[cfg(feature = "nats")]
use crate::nats::NatsTransport;

/// Background engine that consumes published events and dispatches them to the
/// registered handlers.
pub struct EventApplication {
    transports: Vec<TransportRunner>,
}

impl EventApplication {
    pub fn new(config: &Config, providers: &ProviderRegistry) -> Result<Self, Error> {
        check_event_key_uniqueness()?;

        let events_config = config.get_or_default::<EventApplicationConfig>();

        if events_config.concurrency == 0 {
            return Err(sword_error! {
                code: "sword.build.events",
                phase: DiagnosticPhase::Build,
                target: "events",
                title: "Invalid events concurrency",
                reason: "`concurrency` must be greater than 0",
                source: "EventApplication::new",
                hints: [
                    "Set a positive value in the [events] config section",
                    "The default is 256",
                ],
            });
        }

        let mut transports = Vec::new();

        #[cfg(feature = "memory")]
        {
            let (publisher, transport) = MemoryTransport::new(&events_config)?;

            providers.register(publisher);
            transports.push(TransportRunner::Memory(transport));
        }

        #[cfg(feature = "nats")]
        {
            let nats_configured = events_config.nats.is_some();
            let nats_config = events_config.nats.clone().unwrap_or_default();

            let (publisher, transport) = NatsTransport::new(
                nats_config,
                events_config.handler_timeout_ms,
                events_config.concurrency,
            );

            providers.register(publisher);

            if nats_configured {
                transports.push(TransportRunner::Nats(Box::new(transport)));
            }
        }

        Ok(Self { transports })
    }

    /// Registers the event handlers for all the controllers in the application,
    /// grouping them by their transport and event key.
    pub fn with_controllers(
        &mut self,
        controllers: &ControllerRegistry,
        state: &State,
    ) -> Result<(), Error> {
        let mut subscriptions = EventSubscriptions::collect(state, controllers)?;

        // Set the handlers for each transport
        //
        // MemoryTransport will only receive handlers for EventTransport::Memory
        // NatsTransport will only receive handlers for EventTransport::Nats

        for transport in &mut self.transports {
            transport.set_handlers(subscriptions.take(transport.transport()));
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

        let result = tokio::select! {
            _ = shutdown.cancelled() => {
                tasks.abort_all();
                return Ok(());
            }
            result = tasks.join_next() => result,
        };

        tasks.abort_all();

        match result {
            Some(Ok(Ok(()))) => Err(sword_error! {
                code: "sword.runtime.events",
                phase: DiagnosticPhase::Runtime,
                target: "EventApplication::run",
                title: "Event transport stopped before shutdown",
                reason: "a transport finished without a shutdown request",
                source: "EventApplication::run",
                hints: ["Check the event transport logs"],
            }),
            Some(Ok(Err(error))) => Err(error),
            Some(Err(join_error)) => Err(sword_error! {
                code: "sword.runtime.events",
                phase: DiagnosticPhase::Runtime,
                target: "EventApplication::run",
                title: "Event transport task panicked",
                reason: join_error,
                source: "EventApplication::run",
                hints: ["Check the panic message in the logs"],
                cause: join_error,
            }),
            None => Ok(()),
        }
    }
}

/// Possible event transports (backends) that can be used to publish and consume events.
enum TransportRunner {
    #[cfg(feature = "memory")]
    Memory(MemoryTransport),

    #[cfg(feature = "nats")]
    Nats(Box<NatsTransport>),
}

impl TransportRunner {
    fn transport(&self) -> EventTransport {
        match self {
            #[cfg(feature = "memory")]
            TransportRunner::Memory(_) => EventTransport::Memory,

            #[cfg(feature = "nats")]
            TransportRunner::Nats(_) => EventTransport::Nats,
        }
    }

    fn set_handlers(&mut self, handlers: Vec<HandlerEntry>) {
        match self {
            #[cfg(feature = "memory")]
            TransportRunner::Memory(transport) => transport.set_handlers(handlers),

            #[cfg(feature = "nats")]
            TransportRunner::Nats(transport) => transport.set_handlers(handlers),
        }
    }

    async fn run(self, shutdown: Shutdown) -> Result<(), Error> {
        #[allow(unreachable_patterns)]
        match self {
            #[cfg(feature = "memory")]
            TransportRunner::Memory(transport) => {
                transport.run(shutdown).await;
            }

            #[cfg(feature = "nats")]
            TransportRunner::Nats(transport) => {
                transport.run(shutdown).await?;
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
/// grouped by event transport and event key.
struct EventSubscriptions {
    by_transport: HashMap<EventTransport, Vec<HandlerEntry>>,
}

impl EventSubscriptions {
    /// Builds all the registered event controllers and its handlers,
    /// grouping them by their transport
    fn collect(state: &State, controllers: &ControllerRegistry) -> Result<Self, Error> {
        let mut events_by_transport: HashMap<EventTransport, Vec<HandlerEntry>> = HashMap::new();

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

                events_by_transport
                    .entry(handler.transport)
                    .or_default()
                    .push(handler_entry);
            }
        }

        Ok(Self {
            by_transport: events_by_transport,
        })
    }

    /// Returns the map of event handlers for a specific transport, removing it from the subscriptions.
    fn take(&mut self, transport: EventTransport) -> Vec<HandlerEntry> {
        self.by_transport.remove(&transport).unwrap_or_default()
    }
}

fn controller_registrar(type_id: &TypeId) -> Option<&'static EventControllerRegistrar> {
    inventory::iter::<EventControllerRegistrar>().find(|controller| controller.ty == *type_id)
}

fn check_event_key_uniqueness() -> Result<(), Error> {
    let mut keys = HashSet::new();

    for event in inventory::iter::<EventRegistrar>() {
        if !keys.insert(event.key) {
            return Err(sword_error! {
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
