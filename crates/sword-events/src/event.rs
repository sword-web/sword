use std::{any::Any, fmt::Debug};
use thiserror::Error;

/// Event source enum used by `#[controller(kind = Controller::EventHandler, ...)]`
/// attributes and runtime internals to select which event backend a handler consumes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventSource {
    Memory,
    Nats,
}

/// Event publishing trait implemented by any event transport.
pub trait EventPublish: Send + Sync {
    fn publish(
        &self,
        event: impl Event + 'static,
    ) -> impl Future<Output = Result<(), EventPublishError>> + Send;
}

pub trait Event: Send + Sync + Any {
    fn key(&self) -> &'static str;
    fn to_json(&self) -> Result<Vec<u8>, EventCodecError>;
}

impl dyn Event {
    pub fn downcast_ref<T: Event>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref::<T>()
    }
}

#[derive(Debug, Error)]
pub enum EventPublishError {
    #[error("the event channel is closed")]
    Closed,

    #[error("event transport error: {0}")]
    Transport(Box<dyn std::error::Error + Send + Sync>),

    #[error("the NATS transport is not connected yet")]
    NotConnected,

    #[error("failed to encode/decode event: {0}")]
    Codec(#[from] EventCodecError),
}

#[derive(Debug, thiserror::Error)]
pub enum EventCodecError {
    #[error("failed to encode/decode event: {0}")]
    Json(#[from] serde_json::Error),
}
