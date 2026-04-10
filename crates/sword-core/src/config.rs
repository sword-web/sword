use crate::State;

pub use thisconfig::{ByteConfig, Config, ConfigItem, TimeConfig};

/// A struct that holds a function to register a config type.
/// Used by the inventory system to collect all config types at compile time.
pub struct ConfigRegistrar {
    pub register: fn(&State, &Config) -> (),
}

impl ConfigRegistrar {
    pub const fn new(register: fn(&State, &Config) -> ()) -> Self {
        Self { register }
    }
}

inventory::collect!(ConfigRegistrar);
