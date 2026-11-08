mod application;
mod client;
mod config;
mod controller;
mod interceptor;
mod logger;
mod registry;
mod response;

pub use application::GrpcApplication;
pub use client::{
    ClientConnectionKind, DEFAULT_TIMEOUT, FromConfig, GrpcClientConfig, GrpcClientError,
};
pub use config::GrpcApplicationConfig;
pub use controller::{GrpcResult, GrpcStream};
pub use interceptor::{GrpcInterceptorResult, OnRequest, OnRequestWithConfig};
pub use logger::{GrpcLoggerConfig, GrpcLoggerLevel};
#[cfg(feature = "error-details")]
pub use response::GrpcStatus;
pub use response::{GrpcError, GrpcResponse};
pub use tonic::transport::Channel;

pub mod prelude {
    pub use crate::client::{ClientConnectionKind, FromConfig, GrpcClientConfig, GrpcClientError};
    pub use crate::config::GrpcApplicationConfig;
    pub use crate::controller::{GrpcResult, GrpcStream};
    pub use crate::interceptor::{GrpcInterceptorResult, OnRequest, OnRequestWithConfig};
    pub use crate::logger::{GrpcLoggerConfig, GrpcLoggerLevel};
    pub use crate::response::{GrpcError, GrpcResponse};
    pub use tonic::transport::Channel;
    pub use tonic::{
        Code, Extensions, Request, Response, Status, Streaming, async_trait, include_proto,
    };

    #[cfg(feature = "error-details")]
    pub use crate::response::GrpcStatus;
    #[cfg(feature = "error-details")]
    pub use tonic_types::{
        BadRequest, DebugInfo, ErrorDetail, ErrorDetails, ErrorInfo, FieldViolation, Help,
        HelpLink, LocalizedMessage, PreconditionFailure, PreconditionViolation, QuotaFailure,
        QuotaViolation, RequestInfo, ResourceInfo, RetryInfo, StatusExt,
    };
}

/// Items required by generated code.
///
/// This module is public so procedural macros can name these items from the
/// user's crate, but it is not part of the stable API.
#[doc(hidden)]
pub mod internal {
    pub use crate::controller::{GrpcController, GrpcControllerRegistrar};
    pub use crate::registry::GrpcServiceRegistry;
    pub use sword_layers::body_limit::GrpcBodyLimitValue;
    pub use tonic;
    pub use tonic_async_interceptor;
    #[cfg(feature = "error-details")]
    pub use tonic_types;
}
