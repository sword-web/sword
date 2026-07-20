use sword::prelude::*;

#[event(key = "user.created")]
pub struct UserCreatedEvent {
    pub user_id: String,
    pub username: String,
    pub email: String,
}
