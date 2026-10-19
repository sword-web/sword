use crate::{Event, EventSource};

use std::any::TypeId;
use std::collections::HashMap;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use sword_core::State;

pub type EventHandlerResult<T> = Result<T, Box<dyn Error + Send + Sync + 'static>>;
type BoxedFuture<T> = Pin<Box<dyn Future<Output = EventHandlerResult<T>> + Send>>;

pub type EventHandlerFn = Arc<dyn Fn(Arc<dyn Event>) -> BoxedFuture<()> + Send + Sync>;
pub(crate) type EventKeyHandlerMap = HashMap<&'static str, Vec<EventHandlerFn>>;

/// Builds an EventHandler controller from DI state once and returns
/// a reusable closure that invokes a specific handler method.
pub struct EventControllerRegistrar {
    pub ty: TypeId,
    pub source: EventSource,
    pub build: fn(&State),
}

/// Registers a single event route for an EventHandler controller.
/// `build_and_handle` constructs the controller from state once and
/// returns a closure that can be called for each incoming event.
pub struct EventHandlerRegistrar {
    pub event_key: &'static str,
    pub controller_ty: TypeId,
    pub handler_fn: fn(&State) -> EventHandlerFn,
}

inventory::collect!(EventControllerRegistrar);
inventory::collect!(EventHandlerRegistrar);
