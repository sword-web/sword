pub mod chat;
pub mod database;

use sword::prelude::*;

use crate::{chat::ChatModule, database::Database};

#[sword::main]
async fn main() -> Result<(), Error> {
    let database = Database::new();

    let app = Application::builder()?
        .with_module::<ChatModule>()
        .with_provider(database)
        .build()?;

    app.run().await?;

    Ok(())
}
