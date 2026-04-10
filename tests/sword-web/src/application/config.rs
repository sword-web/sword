use serde::{Deserialize, Serialize};
use std::process::Command;

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

#[controller(kind = Controller::Web, path = "/test")]
struct TestController {
    custom_config: MyConfig,
}

impl TestController {
    #[get("/hello")]
    async fn hello(&self) -> JsonResponse {
        JsonResponse::Ok()
            .data(&self.custom_config)
            .message("Test controller response")
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
    let app = application_builder().with_module::<ConfigModule>().build();
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
