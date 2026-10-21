use serde::{Deserialize, Serialize};
use sword::events::*;

#[event(key = "order.created")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderCreated {
    pub order_id: String,
    pub total: u64,
}
