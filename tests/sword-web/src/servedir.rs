use axum_test::TestServer;
use serde_json::Value;

use crate::application_builder;
use crate::test_server;

fn test_app() -> TestServer {
    let app = application_builder().build().expect("application");

    test_server(app)
}

#[tokio::test]
async fn serves_a_static_file() {
    let response = test_app().get("/static/hello.txt").await;

    assert_eq!(response.status_code().as_u16(), 200);
    assert_eq!(response.text(), "hello from sword\n");
}

#[tokio::test]
async fn missing_file_serves_the_custom_page() {
    let response = test_app().get("/static/missing.txt").await;

    assert_eq!(response.status_code().as_u16(), 404);

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    assert!(content_type.starts_with("text/html"));
    assert!(response.text().contains("custom-not-found"));
}

#[tokio::test]
async fn static_method_not_allowed_returns_405() {
    let response = test_app().post("/static/hello.txt").await;

    assert_eq!(response.status_code().as_u16(), 405);

    let allow = response
        .headers()
        .get("allow")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    assert!(allow.contains("GET"));
}

#[tokio::test]
async fn unmatched_route_is_json_404_with_request_id() {
    let response = test_app().get("/definitely-not-a-route").await;

    assert_eq!(response.status_code().as_u16(), 404);
    assert!(response.headers().get("x-request-id").is_some());

    let body: Value = response.json();
    assert_eq!(body["code"], 404);
}

#[tokio::test]
async fn health_carries_a_request_id() {
    let response = test_app().get("/health").await;

    assert_eq!(response.status_code().as_u16(), 200);
    assert!(response.headers().get("x-request-id").is_some());
}

#[tokio::test]
async fn missing_custom_not_found_file_returns_404() {
    let app = sword::Application::from_config_path("servedir-missing.toml")
        .expect("application builder")
        .build()
        .expect("application");

    let response = test_server(app).get("/static/missing.txt").await;

    assert_eq!(response.status_code().as_u16(), 404);
}
