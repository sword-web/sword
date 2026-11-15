mod jobs;

use sword::prelude::*;

use crate::jobs::JobsModule;

#[sword::main]
async fn main() -> Result<(), Error> {
    let app = Application::builder()?
        .with_module::<JobsModule>()
        .build()?;

    app.run().await
}
