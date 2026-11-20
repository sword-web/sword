mod controller;
mod dtos;
mod entity;
mod repository;
mod service;

pub mod proto {
    tonic::include_proto!("users");
    pub use user_service_server::*;
}

use sword::prelude::*;

use self::controller::UsersController;

pub use self::dtos::{CreateUserDto, UpdateUserDto};
pub use self::entity::{User, UserView};
pub use self::repository::UserRepository;
pub use self::service::UserService;

pub struct UsersModule;

impl Module for UsersModule {
    fn register_components(components: &ComponentRegistry) {
        components.register::<UserRepository>();
        components.register::<UserService>();
    }

    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<UsersController>();
    }
}
