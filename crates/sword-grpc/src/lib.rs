mod application;
mod client;
mod config;
mod controller;
mod interceptor;
mod logger;
mod registry;
mod response;

pub use tonic::transport::Channel;

pub use self::application::GrpcApplication;
pub use self::client::{
    ClientConnectionKind, DEFAULT_TIMEOUT, FromConfig, GrpcClientConfig, GrpcClientError,
};
pub use self::config::GrpcApplicationConfig;
pub use self::controller::{GrpcResult, GrpcStream};
pub use self::interceptor::{GrpcInterceptorResult, OnRequest, OnRequestWithConfig};
pub use self::logger::{GrpcLoggerConfig, GrpcLoggerLevel};
#[cfg(feature = "error-details")]
pub use self::response::GrpcStatus;
pub use self::response::{GrpcError, GrpcResponse};

pub mod prelude {
    pub use tonic::transport::Channel;
    pub use tonic::{
        Code, Extensions, Request, Response, Status, Streaming, async_trait, include_proto,
    };
    #[cfg(feature = "error-details")]
    pub use tonic_types::{
        BadRequest, DebugInfo, ErrorDetail, ErrorDetails, ErrorInfo, FieldViolation, Help,
        HelpLink, LocalizedMessage, PreconditionFailure, PreconditionViolation, QuotaFailure,
        QuotaViolation, RequestInfo, ResourceInfo, RetryInfo, StatusExt,
    };

    pub use crate::client::{ClientConnectionKind, FromConfig, GrpcClientConfig, GrpcClientError};
    pub use crate::config::GrpcApplicationConfig;
    pub use crate::controller::{GrpcResult, GrpcStream};
    pub use crate::interceptor::{GrpcInterceptorResult, OnRequest, OnRequestWithConfig};
    pub use crate::logger::{GrpcLoggerConfig, GrpcLoggerLevel};
    #[cfg(feature = "error-details")]
    pub use crate::response::GrpcStatus;
    pub use crate::response::{GrpcError, GrpcResponse};
}

/// Items required by generated code.
///
/// This module is public so procedural macros can name these items from the
/// user's crate, but it is not part of the stable API.
#[doc(hidden)]
pub mod internal {
    pub use tonic;
    pub use tonic_async_interceptor;
    #[cfg(feature = "error-details")]
    pub use tonic_types;

    pub use sword_layers::body_limit::GrpcBodyLimitValue;

    pub use crate::controller::{GrpcController, GrpcControllerRegistrar};
    pub use crate::registry::GrpcServiceRegistry;
}
