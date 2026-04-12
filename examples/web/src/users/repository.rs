use super::entity::User;
use crate::shared::{database::Database, errors::AppResult};

use std::sync::Arc;
use sword::prelude::*;
use uuid::Uuid;

#[injectable]
pub struct UserRepository {
    db: Arc<Database>,
}

impl UserRepository {
    pub async fn find_by_id(&self, id: &Uuid) -> AppResult<Option<User>> {
        let result = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(self.db.get_pool())
            .await?;

        Ok(result)
    }

    pub async fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
        let result = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(self.db.get_pool())
            .await?;

        Ok(result)
    }

    pub async fn find_all(&self) -> AppResult<Vec<User>> {
        let result = sqlx::query_as::<_, User>("SELECT * FROM users")
            .fetch_all(self.db.get_pool())
            .await?;

        Ok(result)
    }

    pub async fn save(&self, user: &User) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO users (id, username, password)
            VALUES ($1, $2, $3)
            ON CONFLICT (id)
            DO UPDATE SET
                username = EXCLUDED.username,
                password = EXCLUDED.password
        "#,
        )
        .bind(user.id)
        .bind(&user.username)
        .bind(&user.password)
        .execute(self.db.get_pool())
        .await?;

        Ok(())
    }

    pub async fn delete(&self, id: &Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(self.db.get_pool())
            .await?;

        Ok(())
    }
}
