mod dtos;
mod event;

mod controllers {
    mod events;
    mod web;

    pub use events::OrderEventsHandler;
    pub use web::OrdersController;
}

use controllers::{OrderEventsHandler, OrdersController};
pub use dtos::CreateOrderDto;
pub use event::OrderCreated;

use sword::prelude::*;

pub struct OrdersModule;

impl Module for OrdersModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<OrdersController>();
        controllers.register::<OrderEventsHandler>();
    }
}
