mod events;
mod handler;
mod sender;

pub use events::UserCreatedEvent;
pub use handler::MailHandler;
pub use sender::Mailer;

use sword::prelude::*;

pub struct MailerModule;

impl Module for MailerModule {
    fn register_components(components: &ComponentRegistry) {
        components.register::<Mailer>();
    }

    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<MailHandler>();
    }
}
