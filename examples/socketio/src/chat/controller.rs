use std::sync::Arc;
use sword::prelude::*;
use sword::socketio::*;

use crate::{
    chat::{IncommingMessageDto, Message},
    database::Database,
};

#[controller(kind = Controller::SocketIo, namespace = "/chat")]
pub struct ChatController {
    db: Arc<Database>,
}

impl ChatController {
    #[on("connection")]
    async fn on_connect(&self, socket: SocketContext) {
        println!("New client connected");

        let messages = self.db.get_all().await;

        socket.emit("messages", &messages).ok();
    }

    #[on("message")]
    async fn handle_message(&self, socket: SocketContext) {
        let Ok(data) = socket.try_validated_data::<IncommingMessageDto>() else {
            eprintln!("Failed to validate message data");
            return;
        };

        self.db.set(Message::from(data)).await;

        let messages = self.db.get_all().await;

        socket.emit("messages", &messages).ok();
        socket.broadcast().emit("messages", &messages).await.ok();
    }
}
