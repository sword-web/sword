mod traits;

use std::{
    any::{Any, TypeId, type_name},
    collections::HashMap,
    sync::Arc,
};

use crate::DependencyInjectionError;
use parking_lot::RwLock;

pub use traits::*;

/// Application state container for type-safe dependency injection and data sharing.
///
/// `State` provides a thread-safe way to store and retrieve shared data across
/// the entire application. It uses `TypeId` as keys to ensure type safety.
#[derive(Clone, Debug)]
pub struct State {
    inner: Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
}

impl State {
    /// Creates an empty shared state container.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Extract a clone of the stored value of type `T` from the state.
    ///
    /// # Errors
    ///
    /// Returns an error if no value of type `T` has been registered in the
    /// state. This usually indicates that the dependency was never inserted or
    /// was expected to be provided by a module/provider that was not registered.
    pub fn get<T>(&self) -> Result<T, DependencyInjectionError>
    where
        T: Clone + Send + Sync + 'static,
    {
        let map = self.inner.read();
        let type_name = type_name::<T>().to_string();

        let state_ref = map
            .get(&TypeId::of::<T>())
            .ok_or_else(|| DependencyInjectionError::dependency_not_found(type_name.clone()))?;

        state_ref
            .downcast_ref::<T>()
            .cloned()
            .ok_or_else(|| DependencyInjectionError::dependency_not_found(type_name))
    }

    /// Borrow an `Arc` to the stored value of type `T` from the state.
    /// This returns an `Arc<T>` without cloning the underlying value.
    ///
    /// # Errors
    ///
    /// Returns an error if no value of type `T` has been registered in the
    /// state or if the stored value cannot be downcast back to `T`.
    pub fn borrow<T>(&self) -> Result<Arc<T>, DependencyInjectionError>
    where
        T: Send + Sync + 'static,
    {
        let map = self.inner.read();
        let type_name = type_name::<T>().to_string();

        let state_ref = map
            .get(&TypeId::of::<T>())
            .ok_or_else(|| DependencyInjectionError::dependency_not_found(type_name.clone()))?;

        state_ref
            .clone()
            .downcast::<T>()
            .map_err(|_| DependencyInjectionError::dependency_not_found(type_name))
    }

    pub fn insert<T: Send + Sync + 'static>(&self, state: T) {
        self.inner
            .write()
            .insert(TypeId::of::<T>(), Arc::new(state));
    }

    pub fn insert_instance(&self, type_id: TypeId, instance: Arc<dyn Any + Send + Sync>) {
        self.inner.write().insert(type_id, instance);
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
