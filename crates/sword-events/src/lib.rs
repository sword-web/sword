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
pub use event::{Event, EventCodecError, EventPublish, EventPublishError, EventSource};
pub use registrar::EventHandlerResult;

#[cfg(feature = "memory")]
pub use config::InMemoryEventsConfig;

#[cfg(feature = "nats")]
pub use config::NatsEventsConfig;

#[cfg(feature = "memory")]
pub use memory::MemoryEventPublisher;

#[cfg(feature = "nats")]
pub use nats::NatsEventPublisher;

/// Items required by generated code.
///
/// This module is public so procedural macros can name these items from the
/// user's crate, but it is not part of the stable API.
#[doc(hidden)]
pub mod internal {
    pub use crate::registrar::{
        EventControllerRegistrar, EventDecoder, EventHandlerFn, EventHandlerRegistrar,
        EventRegistrar,
    };

    pub use serde_json;
}
