pub mod orders;

use sword::prelude::*;

use crate::orders::OrdersModule;

#[sword::main]
async fn main() -> Result<(), Error> {
    tracing::info!("Starting NATS events example...");

    let app = Application::builder()?
        .with_module::<OrdersModule>()
        .build()?;

    app.run().await?;

    Ok(())
}
