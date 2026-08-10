use tokio_stream::Stream;
use tonic::Status;

use crate::controller::GrpcStream;
pub use sword_macros::GrpcError;

pub struct GrpcResponse;

impl GrpcResponse {
    pub fn message<T>(value: T) -> tonic::Response<T> {
        tonic::Response::new(value)
    }

    pub fn stream<T, S>(stream: S) -> tonic::Response<GrpcStream<T>>
    where
        S: Stream<Item = Result<T, Status>> + Send + 'static,
    {
        tonic::Response::new(Box::pin(stream))
    }
}

#[cfg(feature = "error-details")]
mod grpc_status {
    use std::collections::HashMap;
    use tonic::Code;
    use tonic_types::{ErrorDetails, StatusExt};

    /// A buildable gRPC status implementing the [gRPC Richer Error Model].
    ///
    /// Construct it with an associated function per status code (e.g.
    /// [`GrpcStatus::InvalidArgument`]) and chain detail builders before
    /// converting it into a [`tonic::Status`] with `.into()` or [`build`].
    ///
    /// Error details are applicable to any status code: the gRPC standard
    /// recommends pairings but does not enforce them.
    ///
    /// ```rust,ignore
    /// use sword::grpc::*;
    ///
    /// let status: tonic::Status = GrpcStatus::InvalidArgument()
    ///     .message("invalid request")
    ///     .bad_request("username", "username cannot be empty")
    ///     .into();
    /// ```
    ///
    /// [gRPC Richer Error Model]: https://grpc.io/docs/guides/error/
    /// [`build`]: GrpcStatus::build
    #[derive(Debug, Clone)]
    pub struct GrpcStatus {
        code: Code,
        message: String,
        details: ErrorDetails,
    }

    macro_rules! code_constructor {
        ($(#[$doc:meta] $name:ident => $code:expr),* $(,)?) => {
            $(
                #[$doc]
                pub fn $name() -> Self {
                    Self::new($code)
                }
            )*
        };
    }

    #[allow(non_snake_case)]
    impl GrpcStatus {
        code_constructor! {
            /// Creates a `GrpcStatus` with code `Ok`.
            Ok => Code::Ok,
            /// Creates a `GrpcStatus` with code `Cancelled`.
            Cancelled => Code::Cancelled,
            /// Creates a `GrpcStatus` with code `Unknown`.
            Unknown => Code::Unknown,
            /// Creates a `GrpcStatus` with code `InvalidArgument`.
            InvalidArgument => Code::InvalidArgument,
            /// Creates a `GrpcStatus` with code `DeadlineExceeded`.
            DeadlineExceeded => Code::DeadlineExceeded,
            /// Creates a `GrpcStatus` with code `NotFound`.
            NotFound => Code::NotFound,
            /// Creates a `GrpcStatus` with code `AlreadyExists`.
            AlreadyExists => Code::AlreadyExists,
            /// Creates a `GrpcStatus` with code `PermissionDenied`.
            PermissionDenied => Code::PermissionDenied,
            /// Creates a `GrpcStatus` with code `ResourceExhausted`.
            ResourceExhausted => Code::ResourceExhausted,
            /// Creates a `GrpcStatus` with code `FailedPrecondition`.
            FailedPrecondition => Code::FailedPrecondition,
            /// Creates a `GrpcStatus` with code `Aborted`.
            Aborted => Code::Aborted,
            /// Creates a `GrpcStatus` with code `OutOfRange`.
            OutOfRange => Code::OutOfRange,
            /// Creates a `GrpcStatus` with code `Unimplemented`.
            Unimplemented => Code::Unimplemented,
            /// Creates a `GrpcStatus` with code `Internal`.
            Internal => Code::Internal,
            /// Creates a `GrpcStatus` with code `Unavailable`.
            Unavailable => Code::Unavailable,
            /// Creates a `GrpcStatus` with code `DataLoss`.
            DataLoss => Code::DataLoss,
            /// Creates a `GrpcStatus` with code `Unauthenticated`.
            Unauthenticated => Code::Unauthenticated,
        }

        /// Creates a `GrpcStatus` from a `tonic::Code` with the code's canonical
        /// message and no details. Override the message with [`message`].
        ///
        /// [`message`]: GrpcStatus::message
        pub fn new(code: Code) -> Self {
            Self {
                code,
                message: code.to_string(),
                details: ErrorDetails::new(),
            }
        }

        /// Sets the status message.
        pub fn message(mut self, message: impl Into<String>) -> Self {
            self.message = message.into();
            self
        }

        /// Adds a [`BadRequest`] field violation.
        ///
        /// [`BadRequest`]: tonic_types::BadRequest
        pub fn bad_request(
            mut self,
            field: impl Into<String>,
            description: impl Into<String>,
        ) -> Self {
            self.details.add_bad_request_violation(field, description);
            self
        }

        /// Sets a [`LocalizedMessage`] detail.
        ///
        /// [`LocalizedMessage`]: tonic_types::LocalizedMessage
        pub fn localized_message(
            mut self,
            locale: impl Into<String>,
            message: impl Into<String>,
        ) -> Self {
            self.details.set_localized_message(locale, message);
            self
        }

        /// Sets an [`ErrorInfo`] detail.
        ///
        /// [`ErrorInfo`]: tonic_types::ErrorInfo
        pub fn error_info(
            mut self,
            domain: impl Into<String>,
            reason: impl Into<String>,
            metadata: HashMap<String, String>,
        ) -> Self {
            self.details.set_error_info(reason, domain, metadata);
            self
        }

        /// Sets a [`RetryInfo`] detail advising clients to retry after the given delay.
        ///
        /// [`RetryInfo`]: tonic_types::RetryInfo
        pub fn retry_after(mut self, delay: std::time::Duration) -> Self {
            self.details.set_retry_info(Some(delay));
            self
        }

        /// Adds a [`Help`] link.
        ///
        /// [`Help`]: tonic_types::Help
        pub fn help(mut self, description: impl Into<String>, url: impl Into<String>) -> Self {
            self.details.add_help_link(description, url);
            self
        }

        /// Sets a [`DebugInfo`] detail with stack entries and an additional detail string.
        ///
        /// [`DebugInfo`]: tonic_types::DebugInfo
        pub fn debug_info(
            mut self,
            stack_entries: impl Into<Vec<String>>,
            detail: impl Into<String>,
        ) -> Self {
            self.details.set_debug_info(stack_entries, detail);
            self
        }

        /// Adds a [`PreconditionFailure`] violation.
        ///
        /// [`PreconditionFailure`]: tonic_types::PreconditionFailure
        pub fn precondition_failure(
            mut self,
            violation_type: impl Into<String>,
            subject: impl Into<String>,
            description: impl Into<String>,
        ) -> Self {
            self.details
                .add_precondition_failure_violation(violation_type, subject, description);
            self
        }

        /// Adds a [`QuotaFailure`] violation.
        ///
        /// [`QuotaFailure`]: tonic_types::QuotaFailure
        pub fn quota_failure(
            mut self,
            subject: impl Into<String>,
            description: impl Into<String>,
        ) -> Self {
            self.details
                .add_quota_failure_violation(subject, description);
            self
        }

        /// Sets a [`RequestInfo`] detail.
        ///
        /// [`RequestInfo`]: tonic_types::RequestInfo
        pub fn request_info(
            mut self,
            request_id: impl Into<String>,
            serving_data: impl Into<String>,
        ) -> Self {
            self.details.set_request_info(request_id, serving_data);
            self
        }

        /// Sets a [`ResourceInfo`] detail.
        ///
        /// [`ResourceInfo`]: tonic_types::ResourceInfo
        pub fn resource_info(
            mut self,
            resource_type: impl Into<String>,
            resource_name: impl Into<String>,
            owner: impl Into<String>,
            description: impl Into<String>,
        ) -> Self {
            self.details
                .set_resource_info(resource_type, resource_name, owner, description);
            self
        }

        /// Builds the `GrpcStatus` into a [`tonic::Status`] with the attached error details.
        pub fn build(self) -> tonic::Status {
            <tonic::Status as StatusExt>::with_error_details(self.code, self.message, self.details)
        }

        /// Reconstructs a `GrpcStatus` from a [`tonic::Status`], extracting its
        /// code, message, and error details.
        pub fn from_status(status: &tonic::Status) -> Self {
            Self {
                code: status.code(),
                message: status.message().to_string(),
                details: status.get_error_details(),
            }
        }

        /// Returns the status code.
        pub fn code(&self) -> Code {
            self.code
        }

        /// Returns the status message.
        pub fn message_text(&self) -> &str {
            &self.message
        }

        /// Returns a reference to the error details.
        pub fn details(&self) -> &ErrorDetails {
            &self.details
        }
    }

    impl From<GrpcStatus> for tonic::Status {
        fn from(status: GrpcStatus) -> Self {
            status.build()
        }
    }
}

#[cfg(feature = "error-details")]
pub use grpc_status::GrpcStatus;
