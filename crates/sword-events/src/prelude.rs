pub use crate::{Event, EventHandler, EventHandlerResult, EventQueueConfig};

#[cfg(feature = "in-memory")]
pub use crate::in_memory::EventPublisher;
