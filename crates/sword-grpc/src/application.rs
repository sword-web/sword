use crate::config::GrpcApplicationConfig;
use crate::controller::GrpcControllerRegistrar;
use crate::registry::GrpcServiceRegistry;

use std::collections::HashMap;
use std::net::SocketAddr;
use sword_core::{
    ApplicationConfig, Controller, ControllerMap, EngineBuildContext, State, shutdown_signal,
    sword_error,
};

use sword_layers::{DisplayConfig, body_limit::GrpcBodyLimitValue};

pub struct GrpcApplication {
    pub state: State,
    pub config: GrpcApplicationConfig,
    pub graceful_shutdown: bool,
    pub controllers: ControllerMap,
}

impl GrpcApplication {
    pub async fn start(&self) {
        let bind = format!("{}:{}", self.config.host, self.config.port);

        tracing::info!(
            target: "sword.startup.grpc",
            bind,
            graceful_shutdown = self.graceful_shutdown,
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

        #[cfg(feature = "reflection")]
        let mut reflection_descriptor_sets: Vec<&'static [u8]> = Vec::new();

        let body_limit = self.config.body_limit.clone();

        body_limit.display();

        self.state.insert(GrpcBodyLimitValue::from(body_limit));

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

            #[cfg(feature = "reflection")]
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

        #[cfg(feature = "reflection")]
        let reflection_service = {
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
                    hints: ["Ensure build.rs generates `sword_descriptor_set.bin` when reflection support is enabled"],
                }
            }))
        };

        let server = tonic::transport::Server::builder().add_routes(routes);
        let router = server.add_service(health_service);

        #[cfg(feature = "reflection")]
        let router = if let Some(reflection_service) = reflection_service {
            router.add_service(reflection_service)
        } else {
            router
        };

        if self.graceful_shutdown {
            router
                .serve_with_shutdown(bind_addr, shutdown_signal())
                .await
                .unwrap_or_else(|err| {
                    sword_error! {
                        title: "gRPC server stopped with an internal error",
                        reason: err,
                        context: {
                            "mode" => "graceful_shutdown",
                            "host" => self.config.host.clone(),
                            "port" => self.config.port.to_string(),
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
                    "host" => self.config.host.clone(),
                    "port" => self.config.port.to_string(),
                },
            }
        });
    }
}

impl From<EngineBuildContext> for GrpcApplication {
    fn from(ctx: EngineBuildContext) -> Self {
        let EngineBuildContext {
            state,
            config,
            controllers,
            ..
        } = ctx;

        let app_config = config.get_or_default::<ApplicationConfig>();
        let grpc_config = config.get_or_default::<GrpcApplicationConfig>();

        GrpcApplication {
            state,
            config: grpc_config,
            graceful_shutdown: app_config.graceful_shutdown,
            controllers: controllers.snapshot(),
        }
    }
}
