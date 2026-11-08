mod application;
mod controller;
mod interceptor;
mod request;
mod response;

#[cfg(feature = "swagger-ui")]
pub use application::OpenApiConfig;
pub use application::{
    WebApplication, WebApplicationConfig, WebExtension, WebExtensionContext, WebExtensionRegistrar,
    WebLoggerConfig, WebLoggerLevel,
};
pub use controller::{
    WebController, connect, controller, delete, get, head, options, patch, post, put, sse, trace,
};
pub use interceptor::{
    OnRequest, OnRequestStream, OnRequestStreamWithConfig, OnRequestWithConfig,
    WebInterceptorResult,
};
#[cfg(feature = "validator")]
pub use request::ValidatorRequestValidation;
pub use request::{Request, RequestError, StreamRequest};
pub use response::{
    ContentDisposition, Event, EventStream, File, HttpError, JsonResponse, JsonResponseBody,
    KeepAlive, Redirect, Sse, WebResult,
};

pub mod prelude {
    pub use crate::controller::{
        WebController, connect, delete, get, head, options, patch, post, put, sse, trace,
    };
    pub use crate::interceptor::{
        OnRequest, OnRequestStream, OnRequestStreamWithConfig, OnRequestWithConfig,
        WebInterceptorResult,
    };
    pub use crate::request::{Request, RequestError, StreamRequest};
    pub use crate::response::{
        ContentDisposition, Event, EventStream, File, HttpError, JsonResponse, JsonResponseBody,
        KeepAlive, Redirect, Sse, WebResult,
    };
    pub use async_stream::{stream, try_stream};
    pub use axum::middleware::Next;

    #[cfg(feature = "validator")]
    pub use crate::request::ValidatorRequestValidation;

    pub use sword_layers::cookies::{
        Cookies, Key as CookiesKey, PrivateCookies, SignedCookies,
        cookie::{
            Cookie, CookieBuilder, Expiration as CookiesExpiration, KeyError as CookieKeyError,
            ParseError as CookieParseError, SameSite,
        },
    };

    #[cfg(feature = "multipart")]
    pub use axum::extract::multipart::{
        Field, InvalidBoundary, Multipart, MultipartError, MultipartRejection,
    };

    #[cfg(feature = "multipart")]
    pub use bytes::*;

    pub use axum::body::Bytes as BodyBytes;
    pub use axum::extract::RawPathParams;
    pub use axum::http::{HeaderMap as Headers, Method, Uri};
}

/// Items required by generated code.
///
/// This module is public so procedural macros can name these items from the
/// user's crate, but it is not part of the stable API.
#[doc(hidden)]
pub mod internal {
    pub use axum::extract::Request as AxumRequest;
    pub use axum::middleware::from_fn_with_state as mw_with_state;
    pub use axum::response::{IntoResponse, Response as AxumResponse};
    pub use axum::routing::{self, MethodRouter, Router as AxumRouter};
    pub use tower::{Layer as TowerLayer, Service as TowerService};

    pub use crate::controller::{RouteRegistrar, WebController, WebControllerRegistrar};
}
