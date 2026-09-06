use crate::mailer::{Mailer, events::UserCreatedEvent};

use std::sync::Arc;
use sword::events::*;
use sword::prelude::*;

#[controller(kind = Controller::EventHandler, source = EventSource::Memory)]
pub struct MailHandler {
    mailer: Arc<Mailer>,
}

impl MailHandler {
    #[handle("user.created")]
    async fn on_user_created(&self, event: UserCreatedEvent) -> EventHandlerResult<()> {
        tracing::info!(
            target: "sword.example.mailer",
            user_id = %event.user_id,
            username = %event.username,
            email = %event.email,
            "Sending welcome email to user"
        );

        self.mailer.send_welcome(&event.email, &event.username);

        Ok(())
    }
}
