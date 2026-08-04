pub mod mailer;
pub mod shared;
pub mod sse;
pub mod users;

use dotenv::dotenv;
use sword::prelude::*;

use crate::{mailer::MailerModule, shared::SharedModule, sse::SseModule, users::UsersModule};

#[sword::main]
async fn main() {
    dotenv().ok();

    tracing::info!("Starting Users Management example...");

    let app = Application::builder()
        .with_module::<SharedModule>()
        .with_module::<UsersModule>()
        .with_module::<MailerModule>()
        .with_module::<SseModule>()
        .build();

    app.run().await;
}
