pub mod database;
pub mod errors;
pub mod hasher;

pub use database::*;
pub use hasher::*;

use sword::prelude::*;

pub struct SharedModule;

impl Module for SharedModule {
    async fn register_providers(config: &Config, providers: &ProviderRegistry) {
        let db_config = config.expect::<DatabaseConfig>();

        providers.register(
            Database::new(db_config)
                .await
                .expect("Failed to create Database provider"),
        );

        providers.register(Hasher::new());
    }
}
