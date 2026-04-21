use crate::{ComponentRegistry, Config, ControllerRegistry, ProviderRegistry};

/// A trait for defining modules in the application.
///
/// `Module` represents a cohesive unit of functionality that can be
/// plugged into the application. Modules can register controllers,
/// components, and providers to extend the application's capabilities.
///
/// # Example
///
/// ```rust,ignore
/// use sword_core::Module;
/// use sword_core::ControllerRegistry;
///
/// pub struct MyModule;
///
/// impl Module for MyModule {
///     fn register_controllers(controllers: &ControllerRegistry) {
///         controllers.register::<MyController>();  // Register HTTP controller
///     }
///
///     fn register_components(components: &ComponentRegistry) {
///         components.register::<MyService>();
///     }
///
///     async fn register_providers(_: &Config, providers: &ProviderRegistry) {
///         providers.register(MyProvider::new().await);
///     }
/// }
/// ```
#[allow(async_fn_in_trait)]
#[allow(unused_variables)]
pub trait Module {
    /// Register controllers provided by the module.
    /// A `Controller` is a way to represent entry points into the application,
    /// such as HTTP controllers, Socket.IO Handlers, or gRPC services.
    fn register_controllers(controllers: &ControllerRegistry) {}

    /// Register component structs marked with `#[injectable]`
    fn register_components(components: &ComponentRegistry) {}

    /// Register provider structs marked with `#[injectable(provider)]`
    async fn register_providers(config: &Config, providers: &ProviderRegistry) {}
}
