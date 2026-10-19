use sword::events::*;
use sword::prelude::*;

use crate::orders::OrderCreated;

#[controller(kind = Controller::Events, source = EventSource::Nats)]
pub struct OrderEventsHandler;

impl OrderEventsHandler {
    #[subscribe(key = "order.created", group = "orders-workers")]
    async fn on_created(&self, event: OrderCreated) -> EventHandlerResult<()> {
        tracing::info!(
            target: "sword.example.nats",
            order_id = %event.order_id,
            total = event.total,
            "order.created received from NATS"
        );

        Ok(())
    }
}
