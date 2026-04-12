mod controller;
mod dtos;
mod entity;
mod repository;

use controller::UsersController;

pub use dtos::{CreateUserDto, UpdateUserDto};
pub use entity::User;
pub use repository::UserRepository;

use sword::prelude::*;

pub struct UsersModule;

impl Module for UsersModule {
    fn register_components(components: &ComponentRegistry) {
        components.register::<UserRepository>();
    }

    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<UsersController>();
    }
}
