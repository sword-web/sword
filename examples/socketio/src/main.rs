pub mod chat;
pub mod database;

use crate::{chat::ChatModule, database::Database};
use sword::prelude::*;

#[sword::main]
async fn main() {
    let database = Database::new();

    let app = Application::from_config_path("Config.toml")
        .with_module::<ChatModule>()
        .with_provider(database)
        .build();

    app.run().await;
}
