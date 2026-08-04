use std::time::Duration;

use async_stream::stream;
use sword::prelude::*;
use sword::web::*;

#[controller(kind = Controller::Web, path = "/sse")]
pub struct SseController;

impl SseController {
    #[sse("/countdown")]
    async fn countdown(&self) -> SseResult {
        let events = stream! {
            for i in (1..=5).rev() {
                tokio::time::sleep(Duration::from_millis(250)).await;
                yield Ok(Event::default().event("countdown").data(i.to_string()));
            }
            yield Ok(Event::default().event("done").data("lift off!"));
        };

        Sse::new(Box::pin(events))
    }
}
