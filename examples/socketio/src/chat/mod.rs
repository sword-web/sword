mod controller;
mod dtos;
mod entity;
mod interceptor;

use controller::ChatController;
use sword::prelude::*;

pub use dtos::IncommingMessageDto;
pub use entity::Message;
pub use interceptor::ChatInterceptor;

pub struct ChatModule;

impl Module for ChatModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<ChatController>();
    }
}
