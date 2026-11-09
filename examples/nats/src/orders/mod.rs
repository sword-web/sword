mod dtos;
mod event;

mod controllers {
    mod events;
    mod web;

    pub use self::events::OrderEventsHandler;
    pub use self::web::OrdersController;
}

use sword::prelude::*;

use self::controllers::{OrderEventsHandler, OrdersController};

pub use self::dtos::{CreateOrderDto, ShipOrderDto};
pub use self::event::{OrderCreated, OrderShipped};

pub struct OrdersModule;

impl Module for OrdersModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<OrdersController>();
        controllers.register::<OrderEventsHandler>();
    }
}
