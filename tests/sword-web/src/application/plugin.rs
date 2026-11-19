use std::sync::Arc;

use sword::prelude::*;
use sword::web::*;

use crate::{application_builder, test_server};

#[injectable(provider)]
struct TestProvider {
    value: &'static str,
}

#[injectable]
struct TestComponent {
    provider: Arc<TestProvider>,
}

impl TestComponent {
    fn value(&self) -> &'static str {
        self.provider.value
    }
}

struct TestPlugin;

impl Plugin for TestPlugin {
    fn name() -> &'static str {
        "test-plugin"
    }

    fn register_components(components: &ComponentRegistry) {
        components.register::<TestComponent>();
    }

    async fn register_providers(_: &Config, providers: &ProviderRegistry) {
        std::future::ready(()).await;
        providers.register(TestProvider { value: "plugin-ok" });
    }
}

#[controller(kind = Controller::Web, path = "/plugin-test")]
struct PluginTestController {
    component: Arc<TestComponent>,
}

impl PluginTestController {
    #[get("/hello")]
    async fn hello(&self) -> JsonResponse {
        JsonResponse::Ok().message(self.component.value().to_string())
    }
}

struct PluginTestModule;

impl Module for PluginTestModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<PluginTestController>();
    }
}

#[tokio::test]
async fn plugin_registers_provider_and_component() {
    let app = application_builder()
        .with_plugin::<TestPlugin>()
        .with_module::<PluginTestModule>()
        .build()
        .expect("application");
    let test = test_server(app);

    let response = test.get("/plugin-test/hello").await;
    let json_body = response.json::<JsonResponseBody>();

    assert_eq!(response.status_code(), 200);
    assert_eq!(json_body.message.as_ref(), "plugin-ok");
}

#[injectable(provider)]
struct SharedProvider {
    value: &'static str,
}

struct OverridePlugin;

impl Plugin for OverridePlugin {
    fn name() -> &'static str {
        "override-plugin"
    }

    async fn register_providers(_: &Config, providers: &ProviderRegistry) {
        providers.register(SharedProvider { value: "plugin" });
    }
}

#[controller(kind = Controller::Web, path = "/override")]
struct OverrideController {
    provider: Arc<SharedProvider>,
}

impl OverrideController {
    #[get("/value")]
    async fn value(&self) -> JsonResponse {
        JsonResponse::Ok().message(self.provider.value.to_string())
    }
}

struct OverrideModule;

impl Module for OverrideModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<OverrideController>();
    }

    async fn register_providers(_: &Config, providers: &ProviderRegistry) {
        providers.register(SharedProvider { value: "module" });
    }
}

#[tokio::test]
async fn module_after_plugin_overrides_provider() {
    let app = application_builder()
        .with_plugin::<OverridePlugin>()
        .with_module::<OverrideModule>()
        .build()
        .expect("application");
    let test = test_server(app);

    let response = test.get("/override/value").await;
    let json_body = response.json::<JsonResponseBody>();

    assert_eq!(response.status_code(), 200);
    assert_eq!(json_body.message.as_ref(), "module");
}

#[tokio::test]
async fn plugin_after_module_overrides_provider() {
    let app = application_builder()
        .with_module::<OverrideModule>()
        .with_plugin::<OverridePlugin>()
        .build()
        .expect("application");
    let test = test_server(app);

    let response = test.get("/override/value").await;
    let json_body = response.json::<JsonResponseBody>();

    assert_eq!(response.status_code(), 200);
    assert_eq!(json_body.message.as_ref(), "plugin");
}
