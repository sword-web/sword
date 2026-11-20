use std::{path::Path, time::Duration};

use serde::Deserialize;
use sqlx::{
    migrate::Migrator,
    postgres::{PgPool, PgPoolOptions},
};

use sword::prelude::*;

#[config(key = "postgress", required = false)]
#[derive(Debug, Clone, Deserialize)]
pub struct PostgresConfig {
    pub user: String,
    pub password: String,
    pub database: String,
    pub port: u32,
    pub host: String,
    pub migrations_path: String,
    pub min_connections: u32,
    pub max_connections: u32,
    pub acquire_timeout_ms: u64,
}

#[injectable(provider)]
pub struct SqlxPostgres {
    pool: PgPool,
}

impl SqlxPostgres {
    pub async fn new(db_conf: PostgresConfig) -> Self {
        let pool = PgPoolOptions::new()
            .min_connections(db_conf.min_connections)
            .max_connections(db_conf.max_connections)
            .acquire_timeout(Duration::from_millis(db_conf.acquire_timeout_ms))
            .connect(&Self::create_uri(&db_conf))
            .await
            .expect("Failed to create database connection pool");

        let migrator = Migrator::new(Path::new(&db_conf.migrations_path))
            .await
            .expect("Failed to initialize migrator");

        migrator
            .run(&pool)
            .await
            .expect("Failed to run database migrations");

        Self { pool }
    }

    fn create_uri(db_conf: &PostgresConfig) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            db_conf.user, db_conf.password, db_conf.host, db_conf.port, db_conf.database
        )
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

pub struct SqlxPostgresPlugin;

impl Plugin for SqlxPostgresPlugin {
    fn name() -> &'static str {
        "sqlx-postgres"
    }

    async fn register_providers(config: &Config, providers: &ProviderRegistry) {
        let db_conf = config.expect::<PostgresConfig>();

        providers.register(SqlxPostgres::new(db_conf).await);
    }
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            user: "user".into(),
            password: "password".into(),
            database: "database".into(),
            port: 5432,
            host: "localhost".into(),
            migrations_path: "migrations".into(),
            min_connections: 1,
            max_connections: 10,
            acquire_timeout_ms: 5000,
        }
    }
}
