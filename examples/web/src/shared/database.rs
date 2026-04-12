use serde::Deserialize;
use sqlx::{PgPool, migrate::Migrator};
use std::{path::Path, sync::Arc};
use sword::prelude::*;

use crate::shared::errors::AppResult;

#[config(key = "database")]
#[derive(Clone, Deserialize)]
pub struct DatabaseConfig {
    pub uri: String,
    pub migrations_path: String,
}

#[injectable(provider)]
pub struct Database {
    pool: Arc<PgPool>,
}

impl Database {
    pub async fn new(db_conf: DatabaseConfig) -> AppResult<Self> {
        let pool = PgPool::connect(&db_conf.uri).await?;
        let migrator = Migrator::new(Path::new(&db_conf.migrations_path))
            .await
            .expect("Failed to initialize migrator");

        migrator
            .run(&pool)
            .await
            .expect("Failed to run database migrations");

        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    pub fn get_pool(&self) -> &PgPool {
        &self.pool
    }
}
