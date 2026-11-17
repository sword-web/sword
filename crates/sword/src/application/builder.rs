use std::path::Path;

use sword_core::*;

#[cfg(any(feature = "events-memory", feature = "events-nats"))]
use sword_events::EventApplication;

#[cfg(feature = "grpc")]
use sword_grpc::GrpcApplication;

#[cfg(feature = "jobs")]
use sword_jobs::JobSchedulerRuntime;

#[cfg(any(feature = "web", feature = "socketio"))]
use sword_web::WebApplication;

use crate::application::{Application, Engine, Environment};
use crate::logging::{TracingConfig, TracingSubscriber};

/// Builder pattern of a sword application.
/// It's in charge of assembling the pieces needed to build the application.
///
/// `Config` -> `State` <- `DependencyContainer`
/// `State` -> `Controllers` / `Interceptors` / `Layers`
pub struct ApplicationBuilder {
    pub config: Config,
    state: State,
    container: DependencyContainer,
    controller_registry: ControllerRegistry,
}

impl ApplicationBuilder {
    const DEFAULT_CONFIG_PATH: &str = "config/config.toml";

    /// Initializes the builder from `SWORD_ENV`, falling back to the default
    /// configuration path.
    pub fn new() -> Result<Self, Error> {
        let config_path = match Environment::current() {
            Ok(Some(env)) => env.default_config_path(),
            Ok(None) => Self::DEFAULT_CONFIG_PATH,
            Err(err) => {
                return Err(sword_error! {
                    code: "sword.build.application",
                    phase: DiagnosticPhase::Build,
                    target: "SWORD_ENV",
                    title: "Invalid SWORD_ENV variable",
                    reason: err,
                    source: "ApplicationBuilder::new",
                    hints: ["Valid values are: dev, prod, test"],
                });
            }
        };

        let config = Config::builder()
            .add_required_file(Path::new(config_path))
            .build()
            .map_err(|err| {
                sword_error! {
                    code: "sword.build.config",
                    phase: DiagnosticPhase::Build,
                    target: config_path,
                    title: "Failed to load required configuration file",
                    reason: err,
                    source: "ApplicationBuilder::new",
                    hints: ["Ensure the file exists and contains valid TOML"],
                    cause: err,
                }
            })?;

        Self::from_config(config)
    }

    /// Initializes a sword application from a built `Config`.
    /// When using this method you don't get environment config detection via `SWORD_ENV`.
    pub fn from_config(config: Config) -> Result<Self, Error> {
        validate_all(&config)?;

        let tracing_config = config.get_or_default::<TracingConfig>();

        TracingSubscriber::from(tracing_config)
            .init()
            .map_err(|err| {
                sword_error! {
                    code: "sword.build.tracing",
                    phase: DiagnosticPhase::Build,
                    target: "tracing subscriber",
                    title: "Failed to initialize tracing subscriber",
                    reason: err,
                    source: "ApplicationBuilder::from_config",
                    hints: [
                        "Initialize tracing only once per process",
                        "Avoid initializing tracing manually before building the app",
                    ],
                    cause: err,
                }
            })?;

        let app_config = config
            .try_get::<ApplicationConfig>()
            .map_err(|err| {
                sword_error! {
                    code: "sword.build.application",
                    phase: DiagnosticPhase::Build,
                    target: "application configuration",
                    title: "Invalid [application] configuration",
                    reason: err,
                    source: "ApplicationBuilder::from_config",
                    hints: ["Check the types of the values in [application]"],
                    cause: err,
                }
            })?
            .unwrap_or_default();

        app_config.diagnostics.apply()?;

        let state = State::initialize_with(config.clone());

        // Collection and registration in the state of configuration structs
        // marked with the `#[config]` macro.
        for ConfigRegistrar { register } in inventory::iter::<ConfigRegistrar> {
            register(&state, &config)?;
        }

        Ok(Self {
            state,
            config,
            container: DependencyContainer::new(),
            controller_registry: ControllerRegistry::new(),
        })
    }

    /// Register a module with the application builder.
    /// Can be used with any type that implements the `Module` trait.
    pub fn with_module<M>(self) -> Self
    where
        M: sword_core::Module,
    {
        futures_lite::future::block_on(M::register_providers(
            &self.config,
            self.container.provider_registry(),
        ));

        M::register_components(self.container.component_registry());
        M::register_controllers(&self.controller_registry);

        self
    }

    /// Register a provider directly with the application builder.
    ///
    /// This method can be used to add providers directly to the application, avoiding the need
    /// to create a full module when only a provider is needed.
    pub fn with_provider<T>(self, provider: T) -> Self
    where
        T: Provider + 'static,
    {
        self.container.provider_registry().register(provider);
        self
    }

    /// Build the `Application` instance with the configured options.
    ///
    /// This method ends the builder pattern and constructs the final `Application`
    /// instance ready to run. It assembles every enabled engine (web, grpc, events, jobs)
    /// into the application, the engines themselves start in `Application::run`.
    #[allow(clippy::vec_init_then_push)]
    pub fn build(mut self) -> Result<Application, Error> {
        // `EventApplication` has publishers internally wich are providers so it's important to
        // register them before building the dependency container to make it available to components.

        #[cfg(any(feature = "events-memory", feature = "events-nats"))]
        let mut events_application =
            EventApplication::new(&self.config, self.container.provider_registry())?;

        // The dependency container is built once all providers and components are registered.
        self.build_dependency_container()?;

        // Once the dependency container is built, the interceptors declared with the
        // #[derive(Interceptor)] macro are registered.
        for InterceptorRegistrar { register } in inventory::iter::<InterceptorRegistrar> {
            register(&self.state)?;
        }

        #[allow(unused_variables)]
        let ctx = EngineBuildContext {
            state: self.state,
            config: self.config.clone(),
            controllers: self.controller_registry,
        };

        #[cfg(any(feature = "events-memory", feature = "events-nats"))]
        events_application.with_controllers(&ctx.controllers, &ctx.state)?;

        #[allow(unused_mut, clippy::vec_init_then_push)]
        let mut engines: Vec<Engine> = Vec::new();

        #[cfg(any(feature = "web", feature = "socketio"))]
        engines.push(Engine::Web(WebApplication::try_from(ctx.clone())?));

        #[cfg(feature = "grpc")]
        engines.push(Engine::Grpc(GrpcApplication::try_from(ctx.clone())?));

        #[cfg(feature = "jobs")]
        engines.push(Engine::Jobs(JobSchedulerRuntime::from(ctx)));

        #[cfg(any(feature = "events-memory", feature = "events-nats"))]
        engines.push(Engine::Events(events_application));

        Ok(Application::new(engines, self.config))
    }

    fn build_dependency_container(&mut self) -> Result<(), Error> {
        self.container.build_all(&self.state).map_err(|err| {
            let missing = err
                .missing_dependency_path()
                .and_then(|path| path.rsplit("::").next())
                .unwrap_or("the missing dependency");

            let component = err
                .dependency_path()
                .and_then(|path| path.rsplit("::").next())
                .unwrap_or("the component");

            let mut diagnostic = Diagnostic::builder()
                .code("sword.build.di")
                .phase(DiagnosticPhase::Build)
                .target("dependency container")
                .title("Failed to build the dependency container")
                .reason(err.to_string())
                .source("ApplicationBuilder::build_dependency_container")
                .hint(format!("Register `{missing}` as a provider or component"));

            for (key, value) in err.diagnostic_context() {
                diagnostic = diagnostic.context(key, value);
            }

            if let Some(location) = err.source_location() {
                diagnostic = diagnostic.source_location(
                    location.file(),
                    location.line(),
                    location.column(),
                    location.length(),
                    format!("`{component}`"),
                );
            }

            Error::new(diagnostic.build()).with_source(err)
        })
    }
}

fn validate_all(config: &Config) -> Result<(), Error> {
    let mut failures = Vec::new();

    if let Err(failure) = config.get_validated_section::<ApplicationConfig>() {
        failures.push(failure);
    }

    #[cfg(feature = "web")]
    if let Err(failure) = config.get_validated_section::<sword_web::WebApplicationConfig>() {
        failures.push(failure);
    }

    #[cfg(feature = "grpc")]
    if let Err(failure) = config.get_validated_section::<sword_grpc::GrpcApplicationConfig>() {
        failures.push(failure);
    }

    #[cfg(feature = "socketio")]
    if let Err(failure) = config.get_validated_section::<sword_socketio::SocketIoServerConfig>() {
        failures.push(failure);
    }

    #[cfg(any(feature = "events-memory", feature = "events-nats"))]
    if let Err(failure) = config.get_validated_section::<sword_events::EventApplicationConfig>() {
        failures.push(failure);
    }

    if let Err(failure) = config.get_validated_section::<TracingConfig>() {
        failures.push(failure);
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(sword_core::config_failures_error(failures, config))
    }
}
