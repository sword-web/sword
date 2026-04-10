use crate::config::GrpcApplicationConfig;
use crate::controller::GrpcControllerRegistrar;
use crate::registry::GrpcServiceRegistry;

use std::collections::HashMap;
use std::net::SocketAddr;
use sword_core::{Config, Controller, ControllerMap, ControllerRegistry, State, sword_error};
use sword_layers::{DisplayConfig, body_limit::GrpcBodyLimitValue};

pub struct GrpcApplication {
    state: State,
    app_config: GrpcApplicationConfig,
    graceful_shutdown: bool,
    controllers: ControllerMap,
}

impl GrpcApplication {
    pub fn new(
        state: State,
        _config: &Config,
        grpc_config: GrpcApplicationConfig,
        graceful_shutdown: bool,
        controllers: &ControllerRegistry,
    ) -> Self {
        Self {
            state,
            app_config: grpc_config,
            graceful_shutdown,
            controllers: controllers.snapshot(),
        }
    }

    pub async fn start(&self) {
        let bind = format!("{}:{}", self.app_config.host, self.app_config.port);

        tracing::info!(
            target: "sword.startup.grpc",
            bind,
            "Starting gRPC application listener"
        );

        let grpc_ids = self
            .controllers
            .get(&Controller::Grpc)
            .filter(|ids| !ids.is_empty())
            .unwrap_or_else(|| {
                sword_error! {
                    title: "No gRPC controllers registered",
                    reason: "At least one gRPC controller must be registered before starting the server",
                    context: {
                        "bind" => bind,
                        "source" => "GrpcApplication::start",
                    },
                    hints: ["Register a controller with `controllers.register::<MyGrpcController>()` in your module"],
                }
            });

        let bind_addr: SocketAddr = bind.parse::<SocketAddr>().unwrap_or_else(|err| {
            sword_error! {
                title: "Invalid gRPC bind address",
                reason: err,
                context: {
                    "bind" => bind,
                    "source" => "GrpcApplication::start",
                },
                hints: ["Ensure host and port values are valid"],
            }
        });

        let mut registrars: HashMap<_, _> = HashMap::new();

        for registrar in inventory::iter::<GrpcControllerRegistrar>() {
            registrars.insert(registrar.controller_id, registrar);
        }

        let mut grpc_registry = GrpcServiceRegistry::new();
        let mut reflection_descriptor_sets: Vec<&'static [u8]> = Vec::new();

        let body_limit = self.app_config.body_limit.clone();

        body_limit.display();

        self.state.insert(GrpcBodyLimitValue {
            max_decoding_message_size: body_limit.max_decoding_message_size.parsed,
            max_encoding_message_size: body_limit.max_encoding_message_size.parsed,
        });

        for controller_id in grpc_ids {
            let registrar = registrars.get(controller_id).copied().unwrap_or_else(|| {
                sword_error! {
                    title: "Controller metadata not found",
                    reason: "No GrpcControllerRegistrar entry was found for controller",
                    context: {
                        "controller_id" => format!("{controller_id:?}"),
                        "source" => "GrpcApplication::start",
                    },
                    hints: ["This usually indicates a controller macro expansion issue"],
                }
            });

            (registrar.build)(&self.state);
            (registrar.register)(&self.state, &mut grpc_registry);

            if let Some(descriptor) = registrar.reflection_descriptor_set {
                reflection_descriptor_sets.push(descriptor);
            }
        }

        if grpc_registry.services_count() == 0 {
            sword_error! {
                title: "No gRPC services were registered",
                reason: "Controllers were discovered but no tonic services were added to routes",
                context: {
                    "controllers_count" => grpc_ids.len().to_string(),
                    "source" => "GrpcApplication::start",
                },
                hints: ["Implement generated register hooks to add tonic services into GrpcServiceRegistry"],
            }
        }

        let service_names: Vec<&'static str> = grpc_registry.service_names().collect();
        let routes = grpc_registry.into_routes();

        let (health_reporter, health_service) = tonic_health::server::health_reporter();

        health_reporter
            .set_service_status("", tonic_health::ServingStatus::Serving)
            .await;

        for service_name in service_names {
            health_reporter
                .set_service_status(service_name, tonic_health::ServingStatus::Serving)
                .await;
        }

        let reflection_service = if self.app_config.enable_tonic_reflection {
            let mut reflection_builder = tonic_reflection::server::Builder::configure()
                .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET);

            for descriptor in reflection_descriptor_sets {
                reflection_builder =
                    reflection_builder.register_encoded_file_descriptor_set(descriptor);
            }

            Some(reflection_builder.build_v1().unwrap_or_else(|err| {
                sword_error! {
                    title: "Failed to build tonic reflection service",
                    reason: err,
                    context: {
                        "source" => "GrpcApplication::start",
                    },
                    hints: ["Ensure build.rs generates `sword_descriptor_set.bin` when enable-tonic-reflection is true"],
                }
            }))
        } else {
            None
        };

        let server = tonic::transport::Server::builder().add_routes(routes);
        let mut router = server.add_service(health_service);

        if let Some(reflection_service) = reflection_service {
            router = router.add_service(reflection_service);
        }

        if self.graceful_shutdown {
            router
                .serve_with_shutdown(bind_addr, Self::graceful_signal())
                .await
                .unwrap_or_else(|err| {
                    sword_error! {
                        title: "gRPC server stopped with an internal error",
                        reason: err,
                        context: {
                            "mode" => "graceful_shutdown",
                            "host" => self.app_config.host.clone(),
                            "port" => self.app_config.port.to_string(),
                        },
                    }
                });

            return;
        }

        router.serve(bind_addr).await.unwrap_or_else(|err| {
            sword_error! {
                title: "gRPC server stopped with an internal error",
                reason: err,
                context: {
                    "mode" => "normal",
                    "host" => self.app_config.host.clone(),
                    "port" => self.app_config.port.to_string(),
                },
            }
        });
    }

    async fn graceful_signal() {
        let ctrl_c = async {
            tokio::signal::ctrl_c().await.unwrap_or_else(|err| {
                sword_error! {
                    title: "Failed to install Ctrl+C handler",
                    reason: err,
                    context: {
                        "source" => "GrpcApplication::graceful_signal",
                    },
                }
            });
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .unwrap_or_else(|err| {
                    sword_error! {
                        title: "Failed to install SIGTERM handler",
                        reason: err,
                        context: {
                            "signal" => "SIGTERM",
                            "source" => "GrpcApplication::graceful_signal",
                        },
                    }
                })
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                tracing::info!(
                    target: "sword.startup.signal",
                    signal = "SIGINT",
                    "Shutdown signal received, starting graceful shutdown"
                );
            },
            _ = terminate => {
                tracing::info!(
                    target: "sword.startup.signal",
                    signal = "SIGTERM",
                    "Shutdown signal received, starting graceful shutdown"
                );
            },
        }
    }
}
