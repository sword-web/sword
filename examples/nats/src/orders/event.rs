use serde::{Deserialize, Serialize};
use sword::events::*;

#[event(key = "order.created")]
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderCreated {
    pub order_id: String,
    pub total: u64,
}

#[event(key = "order.shipped")]
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderShipped {
    pub order_id: String,
    pub tracking: String,
}
