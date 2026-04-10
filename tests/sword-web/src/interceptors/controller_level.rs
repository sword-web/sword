use serde_json::json;
use sword::prelude::*;
use sword::web::*;

use crate::application_builder;
use crate::test_server;

#[derive(Interceptor)]
struct ExtensionsTestMiddleware;

impl OnRequest for ExtensionsTestMiddleware {
    async fn on_request(&self, mut req: Request) -> WebInterceptorResult {
        req.extensions
            .insert::<String>("test_extension".to_string());

        req.next().await
    }
}

#[derive(Interceptor)]
struct MwWithState;

impl OnRequest for MwWithState {
    async fn on_request(&self, mut req: Request) -> WebInterceptorResult {
        req.extensions.insert::<u16>(8080);
        req.next().await
    }
}

#[controller(kind = Controller::Web, path = "/test")]
#[interceptor(ExtensionsTestMiddleware)]
struct TestController {}

impl TestController {
    #[get("/extensions-test")]
    async fn extensions_test(&self, req: Request) -> JsonResponse {
        let extension_value = req.extensions.get::<String>();

        JsonResponse::Ok()
            .message("Test controller response with extensions")
            .data(json!({
                "extension_value": extension_value.cloned().unwrap_or_default()
            }))
    }

    #[get("/middleware-state")]
    #[interceptor(MwWithState)]
    async fn middleware_state(&self, req: Request) -> WebResult {
        let port = req.extensions.get::<u16>().cloned().unwrap_or(0);
        let message = req.extensions.get::<String>().cloned().unwrap_or_default();

        let json = json!({
            "port": port,
            "message": message
        });

        Ok(JsonResponse::Ok()
            .message("Test controller response with middleware state")
            .data(json))
    }
}

struct TestModule;

impl Module for TestModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<TestController>();
    }
}

#[tokio::test]
async fn extensions_mw_test() {
    let app = application_builder().with_module::<TestModule>().build();
    let test = test_server(app);

    let response = test.get("/test/extensions-test").await;
    assert_eq!(response.status_code(), 200);

    let json = response.json::<JsonResponseBody>();

    assert!(json.data.is_some());

    let data = json.data.unwrap();

    assert_eq!(data["extension_value"], "test_extension");
}

#[tokio::test]
async fn middleware_state() {
    let app = application_builder().with_module::<TestModule>().build();
    let test = test_server(app);

    let response = test.get("/test/middleware-state").await;

    assert_eq!(response.status_code(), 200);

    let json = response.json::<JsonResponseBody>();

    assert!(json.data.is_some());

    let data = json.data.unwrap();

    assert_eq!(data["port"], 8080);
    assert_eq!(data["message"], "test_extension");
}
