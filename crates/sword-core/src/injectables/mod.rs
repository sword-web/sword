mod components;
mod container;
mod error;
mod providers;

use crate::State;

use std::{
    any::{Any, TypeId},
    sync::Arc,
};

pub use components::{Component, ComponentRegistry};
pub use container::DependencyContainer;
pub use error::DependencyInjectionError;
pub use providers::{Provider, ProviderRegistry};

/// Base trait for any component that can be constructed from the application State.
pub trait Build: Clone + Send + Sync + 'static {
    fn build(state: &State) -> Result<Self, DependencyInjectionError>
    where
        Self: Sized;
}

/// Trait for components that have dependencies on other components.
///
/// The `deps()` method returns a list of `TypeId`s of the dependencies
/// required to build the component.
pub trait HasDeps: Build {
    fn deps() -> Vec<TypeId> {
        Vec::new()
    }
}

/// Pointer to dyn Any element. It retrieves dynamic capabilites
/// to the dependency container. Basically represents Any element.
pub type Injectable = Arc<dyn Any + Send + Sync>;
