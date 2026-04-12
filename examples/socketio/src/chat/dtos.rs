use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct IncommingMessageDto {
    #[validate(length(min = 1, max = 500))]
    pub content: String,
}
