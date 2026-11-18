use axum_test::TestServer;
use serde_json::Value;

use sword::prelude::*;

fn test_app() -> TestServer {
    let app = Application::builder()
        .expect("application builder")
        .build()
        .expect("application");

    TestServer::new(app.router())
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
async fn socketio_handshake_is_served_with_request_id() {
    let response = test_app().get("/socket.io/?EIO=4&transport=polling").await;

    assert_eq!(response.status_code().as_u16(), 200);
    assert!(response.headers().get("x-request-id").is_some());
    assert!(response.text().starts_with("0{"));
}
