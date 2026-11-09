mod controller;
mod dtos;
mod entity;
mod repository;

use sword::prelude::*;

use self::controller::UsersController;

pub use self::dtos::{CreateUserDto, UpdateUserDto};
pub use self::entity::User;
pub use self::repository::UserRepository;

pub struct UsersModule;

impl Module for UsersModule {
    fn register_components(components: &ComponentRegistry) {
        components.register::<UserRepository>();
    }

    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<UsersController>();
    }
}
