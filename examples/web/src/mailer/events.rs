use serde::{Deserialize, Serialize};
use sword::events::*;

#[event(key = "user.created")]
#[derive(Serialize, Deserialize)]
pub struct UserCreatedEvent {
    pub user_id: String,
    pub username: String,
    pub email: String,
}
