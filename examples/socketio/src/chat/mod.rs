mod controller;
mod dtos;
mod entity;

use controller::ChatController;
use sword::prelude::*;

pub use dtos::IncommingMessageDto;
pub use entity::Message;

pub struct ChatModule;

impl Module for ChatModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<ChatController>();
    }
}
