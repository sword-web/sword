mod controller;
mod dtos;
mod entity;
mod interceptor;

use sword::prelude::*;

use self::controller::ChatController;

pub use self::dtos::IncommingMessageDto;
pub use self::entity::Message;
pub use self::interceptor::ChatInterceptor;

pub struct ChatModule;

impl Module for ChatModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<ChatController>();
    }
}
