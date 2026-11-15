use sword::prelude::*;
use sword::web::*;

/// Value left by [`RequestLoggingInterceptor`] for the next interceptor in the
/// chain, or for the handler itself.
#[derive(Clone)]
pub struct RequestStamp(pub String);

/// Controller-level interceptor. It runs for every route of the controller.
#[derive(Interceptor)]
pub struct RequestLoggingInterceptor;

impl OnRequest for RequestLoggingInterceptor {
    async fn on_request(&self, mut req: Request) -> WebInterceptorResult {
        let request_id = req.id().to_string();

        tracing::info!(
            %request_id,
            method = %req.method(),
            uri = %req.uri(),
            "Incoming request"
        );

        req.extensions_mut().insert(RequestStamp(request_id));

        req.next().await
    }
}

/// Handler-level interceptor. It runs only on the routes that declare it, and
/// reads the value left in the extensions by the controller-level interceptor.
#[derive(Interceptor)]
pub struct AuditInterceptor;

impl OnRequest for AuditInterceptor {
    async fn on_request(&self, req: Request) -> WebInterceptorResult {
        let request_id = req
            .extensions()
            .get::<RequestStamp>()
            .map(|stamp| stamp.0.clone())
            .unwrap_or_else(|| "unknown".to_string());

        tracing::info!(%request_id, "Audited request");

        req.next().await
    }
}
