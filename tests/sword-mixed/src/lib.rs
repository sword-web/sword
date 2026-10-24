//! Verifies that a web server and a gRPC server can run together in the same
//! application (they used to be mutually exclusive).

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use serial_test::serial;
    use sword::grpc::{GrpcResponse, GrpcResult, Request, async_trait};
    use sword::prelude::*;
    use sword::web::get;

    pub mod proto {
        tonic::include_proto!("ping");
    }

    use proto::ping_service_server::{PingService, PingServiceServer};
    use proto::{PingReply, PingRequest};

    // ---- gRPC controller ----------------------------------------------------

    #[controller(kind = Controller::Grpc, service = PingServiceServer)]
    struct PingController;

    #[async_trait]
    impl PingService for PingController {
        async fn ping(&self, _: Request<PingRequest>) -> GrpcResult<PingReply> {
            Ok(GrpcResponse::message(PingReply {
                message: "pong".to_string(),
            }))
        }
    }

    // ---- web controller -----------------------------------------------------

    #[controller(kind = Controller::Web, path = "/")]
    struct WebController;

    impl WebController {
        #[get("/ping")]
        async fn ping(&self) -> String {
            "pong".to_string()
        }
    }

    // ---- module -------------------------------------------------------------

    struct MixedModule;

    impl Module for MixedModule {
        fn register_controllers(controllers: &ControllerRegistry) {
            controllers.register::<PingController>();
            controllers.register::<WebController>();
        }
    }

    // ---- harness ------------------------------------------------------------

    async fn start_app() -> tokio::task::JoinHandle<()> {
        let config_path = format!("{}/config/config.toml", env!("CARGO_MANIFEST_DIR"));

        let app = Application::from_config_path(&config_path)
            .with_module::<MixedModule>()
            .build();

        let handle = tokio::spawn(async move {
            let _ = app.run().await;
        });

        // Wait until the gRPC listener accepts connections.
        for _ in 0..100 {
            if tonic::transport::Channel::from_static("http://127.0.0.1:50052")
                .connect()
                .await
                .is_ok()
            {
                return handle;
            }

            tokio::time::sleep(Duration::from_millis(30)).await;
        }

        panic!("servers did not become ready");
    }

    /// Minimal HTTP/1.1 GET so the web engine is exercised over a real socket.
    async fn http_get(addr: &str, path: &str) -> String {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let mut stream = tokio::net::TcpStream::connect(addr)
            .await
            .unwrap_or_else(|err| panic!("web server not reachable at {addr}: {err}"));

        let request = format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
        stream
            .write_all(request.as_bytes())
            .await
            .expect("write request");

        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .await
            .expect("read response");

        String::from_utf8_lossy(&response).to_string()
    }

    #[tokio::test]
    #[serial]
    async fn web_and_grpc_run_together() {
        let server = start_app().await;

        // gRPC responds.
        let mut client =
            proto::ping_service_client::PingServiceClient::connect("http://127.0.0.1:50052")
                .await
                .expect("grpc client must connect");

        let reply = client
            .ping(Request::new(PingRequest {}))
            .await
            .expect("ping should succeed")
            .into_inner();

        assert_eq!(reply.message, "pong");

        // Web responds at the same time.
        let response = http_get("127.0.0.1:8082", "/ping").await;
        assert!(
            response.contains("pong"),
            "web response did not contain the expected body: {response}"
        );

        server.abort();
    }
}
