use sword::prelude::*;

#[injectable]
pub struct Mailer;

impl Mailer {
    pub fn send_welcome(&self, to: &str, username: &str) {
        tracing::info!(
            target: "sword.example.mailer",
            to = %to,
            subject = "Welcome to Sword!",
            body = format!("Hi {},\n\nWelcome to Sword! We're excited to have you on board.\n\nBest,\nThe Sword Team", username),
            "Email sent successfully"
        );
    }
}
