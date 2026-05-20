pub mod chat;
pub mod database;

use crate::{chat::ChatModule, database::Database};
use sword::prelude::*;
use sword_layers::cors::{CorsConfig, CorsLayer};

#[sword::main]
async fn main() {
    let database = Database::new();
    let config = Config::builder().add_file("Config.toml").build().unwrap();
    let cors = CorsLayer::from(config.expect::<CorsConfig>());

    let app = Application::from_config(config)
        .with_module::<ChatModule>()
        .with_provider(database)
        .with_layer(cors)
        .build();

    app.run().await;
}
