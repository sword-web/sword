use chrono::{DateTime, Utc};

use crate::{shared::AppError, users::proto::UserItem};

#[derive(Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
pub struct UserView {
    pub id: String,
    pub username: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
}

impl From<UserView> for User {
    fn from(view: UserView) -> Self {
        Self {
            id: view.id,
            username: view.username,
            password: view.password,
            created_at: view.created_at,
        }
    }
}

impl TryFrom<&User> for UserItem {
    type Error = AppError;

    fn try_from(user: &User) -> Result<Self, Self::Error> {
        Ok(UserItem {
            id: user.id.clone(),
            username: user.username.clone(),
            created_at: Some(prost_types::Timestamp {
                seconds: user.created_at.timestamp(),
                nanos: user.created_at.timestamp_subsec_nanos() as i32,
            }),
        })
    }
}
