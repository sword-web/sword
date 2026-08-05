use async_stream::stream;
use std::time::Duration;
use tokio::time::sleep;

use sword::prelude::*;
use sword::web::*;

#[controller(kind = Controller::Web, path = "/sse")]
pub struct SseController;

impl SseController {
    #[sse("/countdown")]
    async fn countdown(&self) -> Sse<impl EventStream + use<>> {
        let events = stream! {
            for i in (1..=5).rev() {
                sleep(Duration::from_millis(250)).await;
                yield Ok(Event::default().event("countdown").data(i.to_string()));
            }

            yield Ok(Event::default().event("done").data("The countdown has finished!"));
        };

        Sse::new(events)
    }
}
