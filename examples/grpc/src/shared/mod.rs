pub mod database;
pub mod errors;
pub mod hasher;
pub mod interceptor;

pub use database::*;
pub use errors::*;
pub use hasher::*;
pub use interceptor::*;

use sword::prelude::*;

pub struct SharedModule;

impl Module for SharedModule {
    async fn register_providers(_config: &Config, providers: &ProviderRegistry) {
        providers.register(Database::new());
        providers.register(Hasher::new());
    }
}
