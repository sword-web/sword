use std::convert::Infallible;

use sword::prelude::*;
use sword::web::*;
use tokio_stream::{Stream, StreamExt};

use crate::application_builder;
use crate::test_server;

#[controller(kind = Controller::Web, path = "/sse")]
pub struct SseController;

impl SseController {
    #[sse("/stream")]
    async fn stream(&self) -> Sse<impl Stream<Item = Result<Event, Infallible>> + use<>> {
        let events = vec![
            Event::default().event("greeting").data("one"),
            Event::default().event("greeting").data("two"),
        ];

        Sse::new(tokio_stream::iter(events).map(Ok))
    }
}

struct SseModule;

impl Module for SseModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<SseController>();
    }
}

#[tokio::test]
async fn sse_handler_streams_events() {
    let app = application_builder().with_module::<SseModule>().build();
    let app = test_server(app);

    let response = app.get("/sse/stream").await;
    assert_eq!(response.status_code().as_u16(), 200);
    assert_eq!(response.content_type(), "text/event-stream");

    let body = response.text();
    assert!(body.contains("event: greeting"), "body: {body}");
    assert!(body.contains("data: one"), "body: {body}");
    assert!(body.contains("data: two"), "body: {body}");
}
