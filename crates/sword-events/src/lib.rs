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

use sword_core::{Config, ConfigFailure, ConfigValidateExt};

pub use self::application::EventApplication;
pub use self::config::EventApplicationConfig;

#[cfg(feature = "memory")]
pub use self::config::InMemoryEventsConfig;

#[cfg(feature = "nats")]
pub use self::config::{NatsAuthConfig, NatsEventsConfig, NatsTlsConfig};

pub use self::event::{Event, EventCodecError, EventPublish, EventPublishError, EventSource};

#[cfg(feature = "memory")]
pub use self::memory::MemoryEventPublisher;

#[cfg(feature = "nats")]
pub use self::nats::NatsEventPublisher;

pub use self::registrar::EventHandlerResult;

/// Checks the `[events]` section before the application is built.
pub fn validate_config(config: &Config) -> Result<(), ConfigFailure> {
    config.get_validated_section::<EventApplicationConfig>()?;
    Ok(())
}

/// Items required by generated code.
///
/// This module is public so procedural macros can name these items from the
/// user's crate, but it is not part of the stable API.
#[doc(hidden)]
pub mod internal {
    pub use serde_json;

    pub use crate::registrar::{
        EventControllerRegistrar, EventDecoder, EventHandlerFn, EventHandlerRegistrar,
        EventRegistrar,
    };
}
