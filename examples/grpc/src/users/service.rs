use std::sync::Arc;

use chrono::Utc;
use sword::prelude::*;

use crate::{
    shared::{AppError, AppResult, Hasher},
    users::{CreateUserDto, UpdateUserDto, User, UserRepository},
};

#[injectable]
pub struct UserService {
    repository: Arc<UserRepository>,
    hasher: Arc<Hasher>,
}

impl UserService {
    pub async fn find_all(&self) -> AppResult<Vec<User>> {
        Ok(self.repository.find_all().await?)
    }

    pub async fn find_by_id(&self, id: &str) -> AppResult<User> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFoundError {
                message: "User not found".to_string(),
            })
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

        if self
            .repository
            .find_by_username(&dto.username)
            .await?
            .is_some()
        {
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
            created_at: Utc::now(),
        };

        self.repository.set(&user).await?;

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
            created_at: existing.created_at,
        };

        self.repository.upsert(&updated).await?;

        Ok(updated)
    }

    pub async fn delete(&self, id: &str) -> AppResult<()> {
        let deleted = self.repository.delete(id).await?;

        if deleted {
            Ok(())
        } else {
            Err(AppError::NotFoundError {
                message: "User not found".to_string(),
            })
        }
    }
}
