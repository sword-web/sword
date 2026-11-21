use std::sync::Arc;

use plugins::redis::RedisCache;
use uuid::Uuid;

use sword::events::*;
use sword::prelude::*;
use sword::web::*;

use crate::{
    mailer::UserCreatedEvent,
    shared::{AuditInterceptor, Hasher, RequestLoggingInterceptor, errors::AppError},
    users::*,
};

#[controller(kind = Controller::Web, path = "/users")]
#[interceptor(RequestLoggingInterceptor)]
pub struct UsersController {
    hasher: Arc<Hasher>,
    users: Arc<UserRepository>,
    publisher: Arc<MemoryEventPublisher>,
    cache: Arc<RedisCache>,
}

const USERS_CACHE_KEY: &str = "users:list";

impl UsersController {
    #[get("/")]
    async fn get_users(&self, _: Request) -> WebResult<Vec<User>> {
        match self.cache.get::<Vec<User>>(USERS_CACHE_KEY).await {
            Ok(Some(users)) => return Ok(users),
            Ok(None) => {}
            Err(err) => tracing::warn!(error = %err, "users cache read failed"),
        }

        let users = self.users.find_all().await?;

        if let Err(err) = self.cache.set(USERS_CACHE_KEY, &users).await {
            tracing::warn!(error = %err, "users cache write failed");
        }

        Ok(users)
    }

    #[post("/")]
    #[interceptor(AuditInterceptor)]
    async fn create_user(&self, req: Request) -> WebResult<User> {
        let body = req.body_validator::<CreateUserDto>()?;

        if self.users.find_by_username(&body.username).await?.is_some() {
            tracing::error!(
                "Attempt to create user with existing username: {}",
                body.username
            );

            Err(AppError::UserConflictError("username", &body.username))?;
        }

        let user = User::new(body.username, self.hasher.hash(&body.password).await?);

        self.users.save(&user).await?;

        if let Err(err) = self.cache.invalidate(USERS_CACHE_KEY).await {
            tracing::warn!(error = %err, "users cache invalidation failed");
        }

        let event_data = UserCreatedEvent {
            user_id: user.id.to_string(),
            username: user.username.clone(),
            email: format!("{}@example.com", user.username),
        };

        self.publisher.publish(event_data).await.ok();

        Ok(user)
    }

    #[put("/{id}")]
    async fn update_user(&self, req: Request) -> WebResult<()> {
        let id = req.param::<Uuid>("id")?;
        let body = req.body_validator::<UpdateUserDto>()?;

        let Some(existing_user) = self.users.find_by_id(&id).await? else {
            Err(AppError::NotFoundError("User not found"))?
        };

        let username = body.username.unwrap_or(existing_user.username.clone());

        let password = match &body.password {
            Some(pwd) => self.hasher.hash(pwd).await?,
            None => existing_user.password.clone(),
        };

        let updated_user = User {
            id,
            username,
            password,
        };

        self.users.save(&updated_user).await?;

        if let Err(err) = self.cache.invalidate(USERS_CACHE_KEY).await {
            tracing::warn!(error = %err, "users cache invalidation failed");
        }

        Ok(())
    }

    #[delete("/{id}")]
    async fn delete_user(&self, req: Request) -> WebResult<()> {
        let id = req.param::<Uuid>("id")?;

        let Some(_) = self.users.find_by_id(&id).await? else {
            Err(AppError::NotFoundError("User not found"))?
        };

        self.users.delete(&id).await?;

        if let Err(err) = self.cache.invalidate(USERS_CACHE_KEY).await {
            tracing::warn!(error = %err, "users cache invalidation failed");
        }

        Ok(())
    }
}
