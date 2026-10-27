pub mod jobs;
pub mod mailer;
pub mod shared;
pub mod sse;
pub mod users;

use dotenv::dotenv;
use sword::prelude::*;

use crate::{
    jobs::JobsModule, mailer::MailerModule, shared::SharedModule, sse::SseModule,
    users::UsersModule,
};

#[sword::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    tracing::info!("Starting Users Management example...");

    let app = Application::builder()?
        .with_module::<SharedModule>()
        .with_module::<UsersModule>()
        .with_module::<MailerModule>()
        .with_module::<SseModule>()
        .with_module::<JobsModule>()
        .build()?;

    app.run().await?;

    Ok(())
}
