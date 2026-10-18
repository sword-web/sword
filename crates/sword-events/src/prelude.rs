pub use crate::{Event, EventApplicationConfig, EventHandlerResult};

#[cfg(feature = "memory")]
pub use crate::InMemoryEventsConfig;

#[cfg(feature = "nats")]
pub use crate::NatsEventsConfig;

pub use sword_core::EventSource;
pub use sword_macros::{event, subscribe};
