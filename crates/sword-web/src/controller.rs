use sword_core::ControllerSpec;

pub use sword_macros::{connect, controller, delete, get, head, options, patch, post, put, trace};

/// Trait for controllers with automatic dependency injection and interceptors support.
///
/// Controllers in Sword group related route handlers together and can declare
/// dependencies that will be automatically resolved and injected.
///
/// Controllers automatically implement the `ControllerSpec` trait, making them registrable
/// as HTTP controllers within modules via `ControllerRegistry::register::<YourController>()`.
///
/// Use the `#[controller]` macro to automatically implement this trait.
///
/// # Example
///
/// ```rust,ignore
/// use sword::prelude::*;
///
/// #[controller(kind = Controller::Web, path = "/api/users")]
/// struct UserController {
///     service: Arc<UserService>,
/// }
///
/// impl UserController {
///     #[get("/")]
///     async fn list(&self) -> JsonResponse {
///         // Handler logic
///     }
/// }
/// ```
pub trait WebController: ControllerSpec {
    fn base_path() -> &'static str;
}

use axum::routing::MethodRouter;
use std::any::TypeId;
use sword_core::State;

#[derive(Clone)]
pub struct WebControllerRegistrar {
    pub controller_id: TypeId,
    pub controller_path: &'static str,
    pub build: fn(&State),
}

#[derive(Clone)]
pub struct RouteRegistrar {
    /// TypeId of the controller for filtering during registration
    pub controller_id: TypeId,

    /// Path of this specific route (e.g., "/{id}")
    pub path: &'static str,

    /// Function that builds the MethodRouter for this route
    /// The closure constructs the controller from state and calls the specific __sword_route_* method
    pub handler: fn(State) -> MethodRouter<State>,
}

inventory::collect!(WebControllerRegistrar);
inventory::collect!(RouteRegistrar);
