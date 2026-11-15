use std::sync::Arc;

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use sword::prelude::*;

use crate::users::User;

type UserRow = (String, String, String, i64, i32);

/// In-memory SQLite database. It is created on startup and dropped when the
/// process exits, so the example runs without any external service.
#[injectable(provider)]
pub struct Database {
    pool: Arc<SqlitePool>,
}

impl Database {
    pub async fn new() -> sqlx::Result<Self> {
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
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL UNIQUE,
                password TEXT NOT NULL,
                created_seconds INTEGER NOT NULL,
                created_nanos INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    pub async fn get(&self, id: &str) -> sqlx::Result<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, username, password, created_seconds, created_nanos \
             FROM users WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&*self.pool)
        .await?;

        Ok(row.map(into_user))
    }

    pub async fn find_by_username(&self, username: &str) -> sqlx::Result<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, username, password, created_seconds, created_nanos \
             FROM users WHERE username = ?",
        )
        .bind(username)
        .fetch_optional(&*self.pool)
        .await?;

        Ok(row.map(into_user))
    }

    pub async fn get_all(&self) -> sqlx::Result<Vec<User>> {
        let rows = sqlx::query_as::<_, UserRow>(
            "SELECT id, username, password, created_seconds, created_nanos FROM users",
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows.into_iter().map(into_user).collect())
    }

    pub async fn set(&self, user: User) -> sqlx::Result<()> {
        sqlx::query(
            "INSERT INTO users (id, username, password, created_seconds, created_nanos) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&user.id)
        .bind(&user.username)
        .bind(&user.password)
        .bind(user.created_at.seconds)
        .bind(user.created_at.nanos)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn upsert(&self, user: User) -> sqlx::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO users (id, username, password, created_seconds, created_nanos)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT (id)
            DO UPDATE SET
                username = excluded.username,
                password = excluded.password
            "#,
        )
        .bind(&user.id)
        .bind(&user.username)
        .bind(&user.password)
        .bind(user.created_at.seconds)
        .bind(user.created_at.nanos)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete(&self, id: &str) -> sqlx::Result<bool> {
        let result = sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id)
            .execute(&*self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

fn into_user((id, username, password, seconds, nanos): UserRow) -> User {
    User {
        id,
        username,
        password,
        created_at: prost_types::Timestamp { seconds, nanos },
    }
}
