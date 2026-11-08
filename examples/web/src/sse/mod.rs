mod controller;

use sword::prelude::*;

use self::controller::SseController;

pub struct SseModule;

impl Module for SseModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<SseController>();
    }
}
