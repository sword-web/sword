pub mod mailer;
pub mod shared;
pub mod users;

use dotenv::dotenv;
use sword::prelude::*;

use crate::{mailer::MailerModule, shared::SharedModule, users::UsersModule};

#[sword::main]
async fn main() {
    dotenv().ok();

    tracing::info!("Starting Users Management example...");

    let app = Application::builder()
        .with_module::<SharedModule>()
        .with_module::<UsersModule>()
        .with_module::<MailerModule>()
        .build();

    app.run().await;
}
