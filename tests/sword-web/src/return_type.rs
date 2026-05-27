use serde::Serialize;
use sword::prelude::*;
use sword::web::*;

use crate::application_builder;
use crate::test_server;

#[derive(Serialize)]
struct TestUser {
    id: u32,
    name: String,
}

#[controller(kind = Controller::Web, path = "/return-type")]
pub struct ReturnTypeController;

impl ReturnTypeController {
    #[get("/serialize")]
    async fn serialize(&self) -> Result<Vec<TestUser>, JsonResponse> {
        Ok(vec![TestUser {
            id: 1,
            name: "Alice".into(),
        }])
    }

    #[get("/passthrough")]
    async fn passthrough(&self) -> Result<JsonResponse, JsonResponse> {
        Ok(JsonResponse::Ok().message("passthrough"))
    }

    #[delete("/empty")]
    async fn empty(&self) -> Result<(), JsonResponse> {
        Ok(())
    }

    #[get("/web-result")]
    async fn web_result(&self) -> WebResult<TestUser> {
        Ok(TestUser {
            id: 2,
            name: "Bob".into(),
        })
    }

    #[get("/plain")]
    async fn plain(&self) -> JsonResponse {
        JsonResponse::Ok().message("plain")
    }
}

struct ReturnTypeModule;

impl Module for ReturnTypeModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<ReturnTypeController>();
    }
}

#[tokio::test]
async fn serialize_return_type_wraps_in_json_response() {
    let app = application_builder()
        .with_module::<ReturnTypeModule>()
        .build();
    let app = test_server(app);

    let response = app.get("/return-type/serialize").await;
    assert_eq!(response.status_code().as_u16(), 200);

    let json: serde_json::Value = response.json();
    assert_eq!(json["success"], true);
    assert_eq!(json["code"], 200);
    assert_eq!(json["data"][0]["name"], "Alice");
}

#[tokio::test]
async fn passthrough_return_type_uses_direct_response() {
    let app = application_builder()
        .with_module::<ReturnTypeModule>()
        .build();
    let app = test_server(app);

    let response = app.get("/return-type/passthrough").await;
    assert_eq!(response.status_code().as_u16(), 200);

    let json: serde_json::Value = response.json();
    assert_eq!(json["message"], "passthrough");
}

#[tokio::test]
async fn empty_return_type_returns_status_without_data() {
    let app = application_builder()
        .with_module::<ReturnTypeModule>()
        .build();
    let app = test_server(app);

    let response = app.delete("/return-type/empty").await;
    assert_eq!(response.status_code().as_u16(), 200);

    let json: serde_json::Value = response.json();
    assert_eq!(json["success"], true);
    assert!(json.get("data").is_none());
}

#[tokio::test]
async fn web_result_serialize_works() {
    let app = application_builder()
        .with_module::<ReturnTypeModule>()
        .build();
    let app = test_server(app);

    let response = app.get("/return-type/web-result").await;
    assert_eq!(response.status_code().as_u16(), 200);

    let json: serde_json::Value = response.json();
    assert_eq!(json["data"]["name"], "Bob");
}

#[tokio::test]
async fn plain_non_result_handler_still_works() {
    let app = application_builder()
        .with_module::<ReturnTypeModule>()
        .build();
    let app = test_server(app);

    let response = app.get("/return-type/plain").await;
    assert_eq!(response.status_code().as_u16(), 200);

    let json: serde_json::Value = response.json();
    assert_eq!(json["message"], "plain");
}
