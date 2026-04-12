use std::sync::Arc;

use sword::prelude::*;

use crate::{
    shared::{AppError, AppResult, Database, Hasher},
    users::{CreateUserDto, UpdateUserDto, User},
};

#[injectable]
pub struct UserRepository {
    db: Arc<Database>,
    hasher: Arc<Hasher>,
}

impl UserRepository {
    pub async fn find_by_id(&self, id: &str) -> AppResult<User> {
        self.db
            .get(id)
            .await
            .ok_or_else(|| AppError::NotFoundError {
                message: "User not found".to_string(),
            })
    }

    pub async fn find_by_username(&self, username: &str) -> Option<User> {
        self.db.find_by_username(username).await
    }

    pub async fn find_all(&self) -> Vec<User> {
        self.db.get_all().await
    }

    pub async fn create(&self, dto: CreateUserDto) -> AppResult<User> {
        if dto.username.trim().is_empty() {
            return Err(AppError::InvalidArgument {
                reason: "username cannot be empty".to_string(),
            });
        }

        if dto.password.len() < 6 {
            return Err(AppError::InvalidArgument {
                reason: "password must contain at least 6 characters".to_string(),
            });
        }

        if self.find_by_username(&dto.username).await.is_some() {
            return Err(AppError::UserConflictError {
                value: dto.username,
            });
        }

        let id = uuid::Uuid::new_v4().to_string();
        let password = self.hasher.hash(&dto.password)?;

        let user = User {
            id,
            username: dto.username,
            password,
        };

        self.db.set(user.clone()).await;

        Ok(user)
    }

    pub async fn update(&self, dto: UpdateUserDto) -> AppResult<User> {
        let existing = self.find_by_id(&dto.id).await?;

        let username = if dto.username.trim().is_empty() {
            existing.username
        } else {
            dto.username
        };

        let password = if dto.password.is_empty() {
            existing.password
        } else {
            self.hasher.hash(&dto.password)?
        };

        let updated = User {
            id: dto.id,
            username,
            password,
        };

        self.db.upsert(updated.clone()).await;

        Ok(updated)
    }

    pub async fn delete(&self, id: &str) -> AppResult<()> {
        let deleted = self.db.delete(id).await;

        if deleted {
            Ok(())
        } else {
            Err(AppError::NotFoundError {
                message: "User not found".to_string(),
            })
        }
    }
}
