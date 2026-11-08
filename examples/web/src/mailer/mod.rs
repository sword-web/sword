mod events;
mod handler;
mod sender;

use sword::prelude::*;

pub use self::events::UserCreatedEvent;
pub use self::handler::MailHandler;
pub use self::sender::Mailer;

pub struct MailerModule;

impl Module for MailerModule {
    fn register_components(components: &ComponentRegistry) {
        components.register::<Mailer>();
    }

    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<MailHandler>();
    }
}
