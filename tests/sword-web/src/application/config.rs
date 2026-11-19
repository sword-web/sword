use std::process::Command;

use serde::{Deserialize, Serialize};

use sword::prelude::*;
use sword::web::*;

use crate::application_builder;
use crate::test_server;

#[config(key = "my-custom-section")]
#[derive(Clone, Serialize, Deserialize)]
struct MyConfig {
    custom_key: String,
    env_user: String,
}

#[config(key = "optional-section", required = false)]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct OptionalConfig {
    value: String,
}

impl Default for OptionalConfig {
    fn default() -> Self {
        Self {
            value: "default".to_string(),
        }
    }
}

#[config(key = "optional-invalid-section", required = false)]
#[derive(Clone, Debug, Serialize, Deserialize)]
struct InvalidOptionalConfig {
    value: String,
}

impl Default for InvalidOptionalConfig {
    fn default() -> Self {
        Self {
            value: "default".to_string(),
        }
    }
}

#[controller(kind = Controller::Web, path = "/test")]
struct TestController {
    custom_config: MyConfig,
    optional_config: OptionalConfig,
    invalid_optional_config: InvalidOptionalConfig,
}

impl TestController {
    #[get("/hello")]
    async fn hello(&self) -> JsonResponse {
        JsonResponse::Ok()
            .data(&self.custom_config)
            .message("Test controller response")
    }

    #[get("/optional")]
    async fn optional(&self) -> JsonResponse {
        JsonResponse::Ok().data(&self.optional_config)
    }

    #[get("/invalid-optional")]
    async fn invalid_optional(&self) -> JsonResponse {
        JsonResponse::Ok().data(&self.invalid_optional_config)
    }
}

struct ConfigModule;

impl Module for ConfigModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<TestController>();
    }
}

#[tokio::test]
async fn test_application() {
    let app = application_builder()
        .with_module::<ConfigModule>()
        .build()
        .expect("application");
    let test = test_server(app);

    let response = test.get("/test/hello").await;
    let json_body = response.json::<JsonResponseBody>();

    assert_eq!(response.status_code(), 200);
    assert!(json_body.data.is_some());

    let data = json_body.data.unwrap();

    let expected = MyConfig {
        custom_key: "value".to_string(),
        env_user: Command::new("sh")
            .arg("-c")
            .arg("echo $USER")
            .output()
            .expect("Failed to get environment variable")
            .stdout
            .into_iter()
            .map(|b| b as char)
            .collect::<String>()
            .trim()
            .to_string(),
    };

    assert_eq!(data["custom_key"], expected.custom_key);
}

#[tokio::test]
async fn test_optional_config_uses_default() {
    let app = application_builder()
        .with_module::<ConfigModule>()
        .build()
        .expect("application");
    let test = test_server(app);

    let response = test.get("/test/optional").await;
    let json_body = response.json::<JsonResponseBody>();

    assert_eq!(response.status_code(), 200);

    let data = json_body.data.expect("optional config data");
    assert_eq!(data["value"], "default");
}

#[tokio::test]
async fn test_optional_config_uses_default_on_invalid_section() {
    let config = Config::builder()
        .add_required_file("Config.toml")
        .add_toml_str("[optional-invalid-section]\nvalue = 123")
        .build()
        .expect("config");
    let app = Application::from_config(config)
        .expect("application builder")
        .with_module::<ConfigModule>()
        .build()
        .expect("application");
    let test = test_server(app);

    let response = test.get("/test/invalid-optional").await;
    let json_body = response.json::<JsonResponseBody>();

    assert_eq!(response.status_code(), 200);

    let data = json_body.data.expect("optional config data");
    assert_eq!(data["value"], "default");
}
