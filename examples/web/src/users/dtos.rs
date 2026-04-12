use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct CreateUserDto {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Username must be between 1 and 100 characters"
    ))]
    pub username: String,

    #[validate(length(
        min = 6,
        max = 100,
        message = "Password must be between 6 and 100 characters"
    ))]
    pub password: String,
}

#[derive(Deserialize, Validate)]
pub struct UpdateUserDto {
    #[validate(length(
        min = 1,
        max = 100,
        message = "Username must be between 1 and 100 characters"
    ))]
    pub username: Option<String>,

    #[validate(length(
        min = 6,
        max = 100,
        message = "Password must be between 6 and 100 characters"
    ))]
    pub password: Option<String>,
}
