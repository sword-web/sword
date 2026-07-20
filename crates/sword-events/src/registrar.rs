use std::any::{Any, TypeId};
use std::error::Error;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use sword_core::State;

pub type EventHandlerResult<T> = Result<T, Box<dyn Error + Send + Sync + 'static>>;
type BoxedFuture<T> = Pin<Box<dyn Future<Output = EventHandlerResult<T>> + Send>>;

pub type EventHandlerFn = Arc<dyn Fn(Arc<dyn Event>) -> BoxedFuture<()> + Send + Sync>;

/// Builds a MemEventHandler controller from DI state once and returns
/// a reusable closure that invokes a specific handler method.
pub struct MemEventControllerRegistrar {
    pub handler_type_id: TypeId,
    pub build: fn(&State),
}

/// Registers a single event route for a MemEventHandler controller.
/// `build_and_handle` constructs the controller from state once and
/// returns a closure that can be called for each incoming event.
pub struct MemEventRouteRegistrar {
    pub event_key: &'static str,
    pub handler_type_id: TypeId,
    pub build_and_handle: fn(&State) -> EventHandlerFn,
}

pub trait Event: Send + Sync + Any + Debug {
    fn key(&self) -> &'static str;
    fn clone_event(&self) -> Box<dyn Event>;
}

impl Clone for Box<dyn Event> {
    fn clone(&self) -> Self {
        self.clone_event()
    }
}

impl dyn Event {
    pub fn downcast_ref<T: Event>(&self) -> Option<&T> {
        (self as &dyn Any).downcast_ref::<T>()
    }
}

pub trait MemEventHandler: sword_core::HasDeps + sword_core::Build {
    fn namespace() -> &'static str;
}

inventory::collect!(MemEventControllerRegistrar);
inventory::collect!(MemEventRouteRegistrar);
