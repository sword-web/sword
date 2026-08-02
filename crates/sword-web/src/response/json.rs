use std::collections::HashMap;

use axum::{
    Json as AxumJson,
    http::{HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response as AxumResponse},
};

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A builder for creating standardized JSON HTTP responses.
///
/// This struct provides methods for constructing JSON responses with
/// common fields like `success`, `code`, `message`, `data`, `error`, and `timestamp`.
///
/// # Example
///
/// ```rust
/// use axum_responses::JsonResponse;
/// use serde_json::json;
///
/// async fn handler() -> JsonResponse {
///     JsonResponse::Ok()
///         .message("Data sent")
///         .data(json!({ "name": "Alice" }))
/// }
/// ```
#[derive(Debug, Clone)]
pub struct JsonResponse {
    code: StatusCode,
    json: Box<Map<String, Value>>,
    headers: Option<HashMap<HeaderName, HeaderValue>>,
}

impl Default for JsonResponse {
    fn default() -> Self {
        Self::Ok()
    }
}

#[allow(non_snake_case)]
impl JsonResponse {
    /// Creates a new JSON response builder with the given status code.
    pub fn status(code: impl TryInto<StatusCode>) -> Self {
        let code = code.try_into().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let default_message = code.canonical_reason().unwrap_or("No Message");

        let json = Map::from_iter([
            ("success".into(), Value::Bool(code.is_success())),
            ("code".into(), Value::Number(code.as_u16().into())),
            ("message".into(), Value::String(default_message.into())),
        ]);

        Self {
            code,
            json: Box::new(json),
            headers: None,
        }
    }

    // ==================== Builder Methods ====================

    /// Sets the response message.
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.json
            .insert("message".into(), Value::String(message.into()));

        self
    }

    /// Sets optional Request ID for tracing purposes
    pub fn request_id(mut self, request_id: impl Into<String>) -> Self {
        self.json
            .insert("request_id".into(), Value::String(request_id.into()));

        self
    }

    /// Adds a header to the response.
    pub fn header(mut self, key: &str, value: &str) -> Self {
        if let (Ok(header_name), Ok(header_value)) =
            (HeaderName::try_from(key), HeaderValue::try_from(value))
        {
            (*self.headers.get_or_insert_with(HashMap::new)).insert(header_name, header_value);
        }
        self
    }

    /// Adds `data` field to the response.
    pub fn data<T: Serialize>(mut self, data: T) -> Self {
        let data = serde_json::to_value(data).unwrap_or_else(|err| {
            eprintln!("Warning: Failed to serialize response 'data' field: {err}");
            Value::String("Serialization failed".into())
        });

        self.json.insert("data".into(), data);
        self
    }

    /// Adds `error` field to the response.
    pub fn error<T: Serialize>(mut self, error: T) -> Self {
        let error = serde_json::to_value(error).unwrap_or_else(|err| {
            eprintln!("Warning: Failed to serialize response 'error' field: {err}");
            Value::String("Serialization failed".into())
        });

        self.json.insert("error".into(), error);
        self
    }

    /// Adds `errors` field to the response.
    pub fn errors<T: Serialize>(mut self, errors: T) -> Self {
        let errors = serde_json::to_value(errors).unwrap_or_else(|err| {
            eprintln!("Warning: Failed to serialize response 'errors' field: {err}");
            Value::String("Serialization failed".into())
        });

        self.json.insert("errors".into(), errors);
        self
    }

    /// 100 Continue
    /// [[RFC9110, Section 15.2.1](https://datatracker.ietf.org/doc/html/rfc9110#section-15.2.1)]
    pub fn Continue() -> Self {
        Self::status(StatusCode::CONTINUE)
    }

    /// 101 Switching Protocols
    /// [[RFC9110, Section 15.2.2](https://datatracker.ietf.org/doc/html/rfc9110#section-15.2.2)]
    pub fn SwitchingProtocols() -> Self {
        Self::status(StatusCode::SWITCHING_PROTOCOLS)
    }

    /// 102 Processing
    /// [[RFC2518, Section 10.1](https://datatracker.ietf.org/doc/html/rfc2518#section-10.1)]
    pub fn Processing() -> Self {
        Self::status(StatusCode::PROCESSING)
    }

    /// 200 OK
    /// [[RFC9110, Section 15.3.1](https://datatracker.ietf.org/doc/html/rfc9110#section-15.3.1)]
    pub fn Ok() -> Self {
        Self::status(StatusCode::OK)
    }

    /// 201 Created
    /// [[RFC9110, Section 15.3.2](https://datatracker.ietf.org/doc/html/rfc9110#section-15.3.2)]
    pub fn Created() -> Self {
        Self::status(StatusCode::CREATED)
    }

    /// 202 Accepted
    /// [[RFC9110, Section 15.3.3](https://datatracker.ietf.org/doc/html/rfc9110#section-15.3.3)]
    pub fn Accepted() -> Self {
        Self::status(StatusCode::ACCEPTED)
    }

    /// 203 Non-Authoritative Information
    /// [[RFC9110, Section 15.3.4](https://datatracker.ietf.org/doc/html/rfc9110#section-15.3.4)]
    pub fn NonAuthoritativeInformation() -> Self {
        Self::status(StatusCode::NON_AUTHORITATIVE_INFORMATION)
    }

    /// 204 No Content
    /// [[RFC9110, Section 15.3.5](https://datatracker.ietf.org/doc/html/rfc9110#section-15.3.5)]
    pub fn NoContent() -> Self {
        Self::status(StatusCode::NO_CONTENT)
    }

    /// 205 Reset Content
    /// [[RFC9110, Section 15.3.6](https://datatracker.ietf.org/doc/html/rfc9110#section-15.3.6)]
    pub fn ResetContent() -> Self {
        Self::status(StatusCode::RESET_CONTENT)
    }

    /// 206 Partial Content
    /// [[RFC9110, Section 15.3.7](https://datatracker.ietf.org/doc/html/rfc9110#section-15.3.7)]
    pub fn PartialContent() -> Self {
        Self::status(StatusCode::PARTIAL_CONTENT)
    }

    /// 207 Multi-Status
    /// [[RFC4918, Section 11.1](https://datatracker.ietf.org/doc/html/rfc4918#section-11.1)]
    pub fn MultiStatus() -> Self {
        Self::status(StatusCode::MULTI_STATUS)
    }

    /// 208 Already Reported
    /// [[RFC5842, Section 7.1](https://datatracker.ietf.org/doc/html/rfc5842#section-7.1)]
    pub fn AlreadyReported() -> Self {
        Self::status(StatusCode::ALREADY_REPORTED)
    }

    /// 226 IM Used
    /// [[RFC3229, Section 10.4.1](https://datatracker.ietf.org/doc/html/rfc3229#section-10.4.1)]
    pub fn ImUsed() -> Self {
        Self::status(StatusCode::IM_USED)
    }

    /// 300 Multiple Choices
    /// [[RFC9110, Section 15.4.1](https://datatracker.ietf.org/doc/html/rfc9110#section-15.4.1)]
    pub fn MultipleChoices() -> Self {
        Self::status(StatusCode::MULTIPLE_CHOICES)
    }

    #[deprecated(note = "Use `Redirect::found` instead")]
    /// 302 Found
    /// [[RFC9110, Section 15.4.3](https://datatracker.ietf.org/doc/html/rfc9110#section-15.4.3)]
    pub fn Found(location: &str) -> Self {
        Self::status(StatusCode::FOUND).header("Location", location)
    }

    #[deprecated(note = "Use `Redirect::see_other` instead")]
    /// 303 See Other
    /// [[RFC9110, Section 15.4.4](https://datatracker.ietf.org/doc/html/rfc9110#section-15.4.4)]
    pub fn SeeOther(location: &str) -> Self {
        Self::status(StatusCode::SEE_OTHER).header("Location", location)
    }

    /// 304 Not Modified
    /// [[RFC9110, Section 15.4.5](https://datatracker.ietf.org/doc/html/rfc9110#section-15.4.5)]
    pub fn NotModified() -> Self {
        Self::status(StatusCode::NOT_MODIFIED)
    }

    /// 305 Use Proxy
    /// [[RFC9110, Section 15.4.6](https://datatracker.ietf.org/doc/html/rfc9110#section-15.4.6)]
    pub fn UseProxy() -> Self {
        Self::status(StatusCode::USE_PROXY)
    }

    #[deprecated(note = "Use `Redirect::temporary` instead")]
    /// 307 Temporary Redirect
    /// [[RFC9110, Section 15.4.7](https://datatracker.ietf.org/doc/html/rfc9110#section-15.4.7)]
    pub fn TemporaryRedirect(location: &str) -> Self {
        Self::status(StatusCode::TEMPORARY_REDIRECT).header("Location", location)
    }

    #[deprecated(note = "Use `Redirect::permanent_redirect` instead")]
    /// 308 Permanent Redirect
    /// [[RFC9110, Section 15.4.8](https://datatracker.ietf.org/doc/html/rfc9110#section-15.4.8)]
    pub fn PermanentRedirect(location: &str) -> Self {
        Self::status(StatusCode::PERMANENT_REDIRECT).header("Location", location)
    }

    /// 400 Bad Request
    /// [[RFC9110, Section 15.5.1](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.1)]
    pub fn BadRequest() -> Self {
        Self::status(StatusCode::BAD_REQUEST)
    }

    /// 401 Unauthorized
    /// [[RFC9110, Section 15.5.2](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.2)]
    pub fn Unauthorized() -> Self {
        Self::status(StatusCode::UNAUTHORIZED)
    }

    /// 402 Payment Required
    /// [[RFC9110, Section 15.5.3](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.3)]
    pub fn PaymentRequired() -> Self {
        Self::status(StatusCode::PAYMENT_REQUIRED)
    }

    /// 403 Forbidden
    /// [[RFC9110, Section 15.5.4](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.4)]
    pub fn Forbidden() -> Self {
        Self::status(StatusCode::FORBIDDEN)
    }

    /// 404 Not Found
    /// [[RFC9110, Section 15.5.5](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.5)]
    pub fn NotFound() -> Self {
        Self::status(StatusCode::NOT_FOUND)
    }

    /// 405 Method Not Allowed
    /// [[RFC9110, Section 15.5.6](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.6)]
    pub fn MethodNotAllowed() -> Self {
        Self::status(StatusCode::METHOD_NOT_ALLOWED)
    }

    /// 406 Not Acceptable
    /// [[RFC9110, Section 15.5.7](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.7)]
    pub fn NotAcceptable() -> Self {
        Self::status(StatusCode::NOT_ACCEPTABLE)
    }

    /// 407 Proxy Authentication Required
    /// [[RFC9110, Section 15.5.8](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.8)]
    pub fn ProxyAuthenticationRequired() -> Self {
        Self::status(StatusCode::PROXY_AUTHENTICATION_REQUIRED)
    }

    /// 408 Request Timeout
    /// [[RFC9110, Section 15.5.9](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.9)]
    pub fn RequestTimeout() -> Self {
        Self::status(StatusCode::REQUEST_TIMEOUT)
    }

    /// 409 Conflict
    /// [[RFC9110, Section 15.5.10](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.10)]
    pub fn Conflict() -> Self {
        Self::status(StatusCode::CONFLICT)
    }

    /// 410 Gone
    /// [[RFC9110, Section 15.5.11](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.11)]
    pub fn Gone() -> Self {
        Self::status(StatusCode::GONE)
    }

    /// 411 Length Required
    /// [[RFC9110, Section 15.5.12](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.12)]
    pub fn LengthRequired() -> Self {
        Self::status(StatusCode::LENGTH_REQUIRED)
    }

    /// 412 Precondition Failed
    /// [[RFC9110, Section 15.5.13](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.13)]
    pub fn PreconditionFailed() -> Self {
        Self::status(StatusCode::PRECONDITION_FAILED)
    }

    /// 413 Payload Too Large
    /// [[RFC9110, Section 15.5.14](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.14)]
    pub fn PayloadTooLarge() -> Self {
        Self::status(StatusCode::PAYLOAD_TOO_LARGE)
    }

    /// 414 URI Too Long
    /// [[RFC9110, Section 15.5.15](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.15)]
    pub fn UriTooLong() -> Self {
        Self::status(StatusCode::URI_TOO_LONG)
    }

    /// 415 Unsupported Media Type
    /// [[RFC9110, Section 15.5.16](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.16)]
    pub fn UnsupportedMediaType() -> Self {
        Self::status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
    }

    /// 416 Range Not Satisfiable
    /// [[RFC9110, Section 15.5.17](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.17)]
    pub fn RangeNotSatisfiable() -> Self {
        Self::status(StatusCode::RANGE_NOT_SATISFIABLE)
    }

    /// 417 Expectation Failed
    /// [[RFC9110, Section 15.5.18](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.18)]
    pub fn ExpectationFailed() -> Self {
        Self::status(StatusCode::EXPECTATION_FAILED)
    }

    /// 418 I'm a teapot
    /// [curiously not registered by IANA but [RFC2324, Section 2.3.2](https://datatracker.ietf.org/doc/html/rfc2324#section-2.3.2)]
    pub fn ImATeapot() -> Self {
        Self::status(StatusCode::IM_A_TEAPOT)
    }

    /// 421 misdirected request
    /// [[rfc9110, section 15.5.20](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.20)]
    pub fn MisdirectedRequest() -> Self {
        Self::status(StatusCode::MISDIRECTED_REQUEST)
    }

    /// 422 Unprocessable Entity
    /// [[RFC9110, Section 15.5.21](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.21)]
    pub fn UnprocessableEntity() -> Self {
        Self::status(StatusCode::UNPROCESSABLE_ENTITY)
    }

    /// 423 Locked
    /// [[RFC4918, Section 11.3](https://datatracker.ietf.org/doc/html/rfc4918#section-11.3)]
    pub fn Locked() -> Self {
        Self::status(StatusCode::LOCKED)
    }

    /// 424 Failed Dependency
    /// [[RFC4918, Section 11.4](https://tools.ietf.org/html/rfc4918#section-11.4)]
    pub fn FailedDependency() -> Self {
        Self::status(StatusCode::FAILED_DEPENDENCY)
    }

    /// 425 Too early
    /// [[RFC8470, Section 5.2](https://httpwg.org/specs/rfc8470.html#status)]
    pub fn TooEarly() -> Self {
        Self::status(StatusCode::TOO_EARLY)
    }

    /// 426 Upgrade Required
    /// [[RFC9110, Section 15.5.22](https://datatracker.ietf.org/doc/html/rfc9110#section-15.5.22)]
    pub fn UpgradeRequired() -> Self {
        Self::status(StatusCode::UPGRADE_REQUIRED)
    }

    /// 428 Precondition Required
    /// [[RFC6585, Section 3](https://datatracker.ietf.org/doc/html/rfc6585#section-3)]
    pub fn PreconditionRequired() -> Self {
        Self::status(StatusCode::PRECONDITION_REQUIRED)
    }

    /// 429 Too Many Requests
    /// [[RFC6585, Section 4](https://datatracker.ietf.org/doc/html/rfc6585#section-4)]
    pub fn TooManyRequests() -> Self {
        Self::status(StatusCode::TOO_MANY_REQUESTS)
    }

    /// 431 Request Header Fields Too Large
    /// [[RFC6585, Section 5](https://datatracker.ietf.org/doc/html/rfc6585#section-5)]
    pub fn RequestHeaderFieldsTooLarge() -> Self {
        Self::status(StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE)
    }

    /// 451 Unavailable For Legal Reasons
    /// [[RFC7725, Section 3](https://tools.ietf.org/html/rfc7725#section-3)]
    pub fn UnavailableForLegalReasons() -> Self {
        Self::status(StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS)
    }

    /// 500 Internal Server Error
    /// [[RFC9110, Section 15.6.1](https://datatracker.ietf.org/doc/html/rfc9110#section-15.6.1)]
    pub fn InternalServerError() -> Self {
        Self::status(StatusCode::INTERNAL_SERVER_ERROR)
    }

    /// 501 Not Implemented
    /// [[RFC9110, Section 15.6.2](https://datatracker.ietf.org/doc/html/rfc9110#section-15.6.2)]
    pub fn NotImplemented() -> Self {
        Self::status(StatusCode::NOT_IMPLEMENTED)
    }

    /// 502 Bad Gateway
    /// [[RFC9110, Section 15.6.3](https://datatracker.ietf.org/doc/html/rfc9110#section-15.6.3)]
    pub fn BadGateway() -> Self {
        Self::status(StatusCode::BAD_GATEWAY)
    }

    /// 503 Service Unavailable
    /// [[RFC9110, Section 15.6.4](https://datatracker.ietf.org/doc/html/rfc9110#section-15.6.4)]
    pub fn ServiceUnavailable() -> Self {
        Self::status(StatusCode::SERVICE_UNAVAILABLE)
    }

    /// 504 Gateway Timeout
    /// [[RFC9110, Section 15.6.5](https://datatracker.ietf.org/doc/html/rfc9110#section-15.6.5)]
    pub fn GatewayTimeout() -> Self {
        Self::status(StatusCode::GATEWAY_TIMEOUT)
    }

    /// 505 HTTP Version Not Supported
    /// [[RFC9110, Section 15.6.6](https://datatracker.ietf.org/doc/html/rfc9110#section-15.6.6)]
    pub fn HttpVersionNotSupported() -> Self {
        Self::status(StatusCode::HTTP_VERSION_NOT_SUPPORTED)
    }

    /// 506 Variant Also Negotiates
    /// [[RFC2295, Section 8.1](https://datatracker.ietf.org/doc/html/rfc2295#section-8.1)]
    pub fn VariantAlsoNegotiates() -> Self {
        Self::status(StatusCode::VARIANT_ALSO_NEGOTIATES)
    }

    /// 507 Insufficient Storage
    /// [[RFC4918, Section 11.5](https://datatracker.ietf.org/doc/html/rfc4918#section-11.5)]
    pub fn InsufficientStorage() -> Self {
        Self::status(StatusCode::INSUFFICIENT_STORAGE)
    }

    /// 508 Loop Detected
    /// [[RFC5842, Section 7.2](https://datatracker.ietf.org/doc/html/rfc5842#section-7.2)]
    pub fn LoopDetected() -> Self {
        Self::status(StatusCode::LOOP_DETECTED)
    }

    /// 510 Not Extended
    /// [[RFC2774, Section 7](https://datatracker.ietf.org/doc/html/rfc2774#section-7)]
    pub fn NotExtended() -> Self {
        Self::status(StatusCode::NOT_EXTENDED)
    }

    /// 511 Network Authentication Required
    /// [[RFC6585, Section 6](https://datatracker.ietf.org/doc/html/rfc6585#section-6)]
    pub fn NetworkAuthenticationRequired() -> Self {
        Self::status(StatusCode::NETWORK_AUTHENTICATION_REQUIRED)
    }
}

impl IntoResponse for JsonResponse {
    fn into_response(mut self) -> AxumResponse {
        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
        self.json
            .insert("timestamp".into(), Value::String(timestamp));

        let mut response = (self.code, AxumJson(self.json)).into_response();

        if let Some(headers) = self.headers {
            for (key, value) in headers.iter() {
                response.headers_mut().insert(key, value.clone());
            }
        }

        response
    }
}

/// Represents the JSON body structure of a response.
/// Useful for testing and deserialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonResponseBody {
    pub code: u16,
    pub request_id: Option<Box<str>>,
    pub success: bool,
    pub message: Box<str>,
    pub timestamp: String,
    pub data: Option<Value>,
    pub error: Option<Value>,
    pub errors: Option<Value>,
}
