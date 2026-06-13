use sword::prelude::*;
use sword::socketio::*;

#[derive(Interceptor)]
pub struct ChatInterceptor;

impl OnConnect for ChatInterceptor {
    type Error = String;

    async fn on_connect(&self, socket: SocketContext<LocalAdapter>) -> Result<(), Self::Error> {
        let Some(cookies) = socket.cookies() else {
            return Err("Missing cookies on connect".into());
        };

        let Some(example_cookie) = cookies.get("CHAT_EXAMPLE_COOKIE") else {
            return Err("Missing example cookie".into());
        };

        tracing::debug!(value = example_cookie.value(), "Received cookie on connect");

        Ok(())
    }
}
