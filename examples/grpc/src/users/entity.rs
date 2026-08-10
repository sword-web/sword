use crate::users::proto::UserItem;

#[derive(Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password: String,
    pub created_at: prost_types::Timestamp,
}

impl From<&User> for UserItem {
    fn from(user: &User) -> Self {
        UserItem {
            id: user.id.clone(),
            username: user.username.clone(),
            created_at: Some(user.created_at),
        }
    }
}
