use std::path::Path;

use serde::Deserialize;
use sqlx::{
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions},
};

use sword::prelude::*;

#[config(key = "sqlite", required = false)]
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct SqliteConfig {
    /// `:memory:` for an in-memory database, or a path to a SQLite file.
    pub path: String,

    /// Keep this at `1` for `:memory:` databases: every connection gets its
    /// own in-memory database otherwise.
    pub max_connections: u32,

    pub create_if_missing: bool,

    /// Optional directory with SQLx migrations.
    pub migrations_path: Option<String>,
}

impl Default for SqliteConfig {
    fn default() -> Self {
        Self {
            path: ":memory:".to_string(),
            max_connections: 1,
            create_if_missing: true,
            migrations_path: None,
        }
    }
}

#[injectable(provider)]
pub struct SqliteDatabase {
    pool: SqlitePool,
}

impl SqliteDatabase {
    pub async fn new(config: &SqliteConfig) -> Self {
        let options = SqliteConnectOptions::new()
            .filename(Path::new(&config.path))
            .create_if_missing(config.create_if_missing);

        let pool = SqlitePoolOptions::new()
            .max_connections(config.max_connections)
            .connect_with(options)
            .await
            .expect("sqlx-sqlite plugin: failed to connect");

        if let Some(migrations_path) = &config.migrations_path {
            let migrator = Migrator::new(Path::new(migrations_path))
                .await
                .expect("sqlx-sqlite plugin: failed to load migrations");

            migrator
                .run(&pool)
                .await
                .expect("sqlx-sqlite plugin: failed to run migrations");
        }

        Self { pool }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

pub struct SqlxSqlitePlugin;

impl Plugin for SqlxSqlitePlugin {
    fn name() -> &'static str {
        "sqlx-sqlite"
    }

    async fn register_providers(config: &Config, providers: &ProviderRegistry) {
        let config = config.get_or_default::<SqliteConfig>();
        providers.register(SqliteDatabase::new(&config).await);
    }
}
