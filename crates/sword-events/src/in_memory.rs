mod publisher;
mod subscriber;

use crate::EventHandlerFn;
use std::collections::HashMap;

pub use publisher::EventPublisher;
pub(crate) use subscriber::EventSubscriber;

pub(crate) type EventKeyHandlerMap = HashMap<&'static str, Vec<EventHandlerFn>>;
