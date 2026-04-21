use sword::prelude::*;
use sword::socketio::*;
use sword::web::*;

#[derive(Interceptor)]
pub struct LoggingInterceptor;

impl OnRequest for LoggingInterceptor {
    async fn on_request(&self, mut req: Request) -> WebInterceptorResult {
        println!(
            "[REST] - Incoming request: ID: {} - [{}] {}",
            req.id(),
            req.method(),
            req.uri()
        );

        req.extensions.insert(format!("Request-{}", req.id()));

        req.next().await
    }
}

#[derive(Interceptor)]
pub struct AnotherInterceptor;

#[derive(Interceptor)]
pub struct SocketAuditInterceptor;

impl OnRequest for AnotherInterceptor {
    async fn on_request(&self, req: Request) -> WebInterceptorResult {
        println!("[AnotherInterceptor] - Processing request ID: {}", req.id());

        let request_id = req
            .extensions
            .get::<String>()
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string());

        println!(
            "[AnotherInterceptor] - Request ID from extensions: {}",
            request_id
        );

        req.next().await
    }
}

impl OnConnect for LoggingInterceptor {
    type Error = String;

    async fn on_connect(&self, ctx: SocketContext) -> Result<(), Self::Error> {
        println!("[SocketIO] - New connection: - Socket ID: {}", ctx.id());

        Ok(())
    }
}

impl OnConnect for SocketAuditInterceptor {
    type Error = String;

    async fn on_connect(&self, ctx: SocketContext) -> Result<(), Self::Error> {
        println!("[SocketIO] - Audit interceptor for Socket ID: {}", ctx.id());

        Ok(())
    }
}
