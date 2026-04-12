pub mod application;
pub mod config;
pub mod controller;
pub mod interceptor;
pub mod request;
pub mod response;
pub mod router;

pub mod prelude {
    pub use crate::controller::{
        WebController, connect, delete, get, head, options, patch, post, put, trace,
    };
    pub use crate::interceptor::{
        OnRequest, OnRequestStream, OnRequestStreamWithConfig, OnRequestWithConfig,
        WebInterceptorResult,
    };
    pub use crate::request::{Request, RequestError, StreamRequest};
    pub use crate::response::{
        ContentDisposition, File, HttpError, JsonResponse, JsonResponseBody, Redirect, WebResult,
    };
    pub use axum::middleware::Next;

    #[cfg(feature = "validation-validator")]
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
}

#[doc(hidden)]
pub mod internal {
    pub use axum::body::{Body as AxumBody, HttpBody as AxumHttpBody};
    pub use axum::extract::{FromRequest, FromRequestParts, Request as AxumRequest};
    pub use axum::middleware::{Next as AxumNext, from_fn_with_state as mw_with_state};
    pub use axum::response::{IntoResponse, Response as AxumResponse};
    pub use axum::routing;
    pub use axum::routing::{
        MethodRouter, Router as AxumRouter, connect, connect as connect_fn, delete,
        delete as delete_fn, get, get as get_fn, head, head as head_fn, options,
        options as options_fn, patch, patch as patch_fn, post, post as post_fn, put, put as put_fn,
        trace, trace as trace_fn,
    };

    pub use crate::controller::{RouteRegistrar, WebController, WebControllerRegistrar};
}
