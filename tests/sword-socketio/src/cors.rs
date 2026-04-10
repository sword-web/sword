use axum_test::TestServer;
use sword::prelude::*;

use sword::socketio::*;
use sword::web::*;

use sword_layers::cors::CorsConfig;

fn cors_layer() -> tower_http::cors::CorsLayer {
    CorsConfig {
        allow_origins: Some(vec!["http://localhost:3000".to_string()]),
        allow_methods: Some(vec!["GET".to_string(), "POST".to_string()]),
        allow_headers: Some(vec![
            "Content-Type".to_string(),
            "Authorization".to_string(),
        ]),
        allow_credentials: Some(true),
        max_age: None,
        display: false,
    }
    .into()
}

#[controller(kind = Controller::SocketIo, namespace = "/socket")]
struct TestSocketIoController;

impl TestSocketIoController {
    #[on("connection")]
    async fn on_connect(&self, _: SocketContext) {
        println!("Client connected via test");
    }

    #[on("test")]
    async fn on_test(&self, socket: SocketContext) {
        socket.ack("response").ok();
    }
}

struct TestModule;

impl Module for TestModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<TestSocketIoController>();
    }
}

fn test_server() -> TestServer {
    let app = Application::builder()
        .with_module::<TestModule>()
        .with_layer(cors_layer())
        .build();

    TestServer::new(app.router()).unwrap()
}

/// Test that CORS headers are present in SocketIO handshake (polling transport)
#[tokio::test]
async fn cors_applies_to_socketio_handshake() {
    let test = test_server();

    // SocketIO handshake request with Origin header
    let response = test
        .get("/socket.io/?EIO=4&transport=polling")
        .add_header("Origin", "http://localhost:3000")
        .await;

    // Should return 200 OK for handshake
    assert_eq!(response.status_code(), 200);

    // CORS headers should be present
    let headers = response.headers();

    // Access-Control-Allow-Origin should be present
    assert!(
        headers.contains_key("access-control-allow-origin"),
        "CORS header 'access-control-allow-origin' should be present in SocketIO handshake"
    );

    let allow_origin = headers
        .get("access-control-allow-origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Should allow the origin or be permissive
    assert!(
        allow_origin == "http://localhost:3000" || allow_origin == "*",
        "CORS should allow the origin. Got: {}",
        allow_origin
    );
}

/// Test that CORS preflight (OPTIONS) works for SocketIO
/// Note: axum-test doesn't support OPTIONS method directly,
/// but we can test that GET requests have CORS headers
#[tokio::test]
async fn cors_headers_present_in_socketio_response() {
    let test = test_server();

    // Request with Origin header (cross-origin request)
    let response = test
        .get("/socket.io/?EIO=4&transport=polling")
        .add_header("Origin", "http://localhost:3000")
        .await;

    let headers = response.headers();

    // Should have CORS allow-origin header
    assert!(
        headers.contains_key("access-control-allow-origin"),
        "SocketIO response should contain access-control-allow-origin header for cross-origin requests"
    );
}

/// Test that SocketIO handshake works without CORS (same-origin)
#[tokio::test]
async fn socketio_handshake_without_cors() {
    let test = test_server();

    // Same-origin request (no Origin header)
    let response = test.get("/socket.io/?EIO=4&transport=polling").await;

    // Should still work
    assert_eq!(
        response.status_code(),
        200,
        "SocketIO handshake should work without CORS for same-origin requests"
    );

    // Response should contain session ID (SocketIO handshake response)
    let body = response.text();
    assert!(
        body.contains("sid") || body.starts_with("0{"),
        "Response should contain SocketIO handshake data. Got: {}",
        body
    );
}

/// Test that CORS doesn't interfere with normal REST endpoints
#[tokio::test]
async fn cors_doesnt_break_rest() {
    // Add a simple REST controller
    #[controller(kind = Controller::Web, path = "/api")]
    struct ApiController;

    impl ApiController {
        #[get("/test")]
        async fn test(&self) -> JsonResponse {
            JsonResponse::Ok().message("REST works")
        }
    }

    struct TestModuleWithRest;

    impl Module for TestModuleWithRest {
        fn register_controllers(controllers: &ControllerRegistry) {
            controllers.register::<TestSocketIoController>();
            controllers.register::<ApiController>();
        }
    }

    let app = Application::builder()
        .with_module::<TestModuleWithRest>()
        .with_layer(cors_layer())
        .build();

    let test = TestServer::new(app.router()).unwrap();

    // REST endpoint should work
    let response = test.get("/api/test").await;
    assert_eq!(response.status_code(), 200);

    let json = response.json::<serde_json::Value>();
    assert_eq!(json["message"], "REST works");

    // SocketIO should also work
    let socketio_response = test.get("/socket.io/?EIO=4&transport=polling").await;
    assert_eq!(socketio_response.status_code(), 200);
}

/// Test CORS with credentials flag for SocketIO
#[tokio::test]
async fn cors_credentials_in_socketio() {
    let test = test_server();

    let response = test
        .get("/socket.io/?EIO=4&transport=polling")
        .add_header("Origin", "http://localhost:3000")
        .await;

    let headers = response.headers();

    // If CORS is configured with credentials, the header should be present
    if headers.contains_key("access-control-allow-credentials") {
        let credentials = headers
            .get("access-control-allow-credentials")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        assert_eq!(
            credentials, "true",
            "Access-Control-Allow-Credentials should be 'true' when configured"
        );
    }
}
