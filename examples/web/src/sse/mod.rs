mod controller;

use controller::SseController;

use sword::prelude::*;

pub struct SseModule;

impl Module for SseModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<SseController>();
    }
}
