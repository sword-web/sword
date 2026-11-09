mod application;
mod config;
mod controllers;
mod diagnostic;
mod injectables;
mod interceptor;
mod layer_stack;
mod module;
mod shutdown;
mod state;

pub use self::application::{ApplicationConfig, EngineBuildContext};
pub use self::config::{
    ByteConfig, Config, ConfigCheck, ConfigFailure, ConfigItem, ConfigProblem, ConfigRegistrar,
    ConfigSourceInfo, ConfigValidateExt, TimeConfig, config_failures_error,
};
pub use self::controllers::{
    Controller, ControllerIds, ControllerMap, ControllerRegistry, ControllerSpec,
};
pub use self::diagnostic::*;
pub use self::injectables::{
    Build, Component, ComponentRegistry, DependencyContainer, DependencyInjectionError, HasDeps,
    Provider, ProviderRegistry,
};
pub use self::interceptor::{Interceptor, InterceptorRegistrar};
pub use self::layer_stack::LayerStack;
pub use self::module::Module;
pub use self::shutdown::{Shutdown, shutdown_signal};
pub use self::state::State;

pub(crate) type RwMap<K, V> = parking_lot::RwLock<std::collections::HashMap<K, V>>;

#[macro_export]
macro_rules! inventory_submit {
    ({ $registrar:expr }) => {
        const _: () = {
            inventory::submit! {
                $registrar
            }
        };
    };

    ([ $( $registrar:expr ),* $(,)? ]) => {
        $(
            const _: () = {
                inventory::submit! {
                    $registrar
                }
            };
        )*
    };
}
