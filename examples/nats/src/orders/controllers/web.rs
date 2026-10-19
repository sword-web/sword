use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use sword::events::*;
use sword::prelude::*;
use sword::web::*;

use crate::orders::{CreateOrderDto, OrderCreated};

static NEXT_ORDER_ID: AtomicU64 = AtomicU64::new(1);

fn next_order_id() -> String {
    format!("order-{}", NEXT_ORDER_ID.fetch_add(1, Ordering::Relaxed))
}

#[controller(kind = Controller::Web, path = "/orders")]
pub struct OrdersController {
    publisher: Arc<NatsEventPublisher>,
}

impl OrdersController {
    #[post("/")]
    async fn create(&self, req: Request) -> WebResult {
        let body = req.body::<CreateOrderDto>()?;

        let event = OrderCreated {
            order_id: next_order_id(),
            total: body.total,
        };

        self.publisher.publish(event).await.map_err(|error| {
            tracing::error!(%error, "failed to publish order.created to NATS");
            JsonResponse::InternalServerError().message("failed to publish event")
        })?;

        Ok(JsonResponse::Created().message("order created"))
    }
}
