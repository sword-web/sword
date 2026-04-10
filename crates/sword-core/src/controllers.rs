use crate::HasDeps;
use parking_lot::{RawRwLock, RwLock, lock_api::RwLockReadGuard};
use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
};

pub type ControllerMap = HashMap<Controller, HashSet<TypeId>>;
pub type ControllerIds = HashSet<TypeId>;

/// Controller enum used by `#[controller(...)]` attributes and runtime internals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Controller {
    Web,
    SocketIo,
    Grpc,
}

/// A trait for defining controllers in the application.
///
/// Controllers represent different entry points into your application.
/// automatically implement this trait, allowing them to be registered as HTTP controllers
/// within modules.
///
/// # Example
///
/// ```rust,ignore
/// use sword::prelude::*;
///
/// #[controller(kind = Controller::Web, path = "/api/items")]
/// struct ItemsController { /* ... */ }
///
/// // The macro automatically implements ControllerSpec for ItemsController
/// // In your module:
/// fn register_controllers(controllers: &ControllerRegistry) {
///     controllers.register::<ItemsController>();
/// }
/// ```
pub trait ControllerSpec: HasDeps {
    fn kind() -> Controller;
    fn type_id() -> TypeId;
}

/// Registry for managing and storing different controller kinds.
///
/// `ControllerRegistry` is used within modules to register controllers that define how requests
/// enter the application.
///
/// # Example
///
/// ```rust,ignore
/// use sword::prelude::*;
///
/// struct MyModule;
///
/// impl Module for MyModule {
///     fn register_controllers(controllers: &ControllerRegistry) {
///         controllers.register::<UserController>();
///         controllers.register::<ProductController>();
///     }
/// }
/// ```
pub struct ControllerRegistry {
    controllers: RwLock<HashMap<Controller, HashSet<TypeId>>>,
}

impl ControllerRegistry {
    pub fn new() -> Self {
        Self {
            controllers: RwLock::new(HashMap::new()),
        }
    }

    /// Registers a controller of type `C` by calling its `kind()` method
    /// and storing the resulting `Controller` in the registry.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// controllers.register::<MyController>();
    /// ```
    pub fn register<C: ControllerSpec>(&self) {
        self.controllers
            .write()
            .entry(C::kind())
            .or_default()
            .insert(C::type_id());
    }

    pub fn read(&self) -> RwLockReadGuard<'_, RawRwLock, HashMap<Controller, HashSet<TypeId>>> {
        self.controllers.read()
    }

    pub fn snapshot(&self) -> ControllerMap {
        self.controllers.read().clone()
    }
}

impl Default for ControllerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
