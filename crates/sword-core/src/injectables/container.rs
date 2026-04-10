use crate::{ComponentRegistry, DependencyInjectionError as DIError, ProviderRegistry, State};

use std::{any::TypeId, collections::HashSet, sync::Arc};

/// A container for managing dependencies and their builders.
///
/// It support two types of registrations:
///
/// **Providers**:
///
/// Providers are pre-created objects that you want to register directly into the container.
/// For example, you might have a database connection or external service client that you
/// need to build beforehand and inject into other Dependencies.
///
/// **Components**
///
/// Are types that has no need to be pre-created. Instead, you register the type itself,
/// and the container will use the `Component` trait to build them when needed, resolving
/// their dependencies automatically.
pub struct DependencyContainer {
    providers: ProviderRegistry,
    components: ComponentRegistry,
}

impl DependencyContainer {
    pub fn new() -> Self {
        Self {
            providers: ProviderRegistry::new(),
            components: ComponentRegistry::new(),
        }
    }

    pub fn provider_registry(&self) -> &ProviderRegistry {
        &self.providers
    }

    pub fn component_registry(&self) -> &ComponentRegistry {
        &self.components
    }

    /// Builds all registered components in dependency order.
    ///
    /// This internal method performs the following steps:
    /// 1. Registers all provider instances in the State
    /// 2. Performs topological sorting on the dependency graph
    /// 3. Constructs components recursively in the correct order
    /// 4. Detects circular dependencies and returns an error if found
    ///
    /// This method is called internally during application initialization.
    pub fn build_all(&self, state: &State) -> Result<(), DIError> {
        let mut built = HashSet::new();
        let mut visiting = HashSet::new();

        // First. register all the provided instances

        let providers = &self.providers.get_providers();

        for (type_id, instance) in providers.read().iter() {
            state.insert_instance(*type_id, Arc::clone(instance));
            built.insert(*type_id);
        }

        // Then, build the rest based on dependencies in topological order.
        // If a type_id is already built, skip it (Dep already built).

        for type_id in self.components.get_dependency_graph().read().keys() {
            self.build_recursive(type_id, state, &mut built, &mut visiting)?;
        }

        Ok(())
    }

    /// Recursively builds a component and its dependencies.
    ///
    /// This method implements depth-first traversal of the dependency graph:
    /// - Skips already built components
    /// - Detects circular dependencies using a visiting set
    /// - Recursively builds all dependencies before the component itself
    /// - Invokes the builder function and stores the result in State
    fn build_recursive(
        &self,
        type_id: &TypeId,
        state: &State,
        built: &mut HashSet<TypeId>,
        visiting: &mut HashSet<TypeId>,
    ) -> Result<(), DIError> {
        if built.contains(type_id) {
            return Ok(());
        }

        if visiting.contains(type_id) {
            return Err(DIError::CircularDependency);
        }

        visiting.insert(*type_id);

        // Explore to all the dependencies first
        // and for each dependency, invoke build_recursive
        // to ensure they are built before building the current type.

        let dependency_graph = &self.components.get_dependency_graph();

        if let Some(deps) = dependency_graph.read().get(type_id) {
            for dep_id in deps {
                self.build_recursive(dep_id, state, built, visiting)?;
            }
        }

        visiting.remove(type_id);

        if let Some(builder) = &self.components.get_builders().read().get(type_id) {
            state.insert_instance(*type_id, builder(state)?);
            built.insert(*type_id);
        }

        Ok(())
    }
}

impl Default for DependencyContainer {
    fn default() -> Self {
        Self::new()
    }
}
