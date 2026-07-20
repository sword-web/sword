use std::sync::Arc;
use sword::events::EventHandlerResult;
use sword::prelude::*;

use crate::mailer::{Mailer, events::UserCreatedEvent};

#[controller(kind = Controller::MemEventHandler, namespace = "user")]
pub struct MailHandler {
    mailer: Arc<Mailer>,
}

impl MailHandler {
    #[handle("created")]
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
