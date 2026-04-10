use axum::body::to_bytes;
use serde_json::json;
use sword::prelude::*;
use sword::web::*;

use crate::application_builder;
use crate::test_server;

#[derive(Interceptor)]
struct StreamTagInterceptor;

impl OnRequestStream for StreamTagInterceptor {
    async fn on_request(&self, mut req: StreamRequest) -> WebInterceptorResult {
        req.extensions.insert("stream-ok".to_string());
        req.next().await
    }
}

#[derive(Interceptor)]
struct StreamConfigInterceptor;

impl OnRequestStreamWithConfig<&'static str> for StreamConfigInterceptor {
    async fn on_request(
        &self,
        config: &'static str,
        mut req: StreamRequest,
    ) -> WebInterceptorResult {
        req.extensions.insert(config.to_string());
        req.next().await
    }
}

#[controller(kind = Controller::Web, path = "/stream")]
struct StreamController;

impl StreamController {
    #[post("/echo")]
    #[interceptor(StreamTagInterceptor)]
    async fn echo(&self, req: StreamRequest) -> WebResult {
        let tag = req.extensions.get::<String>().cloned().unwrap_or_default();
        let body_limit = req.body_limit();

        let body = to_bytes(req.into_body(), body_limit).await.map_err(|_| {
            JsonResponse::InternalServerError().message("Failed to read stream body")
        })?;

        Ok(JsonResponse::Ok().data(json!({
            "len": body.len(),
            "tag": tag,
        })))
    }

    #[post("/echo-with-config")]
    #[interceptor(StreamConfigInterceptor, config = "stream-config")]
    async fn echo_with_config(&self, req: StreamRequest) -> WebResult {
        let tag = req.extensions.get::<String>().cloned().unwrap_or_default();
        let body_limit = req.body_limit();

        let body = to_bytes(req.into_body(), body_limit).await.map_err(|_| {
            JsonResponse::InternalServerError().message("Failed to read stream body")
        })?;

        Ok(JsonResponse::Ok().data(json!({
            "len": body.len(),
            "tag": tag,
        })))
    }
}

struct StreamModule;

impl Module for StreamModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<StreamController>();
    }
}

#[tokio::test]
async fn stream_route_with_interceptor_works() {
    let app = application_builder().with_module::<StreamModule>().build();
    let app = test_server(app);

    let response = app.post("/stream/echo").text("hello-stream").await;
    let body = response.json::<JsonResponseBody>();

    assert_eq!(response.status_code().as_u16(), 200);
    assert_eq!(body.data.as_ref().unwrap()["len"], 12);
    assert_eq!(body.data.as_ref().unwrap()["tag"], "stream-ok");
}

#[tokio::test]
async fn stream_route_with_config_interceptor_works() {
    let app = application_builder().with_module::<StreamModule>().build();
    let app = test_server(app);

    let response = app
        .post("/stream/echo-with-config")
        .text("hello-with-config")
        .await;

    let body = response.json::<JsonResponseBody>();

    assert_eq!(response.status_code().as_u16(), 200);
    assert_eq!(body.data.as_ref().unwrap()["len"], 17);
    assert_eq!(body.data.as_ref().unwrap()["tag"], "stream-config");
}
