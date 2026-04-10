use crate::{FromState, Injectable, RwMap};
use std::{any::TypeId, collections::HashMap, sync::Arc};

/// Marker trait for pre-instantiated dependencies (providers).
///
/// Providers are dependencies that have been pre-constructed and registered
/// into the State. Unlike Components which are built from their dependencies,
/// Providers are already complete instances that only need to be retrieved
/// from the State via the FromState trait.
///
/// Common use cases: database connections, external API clients, or any
/// resource that requires async initialization or complex setup.
pub trait Provider: FromState + Send + Sync {}

pub struct ProviderRegistry {
    providers: RwMap<TypeId, Injectable>,
}

impl ProviderRegistry {
    pub(crate) fn new() -> Self {
        Self {
            providers: RwMap::new(HashMap::new()),
        }
    }

    /// Registers a provider instance.
    ///
    /// Providers are instances that have already been constructed and are ready
    /// to be injected into other components. Typical use cases include database
    /// connections, HTTP clients, or external service configurations that cannot
    /// be auto-constructed from the State.
    pub fn register<T>(&self, provider: T)
    where
        T: Provider + 'static,
    {
        self.providers
            .write()
            .insert(TypeId::of::<T>(), Arc::new(provider));
    }

    pub(crate) fn get_providers(&self) -> &RwMap<TypeId, Injectable> {
        &self.providers
    }
}
