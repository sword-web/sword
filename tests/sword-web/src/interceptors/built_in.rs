use axum_test::{TestServer, multipart::MultipartForm};
use serde_json::Value;
use sword::prelude::*;
use sword::web::*;
use sword_layers::prelude::CompressionConfig;
use tokio::time::{Duration, sleep};

use crate::application_builder;

#[controller(kind = Controller::Web, path = "/test")]
struct TestController;

impl TestController {
    #[get("/timeout")]
    async fn timeout(&self) -> WebResult {
        sleep(Duration::from_secs(3)).await;
        Ok(JsonResponse::Ok().message("This should not be reached"))
    }

    #[get("/timeout-boundary")]
    async fn timeout_boundary(&self) -> WebResult {
        sleep(Duration::from_millis(2000)).await;
        Ok(JsonResponse::Ok().message("This should timeout"))
    }

    #[get("/timeout-just-under")]
    async fn timeout_just_under(&self) -> WebResult {
        sleep(Duration::from_millis(1900)).await;
        Ok(JsonResponse::Ok().message("This should complete"))
    }

    #[get("/timeout-just-over")]
    async fn timeout_just_over(&self) -> WebResult {
        sleep(Duration::from_millis(2100)).await;
        Ok(JsonResponse::Ok().message("This should timeout"))
    }

    #[get("/no-timeout")]
    async fn no_timeout(&self) -> WebResult {
        Ok(JsonResponse::Ok().message("Quick response"))
    }

    #[post("/content-type-json")]
    async fn content_type_json(&self, req: Request) -> WebResult {
        let _body: Value = req.body()?;
        Ok(JsonResponse::Ok().message("JSON received"))
    }

    #[post("/content-type-form")]
    async fn content_type_form(&self) -> WebResult {
        Ok(JsonResponse::Ok().message("Form data received"))
    }

    #[post("/content-type-any")]
    async fn content_type_any(&self, req: Request) -> WebResult {
        let _body: String = req.body()?;
        Ok(JsonResponse::Ok().message("Any content type"))
    }

    #[get("/no-body")]
    async fn no_body(&self) -> WebResult {
        Ok(JsonResponse::Ok().message("No body required"))
    }
}

struct V1UsersModule;

impl Module for V1UsersModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<TestController>();
    }
}

fn test_server() -> TestServer {
    let app = application_builder()
        .with_module::<V1UsersModule>()
        .with_layer(tower_http::compression::CompressionLayer::from(
            CompressionConfig {
                display: false,
                algorithms: vec![
                    "gzip".to_string(),
                    "deflate".to_string(),
                    "brotli".to_string(),
                    "zstd".to_string(),
                ],
            },
        ))
        .build();

    crate::test_server(app)
}

#[tokio::test]
async fn timeout() {
    let test_app = test_server();
    let response = test_app.get("/test/timeout").await;
    let json = response.json::<JsonResponseBody>();

    let expected = JsonResponseBody {
        code: 408,
        success: false,
        message: "Request Timeout".into(),
        data: None,
        error: None,
        errors: None,
        timestamp: json.timestamp,
        request_id: None,
    };

    assert_eq!(json.code, expected.code);
    assert_eq!(json.success, expected.success);
    assert_eq!(json.message, expected.message);
    assert_eq!(json.data, expected.data);
}

#[tokio::test]
async fn timeout_boundary_exact() {
    let test_app = test_server();
    let response = test_app.get("/test/timeout-boundary").await;

    assert_eq!(response.status_code(), 408);

    let json = response.json::<JsonResponseBody>();
    assert_eq!(json.code, 408);
    assert!(!json.success);
    assert_eq!(json.message, "Request Timeout".into());
}

#[tokio::test]
async fn timeout_just_under_limit() {
    let test_app = test_server();
    let response = test_app.get("/test/timeout-just-under").await;

    assert_eq!(response.status_code(), 200);

    let json = response.json::<JsonResponseBody>();

    assert_eq!(json.code, 200);
    assert!(json.success);
    assert!(json.message.contains("This should complete"));
}

#[tokio::test]
async fn timeout_just_over_limit() {
    let test_app = test_server();
    let response = test_app.get("/test/timeout-just-over").await;

    assert_eq!(response.status_code(), 408);

    let json = response.json::<JsonResponseBody>();

    assert_eq!(json.code, 408);
    assert!(!json.success);
    assert_eq!(json.message, "Request Timeout".into());
}

#[tokio::test]
async fn no_timeout_quick_response() {
    let test_app = test_server();
    let response = test_app.get("/test/no-timeout").await;

    assert_eq!(response.status_code(), 200);

    let json = response.json::<JsonResponseBody>();

    assert_eq!(json.code, 200);
    assert!(json.success);
    assert!(json.message.contains("Quick response"));
}

#[tokio::test]
async fn content_type_json_valid() {
    let test_app = test_server();

    let response = test_app
        .post("/test/content-type-json")
        .json(&serde_json::json!({"test": "data"}))
        .await;

    let json = response.json::<JsonResponseBody>();

    assert_eq!(response.status_code(), 200);

    assert_eq!(json.code, 200);
    assert!(json.success);
    assert!(json.message.contains("JSON received"));
}

#[tokio::test]
async fn content_type_multipart_valid() {
    let test_app = test_server();

    let response = test_app
        .post("/test/content-type-form")
        .multipart(MultipartForm::new().add_text("field", "value"))
        .await;

    assert_eq!(response.status_code(), 200);

    let json = response.json::<JsonResponseBody>();

    assert_eq!(json.code, 200);
    assert!(json.success);
    assert!(json.message.contains("Form data received"));
}

#[tokio::test]
async fn content_type_invalid() {
    let test_app = test_server();

    let response = test_app
        .post("/test/content-type-any")
        .text("plain text data")
        .await;

    assert_eq!(response.status_code(), 415);

    let json = response.json::<JsonResponseBody>();

    assert_eq!(json.code, 415);
    assert!(!json.success);
    assert!(
        json.message
            .contains("Expected Content-Type to be application/json")
    );
}

#[tokio::test]
async fn content_type_xml_invalid() {
    let test_app = test_server();

    let response = test_app
        .post("/test/content-type-any")
        .bytes(Bytes::from("<xml>data</xml>"))
        .content_type("application/xml")
        .await;

    assert_eq!(response.status_code(), 415);

    let json = response.json::<JsonResponseBody>();

    assert_eq!(json.code, 415);
    assert!(!json.success);
    assert!(
        json.message
            .contains("Expected Content-Type to be application/json")
    );
}

#[tokio::test]
async fn content_type_form_urlencoded_invalid() {
    let test_app = test_server();

    let response = test_app
        .post("/test/content-type-any")
        .bytes(Bytes::from("key=value&another=data"))
        .content_type("application/x-www-form-urlencoded")
        .await;

    assert_eq!(response.status_code(), 415);

    let json = response.json::<JsonResponseBody>();

    assert_eq!(json.code, 415);
    assert!(!json.success);
    assert!(
        json.message
            .contains("Expected Content-Type to be application/json")
    );
}

#[tokio::test]
async fn content_type_no_body_allowed() {
    let test_app = test_server();
    let response = test_app.get("/test/no-body").await;

    assert_eq!(response.status_code(), 200);

    let json = response.json::<JsonResponseBody>();

    assert_eq!(json.code, 200);
    assert!(json.success);
}

#[tokio::test]
async fn content_type_missing_header_with_body() {
    let test_app = test_server();

    let response = test_app
        .post("/test/content-type-any")
        .text("some data without content type")
        .await;

    assert_eq!(response.status_code(), 415);

    let json = response.json::<JsonResponseBody>();

    assert_eq!(json.code, 415);
    assert!(!json.success);
    assert!(
        json.message
            .contains("Expected Content-Type to be application/json")
    );
}

#[tokio::test]
async fn content_type_case_sensitivity() {
    let test_app = test_server();

    let response = test_app
        .post("/test/content-type-json")
        .bytes(Bytes::from(r#"{"test": "data"}"#))
        .content_type("Application/JSON")
        .await;

    assert_eq!(response.status_code(), 200);
}

#[tokio::test]
async fn content_type_json_with_charset() {
    let test_app = test_server();

    let response = test_app
        .post("/test/content-type-json")
        .bytes(Bytes::from(r#"{"test": "data"}"#))
        .content_type("application/json; charset=utf-8")
        .await;

    assert_eq!(response.status_code(), 200);
}

#[tokio::test]
async fn compression_gzip() {
    let test_app = test_server();

    let response = test_app
        .get("/test/no-body")
        .add_header("Accept-Encoding", "gzip")
        .await;

    assert_eq!(response.status_code(), 200);

    // Check if response has compression headers
    let headers = response.headers();
    assert!(headers.get("content-encoding").is_some());
    assert_eq!(headers.get("content-encoding").unwrap(), "gzip");
}

#[tokio::test]
async fn compression_deflate() {
    let test_app = test_server();

    let response = test_app
        .get("/test/no-body")
        .add_header("Accept-Encoding", "deflate")
        .await;

    assert_eq!(response.status_code(), 200);

    let headers = response.headers();
    assert!(headers.get("content-encoding").is_some());
    assert_eq!(headers.get("content-encoding").unwrap(), "deflate");
}

#[tokio::test]
async fn compression_brotli() {
    let test_app = test_server();

    let response = test_app
        .get("/test/no-body")
        .add_header("Accept-Encoding", "br")
        .await;

    assert_eq!(response.status_code(), 200);

    let headers = response.headers();
    assert!(headers.get("content-encoding").is_some());
    assert_eq!(headers.get("content-encoding").unwrap(), "br");
}

#[tokio::test]
async fn compression_no_preference() {
    let test_app = test_server();

    let response = test_app.get("/test/no-body").await;

    assert_eq!(response.status_code(), 200);

    // Without Accept-Encoding, should not compress
    let headers = response.headers();
    assert!(headers.get("content-encoding").is_none());
}
