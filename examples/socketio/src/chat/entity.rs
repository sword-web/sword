use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::chat::IncommingMessageDto;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub content: String,
    pub timestamp: i64,
}

impl From<IncommingMessageDto> for Message {
    fn from(dto: IncommingMessageDto) -> Self {
        Message {
            id: Uuid::new_v4().to_string(),
            content: dto.content,
            timestamp: Utc::now().timestamp(),
        }
    }
}
