use crate::{DependencyInjectionError, HasDeps, Injectable, RwMap, State};
use std::{
    any::{TypeId, type_name},
    collections::HashMap,
    sync::Arc,
};

type ComponentBuilderFn = Box<dyn Fn(&State) -> Result<Injectable, DependencyInjectionError>>;

/// Trait for injectable components that can be automatically constructed
/// by the dependency container with automatic dependency resolution.
///
/// Components are services or dependencies that need to be built from other
/// components in the State.
///
/// Use the `#[injectable]` macro to automatically implement this trait.
pub trait Component: HasDeps {}

pub struct ComponentRegistry {
    builders: RwMap<TypeId, ComponentBuilderFn>,
    dependency_graph: RwMap<TypeId, Vec<TypeId>>,
}

impl ComponentRegistry {
    pub(crate) fn new() -> Self {
        Self {
            builders: RwMap::new(HashMap::new()),
            dependency_graph: RwMap::new(HashMap::new()),
        }
    }

    pub fn register<T: Component>(&self) {
        let type_id = TypeId::of::<T>();
        let type_name = type_name::<T>();

        let component_builder = Box::new(move |state: &State| {
            T::build(state)
                .map(|instance| Arc::new(instance) as Injectable)
                .map_err(|e| DependencyInjectionError::build_failed(type_name, e))
        });

        self.dependency_graph.write().insert(type_id, T::deps());
        self.builders.write().insert(type_id, component_builder);
    }

    pub(crate) fn get_builders(&self) -> &RwMap<TypeId, ComponentBuilderFn> {
        &self.builders
    }

    pub(crate) fn get_dependency_graph(&self) -> &RwMap<TypeId, Vec<TypeId>> {
        &self.dependency_graph
    }
}
