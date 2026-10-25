use crate::application::{Application, Engine, Environment};

#[cfg(any(feature = "events-memory", feature = "events-nats"))]
use sword_events::EventApplication;

#[cfg(feature = "grpc")]
use sword_grpc::application::GrpcApplication;

#[cfg(feature = "jobs")]
use sword_jobs::JobSchedulerRuntime;

#[cfg(any(feature = "web", feature = "socketio"))]
use sword_web::{
    application::WebApplication,
    internal::{
        AxumRequest, IntoResponse, TowerLayer as Layer, TowerService as Service, routing::Route,
    },
};

use std::any::Any;
use std::path::Path;
use std::sync::Arc;
use sword_core::diagnostic::{DiagnosticPhase, Error};
use sword_core::sword_diagnostic;
use sword_core::*;
use sword_layers::tracing::{TracingConfig, TracingSubscriber};

/// Builder pattern of a sword application.
/// It's in charge of assembling the pieces needed to build the application.
///
/// `Config` -> `State` <- `DependencyContainer`
/// `State` -> `Controllers` / `Interceptors` / `Layers`
pub struct ApplicationBuilder {
    state: State,
    container: DependencyContainer,
    controller_registry: ControllerRegistry,
    layer_stack: LayerStack<State>,
    pub config: Config,
    config_document: Option<Arc<ConfigDocument>>,
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
                return Err(sword_diagnostic! {
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
                sword_diagnostic! {
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
        let config_document = ConfigDocument::load(&config);
        let tracing_config =
            ConfigSection::load::<TracingConfig>(&config, config_document.as_ref())?
                .unwrap_or_default();

        TracingSubscriber::from(tracing_config)
            .init()
            .map_err(|err| {
                sword_diagnostic! {
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
            layer_stack: LayerStack::new(),
            config_document: config_document.map(Arc::new),
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

    #[cfg(any(feature = "web", feature = "socketio"))]
    /// Adds a `tower::Layer` to the application builder.
    ///
    /// This method is equivalent to Axum's `Router::layer` method, allowing you to
    /// apply Tower layers to the application's router.
    ///
    /// Custom layers are applied **after** all built-in Sword layers,
    /// making them the outermost layer in the layer stack.
    /// This means custom layers execute first on incoming requests and last on outgoing responses.
    pub fn with_layer<L>(mut self, layer: L) -> Self
    where
        L: Layer<Route> + Clone + Send + Sync + 'static,
        L::Service: Service<AxumRequest> + Clone + Send + Sync + 'static,
        <L::Service as Service<AxumRequest>>::Response: IntoResponse + 'static,
        <L::Service as Service<AxumRequest>>::Error: Into<std::convert::Infallible> + 'static,
        <L::Service as Service<AxumRequest>>::Future: Send + 'static,
    {
        self.layer_stack.push(layer);
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

    /// Iterates and looks for layers defined in `sword_layers` and activated with their
    /// corresponding feature flags. The collection is automatic, so there's no need to use `with_layer`.
    fn register_sword_built_in_layers(&mut self) {
        for registrar in inventory::iter::<sword_layers::SwordLayerRegistrar>() {
            let display_fn = registrar.display;
            let push_layer_fn = (registrar.register)(&self.config);

            tracing::info!(
                target: "sword.layers",
                name = registrar.name,
                "Layer registered"
            );

            display_fn(&self.config);
            push_layer_fn(&mut self.layer_stack as &mut dyn Any);
        }
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
        let mut events_application = EventApplication::new(
            &self.config,
            self.container.provider_registry(),
            self.config_document.as_deref(),
        )?;

        // The dependency container is built once all providers and components are registered.
        self.build_dependency_container();

        // Once the dependency container is built, the interceptors declared with the
        // #[derive(Interceptor)] macro are registered.
        for InterceptorRegistrar { register } in inventory::iter::<InterceptorRegistrar> {
            register(&self.state)?;
        }

        // Initialization of built-in layers by the `sword_layers` crate.
        self.register_sword_built_in_layers();

        #[allow(unused_variables)]
        let ctx = EngineBuildContext {
            state: self.state,
            config: self.config.clone(),
            config_document: self.config_document.clone(),
            controllers: self.controller_registry,
            layer_stack: self.layer_stack,
        };

        #[cfg(any(feature = "events-memory", feature = "events-nats"))]
        events_application.with_controllers(&ctx.controllers, &ctx.state);

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

    fn build_dependency_container(&mut self) {
        self.container.build_all(&self.state).unwrap_or_else(|err| {
            match (err.dependency_path(), err.missing_dependency_path()) {
                (Some(dependency_path), Some(missing_dependency_path)) => {
                    sword_error! {
                        title: "Failed to Build DI Container",
                        reason: err,
                        source: "ApplicationBuilder::build_dependency_container",
                        fields: {
                            dependency_path = dependency_path,
                            missing_dependency_path = missing_dependency_path,
                        },
                        hints: ["Check that all required components and providers are registered"],
                    }
                }
                _ => {
                    sword_error! {
                        title: "Failed to Build DI Container",
                        reason: err,
                        source: "ApplicationBuilder::build_dependency_container",
                        extra_context: err.diagnostic_context(),
                        hints: ["Check that all required components and providers are registered"],
                    }
                }
            }
        });
    }
}
