use std::sync::Arc;

use plugins::sqlx_sqlite::SqliteDatabase;
use sword::prelude::*;

use crate::users::{User, UserView};

#[injectable]
pub struct UserRepository {
    db: Arc<SqliteDatabase>,
}

impl UserRepository {
    pub async fn find_by_id(&self, id: &str) -> sqlx::Result<Option<User>> {
        let row = sqlx::query_as::<_, UserView>(
            "SELECT id, username, password, created_at \
             FROM users WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.db.pool())
        .await?;

        Ok(row.map(User::from))
    }

    pub async fn find_by_username(&self, username: &str) -> sqlx::Result<Option<User>> {
        let row = sqlx::query_as::<_, UserView>(
            "SELECT id, username, password, created_at \
             FROM users WHERE username = ?",
        )
        .bind(username)
        .fetch_optional(self.db.pool())
        .await?;

        Ok(row.map(User::from))
    }

    pub async fn find_all(&self) -> sqlx::Result<Vec<User>> {
        let rows =
            sqlx::query_as::<_, UserView>("SELECT id, username, password, created_at FROM users")
                .fetch_all(self.db.pool())
                .await?;

        Ok(rows.into_iter().map(User::from).collect())
    }

    pub async fn set(&self, user: &User) -> sqlx::Result<()> {
        sqlx::query(
            "INSERT INTO users (id, username, password, created_at) \
             VALUES (?, ?, ?, ?)",
        )
        .bind(&user.id)
        .bind(&user.username)
        .bind(&user.password)
        .bind(user.created_at)
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    pub async fn upsert(&self, user: &User) -> sqlx::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO users (id, username, password, created_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT (id)
            DO UPDATE SET
                username = excluded.username,
                password = excluded.password
            "#,
        )
        .bind(&user.id)
        .bind(&user.username)
        .bind(&user.password)
        .bind(user.created_at)
        .execute(self.db.pool())
        .await?;

        Ok(())
    }

    pub async fn delete(&self, id: &str) -> sqlx::Result<bool> {
        let result = sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id)
            .execute(self.db.pool())
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
