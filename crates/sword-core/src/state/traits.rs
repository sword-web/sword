use crate::{DependencyInjectionError, State};
use std::sync::Arc;

pub trait FromState: Sized {
    fn from_state(state: &State) -> Result<Self, DependencyInjectionError>;
}

pub trait FromStateArc: Sized {
    fn from_state_arc(state: &State) -> Result<Self, DependencyInjectionError>;
}

// Implement FromState for T (uses .get() which clones the value)
impl<T> FromState for T
where
    T: Clone + Send + Sync + 'static,
{
    fn from_state(state: &State) -> Result<Self, DependencyInjectionError> {
        state.get::<T>()
    }
}

// Implement FromStateArc for Arc<T> (uses .borrow() which returns Arc)
impl<T> FromStateArc for Arc<T>
where
    T: Send + Sync + 'static,
{
    fn from_state_arc(state: &State) -> Result<Self, DependencyInjectionError> {
        state.borrow::<T>()
    }
}
