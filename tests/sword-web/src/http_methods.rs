use http::Method;
use sword::prelude::*;
use sword::web::*;

use crate::application_builder;
use crate::test_server;

#[controller(kind = Controller::Web, path = "/http-methods")]
pub struct HttpMethodsController;

impl HttpMethodsController {
    #[get("/resource")]
    async fn get_resource(&self) -> String {
        "GET response".to_string()
    }

    #[head("/resource")]
    async fn head_resource(&self) -> String {
        "HEAD response".to_string()
    }

    #[options("/resource")]
    async fn options_resource(&self) -> String {
        "OPTIONS response".to_string()
    }

    #[trace("/debug")]
    async fn trace_debug(&self) -> String {
        "TRACE response".to_string()
    }
}

struct HttpMethodsModule;

impl Module for HttpMethodsModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<HttpMethodsController>();
    }
}

fn test_server_with_module() -> axum_test::TestServer {
    let app = application_builder()
        .with_module::<HttpMethodsModule>()
        .build();

    test_server(app)
}

#[tokio::test]
async fn head_route_returns_headers_without_body() {
    let test_server = test_server_with_module();

    let response = test_server
        .method(Method::HEAD, "/http-methods/resource")
        .await;

    assert_eq!(response.status_code().as_u16(), 200);

    let body = response.into_bytes();
    assert!(body.is_empty(), "HEAD response should have no body");
}

#[tokio::test]
async fn head_route_with_custom_handler() {
    let test_server = test_server_with_module();

    let response = test_server
        .method(Method::HEAD, "/http-methods/resource")
        .await;

    assert_eq!(response.status_code().as_u16(), 200);
}

#[tokio::test]
async fn options_route_returns_allowed_methods() {
    let test_server = test_server_with_module();

    let response = test_server
        .method(Method::OPTIONS, "/http-methods/resource")
        .await;

    assert_eq!(response.status_code().as_u16(), 200);
}

#[tokio::test]
async fn options_custom_response() {
    let test_server = test_server_with_module();

    let response = test_server
        .method(Method::OPTIONS, "/http-methods/resource")
        .await;

    let body = response.text();
    assert_eq!(body, "OPTIONS response");
}

#[tokio::test]
async fn trace_route_echoes_request() {
    let test_server = test_server_with_module();

    let response = test_server
        .method(Method::TRACE, "/http-methods/debug")
        .await;

    assert_eq!(response.status_code().as_u16(), 200);
}

#[tokio::test]
async fn trace_custom_response() {
    let test_server = test_server_with_module();

    let response = test_server
        .method(Method::TRACE, "/http-methods/debug")
        .await;

    let body = response.text();
    assert_eq!(body, "TRACE response");
}
