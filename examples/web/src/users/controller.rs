use serde_json::json;
use std::sync::Arc;
use sword::prelude::*;
use sword::web::*;
use uuid::Uuid;

use crate::{
    shared::{Hasher, errors::AppError},
    users::*,
};

#[controller(kind = Controller::Web, path = "/users")]
pub struct UsersController {
    hasher: Arc<Hasher>,
    users: Arc<UserRepository>,
}

impl UsersController {
    #[get("/")]
    async fn get_users(&self, req: Request) -> WebResult<JsonResponse> {
        let data = self.users.find_all().await?;

        Ok(JsonResponse::Ok().data(data).request_id(req.id()))
    }

    #[post("/")]
    async fn create_user(&self, req: Request) -> WebResult {
        let body = req.body_validator::<CreateUserDto>()?;
        let user = User::new(body.username, self.hasher.hash(&body.password)?);

        if self.users.find_by_username(&user.username).await?.is_some() {
            tracing::error!(
                "Attempt to create user with existing username: {}",
                user.username
            );

            return Err(AppError::UserConflictError(
                format!("User with username '{}' already exists", user.username),
                "username",
                &user.username,
            ))?;
        }

        self.users.save(&user).await?;

        Ok(JsonResponse::Created().message("User created").data(user))
    }

    #[put("/{id}")]
    async fn update_user(&self, req: Request) -> WebResult {
        let id = req.param::<Uuid>("id")?;
        let body = req.body_validator::<UpdateUserDto>()?;

        let Some(existing_user) = self.users.find_by_id(&id).await? else {
            return Err(AppError::NotFoundError("User not found"))?;
        };

        let username = body.username.unwrap_or(existing_user.username.clone());

        let password = match &body.password {
            Some(pwd) => self.hasher.hash(pwd)?,
            None => existing_user.password.clone(),
        };

        let updated_user = User {
            id,
            username,
            password,
        };

        self.users.save(&updated_user).await?;

        Ok(JsonResponse::Ok().message("User updated"))
    }

    #[delete("/{id}")]
    async fn delete_user(&self, req: Request) -> WebResult {
        let id = req.param::<Uuid>("id")?;

        let Some(_) = self.users.find_by_id(&id).await? else {
            return Err(AppError::NotFoundError("User not found"))?;
        };

        self.users.delete(&id).await?;

        Ok(JsonResponse::Ok().message("User deleted"))
    }

    #[get("/test-compression")]
    async fn test_compression(&self) -> WebResult {
        let repeated_data = "x".repeat(5000); // 5KB de 'x'
        let large_json = json!({
            "size_kb": 5,
            "data": repeated_data,
        });

        Ok(JsonResponse::Ok()
            .data(large_json)
            .message("Test compression data"))
    }
}
