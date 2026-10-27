use crate::config::GrpcApplicationConfig;
use crate::controller::GrpcControllerRegistrar;
use crate::logger::{GrpcLoggerConfig, GrpcLoggerLayer};
use crate::registry::GrpcServiceRegistry;

use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use sword_core::diagnostic::{DiagnosticPhase, Error};
use sword_core::{
    ApplicationConfig, ConfigSection, Controller, EngineBuildContext, Shutdown, State, sword_error,
};

use sword_layers::{DisplayConfig, body_limit::GrpcBodyLimitValue, request_id::RequestIdLayer};

pub struct GrpcApplication {
    pub state: State,
    pub config: GrpcApplicationConfig,
    pub graceful_shutdown: bool,
    pub controller_ids: HashSet<TypeId>,
}

impl GrpcApplication {
    pub async fn run(self, shutdown: Shutdown) -> Result<(), Error> {
        let bind = format!("{}:{}", self.config.host, self.config.port);

        tracing::info!(
            target: "sword.startup.grpc",
            bind,
            graceful_shutdown = self.graceful_shutdown,
            "Starting gRPC application listener"
        );

        // Controllers are discovered at runtime, so an empty set is only caught here.
        if self.controller_ids.is_empty() {
            return Err(sword_error! {
                code: "sword.runtime.grpc",
                phase: DiagnosticPhase::Runtime,
                target: "gRPC controller registry",
                title: "No gRPC controllers registered",
                reason: "The server has nothing to serve",
                context: {
                    "bind" => bind,
                },
                hints: ["Register at least one controller with `controllers.register::<MyGrpcController>()`"],
            });
        }

        let bind_addr = bind.parse::<SocketAddr>().map_err(|err| {
            sword_error! {
                code: "sword.runtime.grpc",
                phase: DiagnosticPhase::Runtime,
                target: "GrpcApplication::run",
                title: "Invalid gRPC bind address",
                reason: err,
                context: {
                    "bind" => bind,
                },
                hints: ["Ensure host and port values are valid"],
            }
        })?;

        // Collect all registrars once, indexed by controller_id. The loop below uses this
        // map to avoid walking the inventory for each controller.

        let registrars = inventory::iter::<GrpcControllerRegistrar>()
            .map(|registrar| (registrar.controller_id, registrar))
            .collect::<HashMap<_, _>>();

        let mut grpc_registry = GrpcServiceRegistry::new();

        #[cfg(feature = "reflection")]
        let mut reflection_descriptor_sets: Vec<&'static [u8]> = Vec::new();

        let body_limit = self.config.body_limit.clone();

        body_limit.display();

        // The register closures borrow the body limit from State, so it must be inserted
        // before the loop below.

        self.state.insert(GrpcBodyLimitValue::from(body_limit));

        for controller_id in &self.controller_ids {
            let Some(registrar) = registrars.get(controller_id).copied() else {
                return Err(sword_error! {
                    code: "sword.runtime.grpc",
                    phase: DiagnosticPhase::Runtime,
                    target: "gRPC controller registry",
                    title: "Controller metadata not found",
                    reason: "No GrpcControllerRegistrar entry was found for controller",
                    context: {
                        "controller_id" => format!("{controller_id:?}"),
                    },
                    hints: ["This usually indicates a controller macro expansion issue"],
                });
            };

            let GrpcControllerRegistrar {
                build: build_controller,
                register: register_controller,
                reflection_descriptor_set: _reflection_descriptor_set,
                ..
            } = registrar;

            // Build first: this inserts the controller into State.
            build_controller(&self.state)?;

            // Register after: it borrows the controller built above and adds its tonic service.
            register_controller(&self.state, &mut grpc_registry)?;

            #[cfg(feature = "reflection")]
            if let Some(descriptor) = _reflection_descriptor_set {
                reflection_descriptor_sets.push(descriptor);
            }
        }

        // A registered controller may still add no tonic services
        // this catches broken macro expansions.
        if grpc_registry.services_count() == 0 {
            return Err(sword_error! {
                code: "sword.runtime.grpc",
                phase: DiagnosticPhase::Runtime,
                target: "GrpcApplication::run",
                title: "No gRPC services were registered",
                reason: "Controllers were discovered but no tonic services were added to routes",
                context: {
                    "controllers_count" => self.controller_ids.len().to_string(),
                },
                hints: ["Implement generated register hooks to add tonic services into GrpcServiceRegistry"],
            });
        }

        // Pull names out before the registry is consumed, so each service can be marked healthy.
        let service_names: Vec<&'static str> = grpc_registry.service_names().collect();
        let routes = grpc_registry.into_routes();

        let (health_reporter, health_service) = tonic_health::server::health_reporter();

        // The empty name marks the whole server as serving.
        health_reporter
            .set_service_status("", tonic_health::ServingStatus::Serving)
            .await;

        for service_name in service_names {
            health_reporter
                .set_service_status(service_name, tonic_health::ServingStatus::Serving)
                .await;
        }

        let logger_config = match &self.config.logger {
            Some(config) => config.clone(),
            None => GrpcLoggerConfig::disabled(),
        };

        // Request-id runs first so the logger (outer layer) can pick it up from the request.
        let server = tonic::transport::Server::builder()
            .layer(RequestIdLayer::new())
            .layer(GrpcLoggerLayer::new(&logger_config))
            .add_routes(routes);

        #[allow(unused_mut)]
        let mut router = server.add_service(health_service);

        #[cfg(feature = "reflection")]
        {
            router = router.add_service(self.build_reflection_service(reflection_descriptor_sets)?);
        }

        // With graceful shutdown the server waits for a signal and drains connections.
        if self.graceful_shutdown {
            router
                .serve_with_shutdown(bind_addr, shutdown.cancelled())
                .await
                .map_err(|err| {
                    sword_error! {
                        code: "sword.runtime.grpc",
                        phase: DiagnosticPhase::Runtime,
                        target: "GrpcApplication::run",
                        title: "gRPC server stopped with an internal error",
                        reason: err,
                        context: {
                            "mode" => "graceful_shutdown",
                            "host" => self.config.host.clone(),
                            "port" => self.config.port.to_string(),
                        },
                        hints: ["Check the gRPC server logs and listener configuration"],
                    }
                })?;

            return Ok(());
        }

        tokio::select! {
            res = router.serve(bind_addr) => {
                res.map_err(|err| {
                    sword_error! {
                        code: "sword.runtime.grpc",
                        phase: DiagnosticPhase::Runtime,
                        target: "GrpcApplication::run",
                        title: "gRPC server stopped with an internal error",
                        reason: err,
                        context: {
                            "mode" => "normal",
                            "host" => self.config.host.clone(),
                            "port" => self.config.port.to_string(),
                        },
                        hints: ["Check the gRPC server logs and listener configuration"],
                    }
                })?;
            }
            _ = shutdown.cancelled() => {}
        }

        Ok(())
    }

    #[cfg(feature = "reflection")]
    fn build_reflection_service(
        &self,
        descriptors: Vec<&'static [u8]>,
    ) -> Result<
        tonic_reflection::server::v1::ServerReflectionServer<
            impl tonic_reflection::server::v1::ServerReflection,
        >,
        Error,
    > {
        let mut reflection_builder = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET);

        for descriptor in descriptors {
            reflection_builder =
                reflection_builder.register_encoded_file_descriptor_set(descriptor);
        }

        reflection_builder.build_v1().map_err(|err| {
            sword_error! {
                code: "sword.runtime.grpc",
                phase: DiagnosticPhase::Runtime,
                target: "GrpcApplication::run",
                title: "Failed to build tonic reflection service",
                reason: err,
                hints: ["Ensure build.rs generates `sword_descriptor_set.bin` when reflection support is enabled"],
            }
        })
    }
}

impl TryFrom<EngineBuildContext> for GrpcApplication {
    type Error = Error;

    fn try_from(ctx: EngineBuildContext) -> Result<Self, Error> {
        let EngineBuildContext {
            state,
            config,
            config_document,
            controllers,
            ..
        } = ctx;

        let app_config = config.get_or_default::<ApplicationConfig>();
        let grpc_config =
            ConfigSection::load::<GrpcApplicationConfig>(&config, config_document.as_deref())?
                .unwrap_or_default();

        let grpc_controllers_ids = controllers.get_by_kind(Controller::Grpc);

        Ok(GrpcApplication {
            state,
            config: grpc_config,
            graceful_shutdown: app_config.graceful_shutdown,
            controller_ids: grpc_controllers_ids,
        })
    }
}
