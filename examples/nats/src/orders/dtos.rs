use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateOrderDto {
    pub total: u64,
}

#[derive(Deserialize)]
pub struct ShipOrderDto {
    pub order_id: String,
    pub tracking: String,
}
