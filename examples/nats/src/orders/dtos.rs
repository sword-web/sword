use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateOrderDto {
    pub total: u64,
}
