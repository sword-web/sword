mod grpc_error;
#[cfg(feature = "web-controllers")]
mod http_error;

pub use grpc_error::*;
#[cfg(feature = "web-controllers")]
pub use http_error::*;

#[derive(Debug, Clone)]
pub enum MessageValue {
    Static(String),
    Field(String),
}
