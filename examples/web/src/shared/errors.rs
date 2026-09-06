use sword::web::*;
use thiserror::Error as ThisError;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, ThisError, HttpError)]
#[http_error(code = 500, tracing = error, message = "Internal server error")]
pub enum AppError {
    #[error("Database error occurred: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Hasher error occurred: {0}")]
    HasherError(#[from] argon2::password_hash::Error),

    #[error("Hasher task error occurred: {0}")]
    HasherTaskError(#[from] tokio::task::JoinError),

    #[http(code = 404)]
    #[tracing(warn)]
    #[error("Not found error: {message}")]
    NotFoundError { message: String },

    #[http(code = 409, message = "User with username '{value}' already exists")]
    #[tracing(error)]
    #[error("Conflict error {field} - {value}")]
    UserConflictError { field: String, value: String },
}
