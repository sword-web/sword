use serde_json::{Value, json};
use sword::prelude::*;
use sword::web::*;
use thiserror::Error;

use crate::application_builder;
use crate::test_server;

#[derive(Debug, Error, HttpError)]
#[http_error(code = 500, tracing = error)]
enum TestWebError {
    #[error("Not found")]
    #[http(code = 404, message = "Resource missing", tracing = info)]
    NotFound,

    #[error("Conflict: {field}")]
    #[http(code = 409, message = client_message, error = detail)]
    Conflict {
        client_message: String,
        field: String,
        detail: Value,
    },
}

#[controller(kind = Controller::Web, path = "/test-errors")]
pub struct TestErrorController;

impl TestErrorController {
    #[get("/not-found")]
    async fn not_found(&self) -> WebResult {
        Err(TestWebError::NotFound)?
    }

    #[get("/conflict")]
    async fn conflict(&self) -> WebResult {
        Err(TestWebError::Conflict {
            client_message: "Conflict occurred".into(),
            field: "username".into(),
            detail: json!({"reason": "username already taken"}),
        })?
    }
}

struct TestErrorModule;

impl Module for TestErrorModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<TestErrorController>();
    }
}

#[tokio::test]
async fn web_error_route_not_found_returns_json_response() {
    let app = application_builder()
        .with_module::<TestErrorModule>()
        .build();

    let app = test_server(app);

    let response = app.get("/test-errors/not-found").await;

    assert_eq!(response.status_code().as_u16(), 404);

    let json: Value = response.json();

    dbg!(&json);

    assert_eq!(json["message"], "Resource missing");
    assert!(json.get("error").is_none());
}

#[tokio::test]
async fn web_error_route_conflict_returns_error_field() {
    let app = application_builder()
        .with_module::<TestErrorModule>()
        .build();
    let app = test_server(app);

    let response = app.get("/test-errors/conflict").await;

    assert_eq!(response.status_code().as_u16(), 409);

    let json: Value = response.json();
    assert_eq!(json["message"], "Conflict occurred");
    assert_eq!(json["error"]["reason"], "username already taken");
}
