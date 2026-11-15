use std::sync::Arc;

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use sword::prelude::*;

use crate::shared::errors::AppResult;

/// In-memory SQLite database. It is created on startup and dropped when the
/// process exits, so the example runs without any external service.
#[injectable(provider)]
pub struct Database {
    pool: Arc<SqlitePool>,
}

impl Database {
    pub async fn new() -> AppResult<Self> {
        let options = SqliteConnectOptions::new()
            .filename(":memory:")
            .create_if_missing(true);

        // `:memory:` databases are per connection, so the pool keeps a single
        // connection to share the same data across every query.
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id BLOB PRIMARY KEY,
                username TEXT NOT NULL UNIQUE,
                password TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    pub fn get_pool(&self) -> &SqlitePool {
        &self.pool
    }
}
