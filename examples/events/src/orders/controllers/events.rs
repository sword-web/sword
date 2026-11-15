use sword::events::*;
use sword::prelude::*;

use crate::orders::{OrderCreated, OrderShipped};

/// One controller can handle events from several transports. When it does, the
/// transport is declared per handler with `#[subscribe(transport = ...)]`
/// instead of on the controller.
#[controller(kind = Controller::Events)]
pub struct OrderEventsHandler;

impl OrderEventsHandler {
    /// In-memory handler: the event never leaves the process.
    #[subscribe(key = "order.created", transport = EventTransport::Memory)]
    async fn on_created(&self, event: &OrderCreated) -> EventHandlerResult<()> {
        tracing::info!(
            target: "events.memory",
            order_id = %event.order_id,
            total = event.total,
            "order.created handled in memory"
        );

        Ok(())
    }

    /// NATS handler: the event travels through the broker, so any other
    /// instance subscribed to the same subject receives it. The queue group
    /// makes the broker deliver it to a single instance instead of all of them.
    #[subscribe(
        key = "order.shipped",
        group = "orders-workers",
        transport = EventTransport::Nats
    )]
    async fn on_shipped(&self, event: &OrderShipped) -> EventHandlerResult<()> {
        tracing::info!(
            target: "events.nats",
            order_id = %event.order_id,
            tracking = %event.tracking,
            "order.shipped handled through NATS"
        );

        Ok(())
    }

    /// Wildcards belong to the broker: this handler receives every subject that
    /// starts with `order.`, as `&dyn Event`, and can branch on the key.
    #[subscribe(key = "order.*", transport = EventTransport::Nats)]
    async fn on_any_order(&self, event: &dyn Event) -> EventHandlerResult<()> {
        tracing::info!(
            target: "events.nats",
            subject = %event.key(),
            "order.* received from NATS"
        );

        Ok(())
    }
}
