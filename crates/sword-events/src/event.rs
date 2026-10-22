use std::{any::Any, fmt::Debug};
use thiserror::Error;

/// Event source enum used by `#[controller(kind = Controller::EventHandler, ...)]`
/// attributes and runtime internals to select which event backend a handler consumes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventSource {
    #[cfg(feature = "memory")]
    Memory,

    #[cfg(feature = "nats")]
    Nats,
}

/// Publishes events to a transport.
pub trait EventPublish: Send + Sync {
    /// Publishes `event` to the configured transport.
    ///
    /// Returns an error if the transport is not connected or fails to
    /// serialize or send the event.
    fn publish(
        &self,
        event: impl Event + 'static,
    ) -> impl Future<Output = Result<(), EventPublishError>> + Send;
}

/// A typed event that can be published and consumed by the events subsystem.
pub trait Event: Send + Sync + Any {
    /// Returns the event key (subject) that identifies this event.
    fn key(&self) -> &'static str;

    /// Serializes the event to JSON bytes.
    fn to_vec(&self) -> Result<Vec<u8>, EventCodecError>;
}

impl dyn Event {
    /// Attempts to downcast this event to a concrete type.
    ///
    /// Returns `Some` when the underlying event is of type `T`, or `None`
    /// otherwise. Useful in wildcard handlers to recover the concrete type.
    pub fn cast<T: Event>(&self) -> Option<&T> {
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
