pub mod database;
pub mod errors;
pub mod hasher;
pub mod interceptor;

use sword::prelude::*;

pub use self::database::*;
pub use self::errors::*;
pub use self::hasher::*;
pub use self::interceptor::*;

pub struct SharedModule;

impl Module for SharedModule {
    async fn register_providers(_config: &Config, providers: &ProviderRegistry) {
        providers.register(Database::new());
        providers.register(Hasher::new());
    }
}
