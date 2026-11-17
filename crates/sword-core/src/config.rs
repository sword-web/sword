mod check;
mod validation;

pub use check::*;
pub use thisconfig::{ByteConfig, Config, ConfigItem, ConfigSourceInfo, TimeConfig};
pub use validation::ConfigValidateExt;

use crate::State;
use crate::diagnostic::Error;

/// A struct that holds a function to register a config type.
/// Used by the inventory system to collect all config types at compile time.
pub struct ConfigRegistrar {
    pub register: fn(&State, &Config) -> Result<(), Error>,
}

impl ConfigRegistrar {
    pub const fn new(register: fn(&State, &Config) -> Result<(), Error>) -> Self {
        Self { register }
    }
}

inventory::collect!(ConfigRegistrar);

pub trait DisplayConfig {
    fn display(&self);
}
