use sword::events::*;
use sword::prelude::*;

use crate::orders::OrderCreated;

#[controller(kind = Controller::Events, source = EventSource::Nats)]
pub struct OrderEventsHandler;

impl OrderEventsHandler {
    #[subscribe(key = "order.created", group = "orders-workers")]
    async fn on_created(&self, event: &OrderCreated) -> EventHandlerResult<()> {
        tracing::info!(
            target: "sword.example.nats",
            order_id = %event.order_id,
            total = event.total,
            "order.created received from NATS"
        );

        Ok(())
    }

    // Wildcard subscription: receives every event whose subject starts with "order.",
    // regardless of the concrete event type.
    //
    // The handler gets the event as `&dyn Event` and branches on `event.key()`,
    // casting to the concrete type with `event.cast::<T>()`.
    #[subscribe(key = "order.*")]
    async fn on_order_event(&self, event: &dyn Event) -> EventHandlerResult<()> {
        tracing::info!(
            target: "sword.example.nats",
            subject = %event.key(),
            "order event received from NATS"
        );

        let json = event.to_vec().inspect_err(|e| {
            tracing::error!(
                target: "sword.example.nats",
                subject = %event.key(),
                error = %e,
                "Failed to encode event to JSON"
            );
        })?;

        tracing::info!(
            target: "sword.example.nats",
            subject = %event.key(),
            payload = %String::from_utf8_lossy(&json),
            "order event payload"
        );

        Ok(())
    }
}
