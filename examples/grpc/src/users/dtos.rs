#[derive(Clone)]
pub struct CreateUserDto {
    pub username: String,
    pub password: String,
}

#[derive(Clone)]
pub struct UpdateUserDto {
    pub id: String,
    pub username: String,
    pub password: String,
}
