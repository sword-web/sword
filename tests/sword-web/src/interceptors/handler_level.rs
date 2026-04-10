#![allow(unused)]

use serde_json::json;
use sword::prelude::*;
use sword::web::*;
use tower_http::cors::CorsLayer;

use crate::application_builder;
use crate::test_server;

#[derive(Debug, Clone)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub enum DatabaseConfig {
    Sqlite(String),
    Postgres { host: String, port: u16 },
    Memory,
}

#[derive(Debug, Clone)]
pub enum AuthMethod {
    Basic { username: String, password: String },
    None,
}

#[derive(Interceptor)]
pub struct FileValidationMiddleware;

impl OnRequestWithConfig<(&'static str, &'static str)> for FileValidationMiddleware {
    async fn on_request(
        &self,
        config: (&'static str, &'static str),
        mut req: Request,
    ) -> WebInterceptorResult {
        req.extensions
            .insert((config.0.to_string(), config.1.to_string()));

        req.next().await
    }
}

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

#[derive(Interceptor)]
struct RoleMiddleware;

impl OnRequestWithConfig<Vec<&'static str>> for RoleMiddleware {
    async fn on_request(&self, roles: Vec<&'static str>, mut req: Request) -> WebInterceptorResult {
        let roles_owned: Vec<String> = roles.iter().map(|s| s.to_string()).collect();
        req.extensions.insert(roles_owned);

        req.next().await
    }
}

#[derive(Interceptor)]
struct TupleConfigMiddleware;

impl OnRequestWithConfig<(&'static str, &'static str)> for TupleConfigMiddleware {
    async fn on_request(
        &self,
        config: (&'static str, &'static str),
        mut req: Request,
    ) -> WebInterceptorResult {
        req.extensions
            .insert((config.0.to_string(), config.1.to_string()));
        req.next().await
    }
}

#[derive(Interceptor)]
struct ArrayConfigMiddleware;

impl OnRequestWithConfig<[i32; 3]> for ArrayConfigMiddleware {
    async fn on_request(&self, config: [i32; 3], mut req: Request) -> WebInterceptorResult {
        req.extensions.insert(config);

        req.next().await
    }
}

#[derive(Interceptor)]
struct StringConfigMiddleware;

impl OnRequestWithConfig<String> for StringConfigMiddleware {
    async fn on_request(&self, config: String, mut req: Request) -> WebInterceptorResult {
        req.extensions.insert(config);

        req.next().await
    }
}

#[derive(Interceptor)]
struct StrConfigMiddleware;

impl OnRequestWithConfig<&'static str> for StrConfigMiddleware {
    async fn on_request(&self, config: &'static str, mut req: Request) -> WebInterceptorResult {
        req.extensions.insert(config.to_string());

        req.next().await
    }
}

#[derive(Interceptor)]
struct NumberConfigMiddleware;

impl OnRequestWithConfig<i32> for NumberConfigMiddleware {
    async fn on_request(&self, config: i32, mut req: Request) -> WebInterceptorResult {
        req.extensions.insert(config);

        req.next().await
    }
}

#[derive(Interceptor)]
struct BoolConfigMiddleware;

impl OnRequestWithConfig<bool> for BoolConfigMiddleware {
    async fn on_request(&self, config: bool, mut req: Request) -> WebInterceptorResult {
        req.extensions.insert(config);

        req.next().await
    }
}

#[derive(Interceptor)]
struct ComplexConfigMiddleware;

impl OnRequestWithConfig<(Vec<&'static str>, i32, bool)> for ComplexConfigMiddleware {
    async fn on_request(
        &self,
        config: (Vec<&'static str>, i32, bool),
        mut req: Request,
    ) -> WebInterceptorResult {
        let owned_config = (
            config
                .0
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
            config.1,
            config.2,
        );

        req.extensions.insert(owned_config);

        req.next().await
    }
}

#[derive(Interceptor)]
struct FunctionConfigMiddleware;

impl OnRequestWithConfig<Vec<String>> for FunctionConfigMiddleware {
    async fn on_request(&self, config: Vec<String>, mut req: Request) -> WebInterceptorResult {
        req.extensions.insert(config);

        req.next().await
    }
}

#[derive(Interceptor)]
struct MathConfigMiddleware;

impl OnRequestWithConfig<i32> for MathConfigMiddleware {
    async fn on_request(&self, config: i32, mut req: Request) -> WebInterceptorResult {
        req.extensions.insert(config);

        req.next().await
    }
}

#[derive(Interceptor)]
struct ConstConfigMiddleware;

impl OnRequestWithConfig<&'static str> for ConstConfigMiddleware {
    async fn on_request(&self, config: &'static str, mut req: Request) -> WebInterceptorResult {
        req.extensions.insert(config.to_string());

        req.next().await
    }
}

#[derive(Interceptor)]
struct LogMiddleware;

impl OnRequestWithConfig<LogLevel> for LogMiddleware {
    async fn on_request(&self, config: LogLevel, mut req: Request) -> WebInterceptorResult {
        req.extensions.insert(config);

        req.next().await
    }
}

#[derive(Interceptor)]
struct DatabaseMiddleware;

impl OnRequestWithConfig<DatabaseConfig> for DatabaseMiddleware {
    async fn on_request(&self, config: DatabaseConfig, mut req: Request) -> WebInterceptorResult {
        req.extensions.insert(config);

        req.next().await
    }
}

#[derive(Interceptor)]
struct AuthMiddleware;

impl OnRequestWithConfig<AuthMethod> for AuthMiddleware {
    async fn on_request(&self, config: AuthMethod, mut req: Request) -> WebInterceptorResult {
        req.extensions.insert(config);

        req.next().await
    }
}

#[derive(Interceptor)]
struct EnumOptionMiddleware;

impl OnRequestWithConfig<Option<LogLevel>> for EnumOptionMiddleware {
    async fn on_request(&self, config: Option<LogLevel>, mut req: Request) -> WebInterceptorResult {
        req.extensions.insert(config);

        req.next().await
    }
}

#[derive(Interceptor)]
struct EnumVecMiddleware;

impl OnRequestWithConfig<Vec<LogLevel>> for EnumVecMiddleware {
    async fn on_request(&self, config: Vec<LogLevel>, mut req: Request) -> WebInterceptorResult {
        req.extensions.insert(config);

        req.next().await
    }
}

#[controller(kind = Controller::Web, path = "/test")]
struct TestController {}

impl TestController {
    #[get("/extensions-test")]
    #[interceptor(ExtensionsTestMiddleware)]
    async fn extensions_test(&self, req: Request) -> JsonResponse {
        let extension_value = req.extensions.get::<String>();

        JsonResponse::Ok()
            .message("Test controller response with extensions")
            .data(json!({
                "extension_value": extension_value.cloned().unwrap_or_default()
            }))
    }

    #[get("/middleware-state")]
    #[interceptor(ExtensionsTestMiddleware)]
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

    #[get("/role-test")]
    #[interceptor(RoleMiddleware, config = vec!["admin", "user"])]
    async fn role_test(&self, req: Request) -> JsonResponse {
        let config = req
            .extensions
            .get::<Vec<String>>()
            .cloned()
            .unwrap_or_default();

        JsonResponse::Ok().data(json!({
            "roles": config
        }))
    }

    #[get("/error-test")]
    #[interceptor(FileValidationMiddleware, config = ("jpg", "png"))]
    async fn error_test(&self, req: Request) -> JsonResponse {
        let config = req
            .extensions
            .get::<(String, String)>()
            .cloned()
            .unwrap_or(("".to_string(), "".to_string()));

        JsonResponse::Ok().data(json!({
            "allowed_formats": [config.0, config.1]
        }))
    }

    #[get("/tower-middleware-test")]
    #[interceptor(CorsLayer::permissive())]
    async fn tower_middleware_test(&self) -> JsonResponse {
        JsonResponse::Ok()
            .message("Test with tower middleware")
            .data(json!({"middleware": "cors"}))
    }

    #[get("/tuple-config-test")]
    #[interceptor(TupleConfigMiddleware, config = ("jpg", "png"))]
    async fn tuple_config_test(&self, req: Request) -> JsonResponse {
        let config = req
            .extensions
            .get::<(String, String)>()
            .cloned()
            .unwrap_or(("".to_string(), "".to_string()));

        JsonResponse::Ok().message("Tuple config test").data(json!({
            "config_type": "tuple",
            "config": [config.0, config.1]
        }))
    }

    #[get("/array-config-test")]
    #[interceptor(ArrayConfigMiddleware, config = [1, 2, 3])]
    async fn array_config_test(&self, req: Request) -> JsonResponse {
        let config = req
            .extensions
            .get::<[i32; 3]>()
            .cloned()
            .unwrap_or([0, 0, 0]);

        JsonResponse::Ok().message("Array config test").data(json!({
            "config_type": "array",
            "config": config
        }))
    }

    #[get("/string-config-test")]
    #[interceptor(StringConfigMiddleware, config = "test string".to_string())]
    async fn string_config_test(&self, req: Request) -> JsonResponse {
        let config = req.extensions.get::<String>().cloned().unwrap_or_default();

        JsonResponse::Ok()
            .message("String config test")
            .data(json!({
                "config_type": "string",
                "config": config
            }))
    }

    #[get("/str-config-test")]
    #[interceptor(StrConfigMiddleware, config = "test str")]
    async fn str_config_test(&self, req: Request) -> JsonResponse {
        let config = req.extensions.get::<String>().cloned().unwrap_or_default();

        JsonResponse::Ok().message("Str config test").data(json!({
            "config_type": "str",
            "config": config
        }))
    }

    #[get("/number-config-test")]
    #[interceptor(NumberConfigMiddleware, config = 42)]
    async fn number_config_test(&self, req: Request) -> JsonResponse {
        let config = req.extensions.get::<i32>().cloned().unwrap_or(0);

        JsonResponse::Ok()
            .message("Number config test")
            .data(json!({
                "config_type": "number",
                "config": config
            }))
    }

    #[get("/bool-config-test")]
    #[interceptor(BoolConfigMiddleware, config = true)]
    async fn bool_config_test(&self, req: Request) -> JsonResponse {
        let config = req.extensions.get::<bool>().cloned().unwrap_or(false);

        JsonResponse::Ok().message("Bool config test").data(json!({
            "config_type": "bool",
            "config": config
        }))
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

#[tokio::test]
async fn role_middleware_test() {
    let app = application_builder().with_module::<TestModule>().build();
    let test = test_server(app);

    let response = test.get("/test/role-test").await;

    assert_eq!(response.status_code(), 200);

    let json = response.json::<JsonResponseBody>();
    assert!(json.data.is_some());

    let data = json.data.unwrap();
    assert_eq!(data["roles"], json!(["admin", "user"]));
}

#[tokio::test]
async fn error_middleware_test() {
    let app = application_builder().with_module::<TestModule>().build();
    let test = test_server(app);

    let response = test.get("/test/error-test").await;

    assert_eq!(response.status_code(), 200);

    let json = response.json::<JsonResponseBody>();
    assert!(json.data.is_some());

    let data = json.data.unwrap();
    assert_eq!(data["allowed_formats"], json!(["jpg", "png"]));
}

#[tokio::test]
async fn tower_middleware_test() {
    let app = application_builder().with_module::<TestModule>().build();
    let test = test_server(app);

    let response = test.get("/test/tower-middleware-test").await;

    assert_eq!(response.status_code(), 200);

    let json = response.json::<JsonResponseBody>();
    assert!(json.data.is_some());

    let data = json.data.unwrap();
    assert_eq!(data["middleware"], "cors");
}

#[tokio::test]
async fn tuple_config_middleware_test() {
    let app = application_builder().with_module::<TestModule>().build();
    let test = test_server(app);

    let response = test.get("/test/tuple-config-test").await;

    assert_eq!(response.status_code(), 200);
    let json = response.json::<JsonResponseBody>();
    let data = json.data.unwrap();
    assert_eq!(data["config_type"], "tuple");
    assert_eq!(data["config"], json!(["jpg", "png"]));
}

#[tokio::test]
async fn array_config_middleware_test() {
    let app = application_builder().with_module::<TestModule>().build();
    let test = test_server(app);

    let response = test.get("/test/array-config-test").await;

    assert_eq!(response.status_code(), 200);
    let json = response.json::<JsonResponseBody>();
    let data = json.data.unwrap();
    assert_eq!(data["config_type"], "array");
    assert_eq!(data["config"], json!([1, 2, 3]));
}

#[tokio::test]
async fn string_config_middleware_test() {
    let app = application_builder().with_module::<TestModule>().build();
    let test = test_server(app);

    let response = test.get("/test/string-config-test").await;

    assert_eq!(response.status_code(), 200);
    let json = response.json::<JsonResponseBody>();
    let data = json.data.unwrap();
    assert_eq!(data["config_type"], "string");
    assert_eq!(data["config"], "test string");
}

#[tokio::test]
async fn str_config_middleware_test() {
    let app = application_builder().with_module::<TestModule>().build();
    let test = test_server(app);

    let response = test.get("/test/str-config-test").await;

    assert_eq!(response.status_code(), 200);
    let json = response.json::<JsonResponseBody>();
    let data = json.data.unwrap();
    assert_eq!(data["config_type"], "str");
    assert_eq!(data["config"], "test str");
}

#[tokio::test]
async fn number_config_middleware_test() {
    let app = application_builder().with_module::<TestModule>().build();
    let test = test_server(app);

    let response = test.get("/test/number-config-test").await;

    assert_eq!(response.status_code(), 200);
    let json = response.json::<JsonResponseBody>();
    let data = json.data.unwrap();
    assert_eq!(data["config_type"], "number");
    assert_eq!(data["config"], 42);
}

#[tokio::test]
async fn bool_config_middleware_test() {
    let app = application_builder().with_module::<TestModule>().build();
    let test = test_server(app);

    let response = test.get("/test/bool-config-test").await;

    assert_eq!(response.status_code(), 200);
    let json = response.json::<JsonResponseBody>();
    let data = json.data.unwrap();
    assert_eq!(data["config_type"], "bool");
    assert_eq!(data["config"], true);
}
