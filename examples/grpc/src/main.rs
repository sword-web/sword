mod shared;
mod users;

use crate::{shared::SharedModule, users::UsersModule};
use sword::prelude::*;

#[main]
async fn main() {
    tracing::info!("Starting gRPC Greeter example...");

    let config = Config::builder()
        .add_file("config/config.toml")
        .build()
        .expect("Failed to load configuration");

    let app = Application::from_config(config)
        .with_module::<SharedModule>()
        .with_module::<UsersModule>()
        .build();

    app.run().await;
}
