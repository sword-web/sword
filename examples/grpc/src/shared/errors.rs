use sword::grpc::*;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error, GrpcError)]
#[grpc_error(code = "internal", tracing = error)]
pub enum AppError {
    #[grpc(code = "invalid_argument", message = reason)]
    #[error("Invalid argument: {reason}")]
    InvalidArgument { reason: String },

    #[grpc(code = "not_found", message = message)]
    #[error("Not found: {message}")]
    NotFoundError { message: String },

    #[grpc(code = "already_exists", message = value)]
    #[error("Conflict error username - {value}")]
    UserConflictError { value: String },

    #[grpc(code = "invalid_argument", message = reason)]
    #[error("Invalid name: {reason}")]
    InvalidName { reason: String },

    #[grpc(code = "unavailable", tracing = warn)]
    #[error("Greeter service is temporarily unavailable")]
    SystemUnavailable,
}
