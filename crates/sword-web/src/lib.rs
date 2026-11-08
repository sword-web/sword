mod application;
mod controller;
mod interceptor;
mod request;
mod response;

#[cfg(feature = "swagger-ui")]
pub use self::application::OpenApiConfig;
pub use self::application::{
    WebApplication, WebApplicationConfig, WebExtension, WebExtensionContext, WebExtensionRegistrar,
    WebLoggerConfig, WebLoggerLevel,
};
pub use self::controller::{
    WebController, connect, controller, delete, get, head, options, patch, post, put, sse, trace,
};
pub use self::interceptor::{
    OnRequest, OnRequestStream, OnRequestStreamWithConfig, OnRequestWithConfig,
    WebInterceptorResult,
};
#[cfg(feature = "validator")]
pub use self::request::ValidatorRequestValidation;
pub use self::request::{Request, RequestError, StreamRequest};
pub use self::response::{
    ContentDisposition, Event, EventStream, File, HttpError, JsonResponse, JsonResponseBody,
    KeepAlive, Redirect, Sse, WebResult,
};

pub mod prelude {
    pub use async_stream::{stream, try_stream};
    pub use axum::body::Bytes as BodyBytes;
    pub use axum::extract::RawPathParams;
    #[cfg(feature = "multipart")]
    pub use axum::extract::multipart::{
        Field, InvalidBoundary, Multipart, MultipartError, MultipartRejection,
    };
    pub use axum::http::{HeaderMap as Headers, Method, Uri};
    pub use axum::middleware::Next;
    #[cfg(feature = "multipart")]
    pub use bytes::*;

    pub use sword_layers::cookies::{
        Cookies, Key as CookiesKey, PrivateCookies, SignedCookies,
        cookie::{
            Cookie, CookieBuilder, Expiration as CookiesExpiration, KeyError as CookieKeyError,
            ParseError as CookieParseError, SameSite,
        },
    };

    pub use crate::controller::{
        WebController, connect, delete, get, head, options, patch, post, put, sse, trace,
    };
    pub use crate::interceptor::{
        OnRequest, OnRequestStream, OnRequestStreamWithConfig, OnRequestWithConfig,
        WebInterceptorResult,
    };
    #[cfg(feature = "validator")]
    pub use crate::request::ValidatorRequestValidation;
    pub use crate::request::{Request, RequestError, StreamRequest};
    pub use crate::response::{
        ContentDisposition, Event, EventStream, File, HttpError, JsonResponse, JsonResponseBody,
        KeepAlive, Redirect, Sse, WebResult,
    };
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
