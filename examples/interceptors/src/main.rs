mod interceptors;
mod controllers {
    pub mod socketio;
    pub mod web;
}

use controllers::socketio::EventsController;
use controllers::web::ExampleRestController;

use sword::prelude::*;
use sword_layers::cors::*;

pub struct ExampleModule;

impl Module for ExampleModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<EventsController>();
        controllers.register::<ExampleRestController>();
    }
}

#[sword::main]
async fn main() -> Result<(), Error> {
    let config = Config::builder()
        .add_file("Config.toml")
        .build()
        .expect("Failed to load configuration");

    let cors_config = config.expect::<CorsConfig>();
    let cors_layers = CorsLayer::from(cors_config);

    let app = Application::from_config(config)?
        .with_module::<ExampleModule>()
        .with_layer(cors_layers)
        .build()?;

    app.run().await?;

    Ok(())
}
