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

pub use application::{ApplicationConfig, EngineBuildContext};

pub use config::{
    ByteConfig, Config, ConfigDocument, ConfigErrors, ConfigIssue, ConfigItem, ConfigRegistrar,
    ConfigSection, ConfigSourceInfo, TimeConfig, ValidateConfig,
};

pub use controllers::{
    Controller, ControllerIds, ControllerMap, ControllerRegistry, ControllerSpec,
};

pub use diagnostic::*;
pub use injectables::{
    Build, Component, ComponentRegistry, DependencyContainer, DependencyInjectionError, HasDeps,
    Provider, ProviderRegistry,
};

pub use interceptor::{Interceptor, InterceptorRegistrar};
pub use layer_stack::LayerStack;
pub use module::Module;
pub use shutdown::{Shutdown, shutdown_signal};
pub use state::State;

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
