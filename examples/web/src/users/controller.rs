use crate::{
    mailer::UserCreatedEvent,
    shared::{Hasher, errors::AppError},
    users::*,
};

use std::sync::Arc;
use uuid::Uuid;

use sword::events::EventPublisher;
use sword::prelude::*;
use sword::web::*;

#[controller(kind = Controller::Web, path = "/users")]
pub struct UsersController {
    hasher: Arc<Hasher>,
    users: Arc<UserRepository>,
    publisher: Arc<EventPublisher>,
}

impl UsersController {
    #[get("/")]
    async fn get_users(&self, _: Request) -> WebResult<Vec<User>> {
        Ok(self.users.find_all().await?)
    }

    #[post("/")]
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

        self.publisher
            .publish(UserCreatedEvent {
                user_id: user.id.to_string(),
                username: user.username.clone(),
                email: format!("{}@example.com", user.username),
            })
            .await;

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

        Ok(())
    }

    #[delete("/{id}")]
    async fn delete_user(&self, req: Request) -> WebResult<()> {
        let id = req.param::<Uuid>("id")?;

        let Some(_) = self.users.find_by_id(&id).await? else {
            Err(AppError::NotFoundError("User not found"))?
        };

        self.users.delete(&id).await?;

        Ok(())
    }
}
