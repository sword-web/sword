#[cfg(not(any(feature = "memory", feature = "nats")))]
compile_error!("sword-events requires at least one transport feature: `memory` or `nats`");

mod application;
mod config;
mod dispatch;
mod event;
mod registrar;

#[cfg(feature = "memory")]
mod memory;

#[cfg(feature = "nats")]
mod nats;

pub mod prelude;

pub use application::EventApplication;
pub use config::EventApplicationConfig;
pub use event::*;

#[doc(hidden)]
pub use registrar::*;

#[cfg(feature = "memory")]
pub use config::InMemoryEventsConfig;

#[cfg(feature = "nats")]
pub use config::NatsEventsConfig;

#[cfg(feature = "memory")]
pub use memory::MemoryEventPublisher;

#[cfg(feature = "nats")]
pub use nats::NatsEventPublisher;

#[doc(hidden)]
pub mod internal {
    pub use crate::registrar::*;
    pub use crate::{Event, EventApplicationConfig, EventCodecError};

    pub use serde_json;
}
