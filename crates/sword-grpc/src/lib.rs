pub mod application;
pub mod config;
pub mod controller;
pub mod interceptor;
pub mod registry;
pub mod response;

pub mod prelude {
    pub use crate::config::GrpcApplicationConfig;
    pub use crate::controller::{GrpcResult, GrpcStream};
    pub use crate::interceptor::{GrpcInterceptorResult, OnRequest, OnRequestWithConfig};
    pub use crate::response::{GrpcError, GrpcResponse};
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

#[doc(hidden)]
pub mod internal {
    pub use tonic;
    pub use tonic_async_interceptor;
    #[cfg(feature = "error-details")]
    pub use tonic_types;

    pub use crate::controller::{GrpcController, GrpcControllerRegistrar};
    pub use crate::registry::GrpcServiceRegistry;
    pub use sword_layers::body_limit::GrpcBodyLimitValue;
}
