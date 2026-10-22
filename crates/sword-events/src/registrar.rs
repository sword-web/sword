use crate::{Event, EventCodecError, EventSource};

use std::any::TypeId;
#[cfg(feature = "memory")]
use std::collections::HashMap;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use sword_core::State;

pub type EventHandlerResult<T> = Result<T, Box<dyn Error + Send + Sync + 'static>>;
type BoxedFuture<T> = Pin<Box<dyn Future<Output = EventHandlerResult<T>> + Send>>;

pub type EventHandlerFn = Arc<dyn Fn(Arc<dyn Event>) -> BoxedFuture<()> + Send + Sync>;

#[cfg(feature = "memory")]
pub(crate) type EventKeyHandlerMap = HashMap<&'static str, Vec<EventHandlerFn>>;

/// Builds an Events controller from DI state once
pub struct EventControllerRegistrar {
    pub ty: TypeId,
    pub build: fn(&State),
}

/// Registers a single event handler for an Events controller.
pub struct EventHandlerRegistrar {
    /// The event key that this handler is subscribed to. AKA the subject in NATS.
    pub key: &'static str,

    /// This is used to determine which event transport (event source) to use.
    pub source: EventSource,

    /// The group that this handler belongs to. This is used to determine which NATS queue group to subscribe to.
    pub group: Option<&'static str>,

    /// The type of the controller that this handler belongs to. This is used to find the controller in the DI state.
    pub controller_ty: TypeId,

    /// The function that will be called to get the event handler function.
    /// This is used to get the handler function from the controller instance.
    pub handler_fn: fn(&State) -> EventHandlerFn,
}

pub type EventDecoder = fn(&[u8]) -> Result<Box<dyn Event>, EventCodecError>;

pub struct EventRegistrar {
    pub key: &'static str,
    pub from_json: EventDecoder,
}

inventory::collect!(EventControllerRegistrar);
inventory::collect!(EventHandlerRegistrar);
inventory::collect!(EventRegistrar);
