#[cfg(any(feature = "memory", feature = "nats"))]
mod application;
mod config;
mod registrar;

#[cfg(feature = "memory")]
pub mod memory;

pub mod prelude;

use std::{any::Any, fmt::Debug};

#[cfg(any(feature = "memory", feature = "nats"))]
pub use application::{EventApplication, EventTransport};
pub use config::EventApplicationConfig;
pub use registrar::*;

#[cfg(feature = "memory")]
pub use config::InMemoryEventsConfig;

#[cfg(feature = "nats")]
pub use config::NatsEventsConfig;

/// Event source enum used by `#[controller(kind = Controller::EventHandler, ...)]`
/// attributes and runtime internals to select which event backend a handler consumes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventSource {
    Memory,
    Nats,
}

pub trait Event: Send + Sync + Any + Debug {
    fn key(&self) -> &'static str;
}

impl dyn Event {
    pub fn downcast_ref<T: Event>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref::<T>()
    }
}

#[doc(hidden)]
pub mod internal {
    pub use crate::{Event, EventApplicationConfig};
    pub use crate::registrar::*;
}
