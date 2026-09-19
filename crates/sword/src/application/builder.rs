use crate::application::{Application, Environment};

#[cfg(any(feature = "web", feature = "socketio"))]
use sword_web::internal::{
    AxumRequest, IntoResponse, TowerLayer as Layer, TowerService as Service, routing::Route,
};

use std::any::Any;
use std::path::Path;
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
}

impl ApplicationBuilder {
    const DEFAULT_CONFIG_PATH: &str = "config/config.toml";

    /// Initialization of the application builder. It can panic at runtime if the
    /// environment is invalid, the config file can't be loaded, or tracing fails to init.
    pub fn new() -> Self {
        let config_path = match Environment::current() {
            Ok(Some(env)) => env.default_config_path(),
            Ok(None) => Self::DEFAULT_CONFIG_PATH,
            Err(err) => {
                sword_error! {
                    title: "Invalid SWORD_ENV variable",
                    reason: err,
                    context: {
                        "source" => "ApplicationBuilder::new",
                    },
                    hints: ["Valid values are: dev, prod, test"],
                }
            }
        };

        let config = Config::builder()
            .add_required_file(Path::new(config_path))
            .build()
            .unwrap_or_else(|err| {
                sword_error! {
                    title: "Failed to load required configuration file",
                    reason: err,
                    context: {
                        "path" => config_path,
                        "source" => "Application initialization"
                    },
                    hints: ["Ensure the file exists and contains valid TOML"],
                }
            });

        Self::from_config(config)
    }

    /// Initializes a sword application from a built `Config`.
    /// When using this method you don't get environment config detection via `SWORD_ENV`.
    pub fn from_config(config: Config) -> Self {
        let state = State::initialize_with(config.clone());

        // Tracing subscriber initialization from the config value. If it's not configured,
        // the default values are used.
        TracingSubscriber::from(config.get_or_default::<TracingConfig>())
            .init()
            .unwrap_or_else(|err| {
                sword_error! {
                    title: "Failed to initialize tracing subscriber",
                    reason: err,
                    source: "ApplicationBuilder::from_config",
                    hints: [
                        "Ensure tracing is initialized only once per process",
                        "Avoid initializing tracing manually before building the app when using Sword bootstrap",
                    ],
                }
            });

        // Collection and registration in the state of configuration structs
        // marked with the `#[config]` macro.
        for ConfigRegistrar { register } in inventory::iter::<ConfigRegistrar> {
            register(&state, &config)
        }

        Self {
            state,
            config,
            container: DependencyContainer::new(),
            controller_registry: ControllerRegistry::new(),
            layer_stack: LayerStack::new(),
        }
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
    /// instance ready to run.
    pub fn build(mut self) -> Application {
        // Runtime check — fires only if both features are enabled AND build() is called.
        // This preserves dev experience for users who enable all features in their IDE.
        if cfg!(feature = "grpc") && (cfg!(feature = "web") || cfg!(feature = "socketio")) {
            sword_error! {
                title: "Multiple application types enabled",
                reason: "Only one app type feature can be enabled at a time",
                hints: [
                    "Enable only one of `web` or `grpc` application type",
                    "Use controller features that match the selected app type",
                ],
            }
        }

        // Since any piece of the application may require the EventPublisher
        // (if the feature is enabled), the events runtime must be built before
        // building the dependency container. Otherwise there will be a runtime error.

        #[cfg(feature = "events-in-memory")]
        let events = sword_events::EventApplicationRuntime::new(&self.state, &self.config);

        self.container.build_all(&self.state).unwrap_or_else(|err| {
            match (err.dependency_path(), err.missing_dependency_path()) {
                (Some(dependency_path), Some(missing_dependency_path)) => {
                    sword_error! {
                        title: "Failed to Build DI Container",
                        reason: err,
                        source: "ApplicationBuilder::build",
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
                        source: "ApplicationBuilder::build",
                        extra_context: err.diagnostic_context(),
                        hints: ["Check that all required components and providers are registered"],
                    }
                }
            }
        });

        // Once the dependency container is built, the interceptors declared with the
        // #[derive(Interceptor)] macro are registered.

        for InterceptorRegistrar { register } in inventory::iter::<InterceptorRegistrar> {
            register(&self.state);
        }

        #[cfg(feature = "events-in-memory")]
        events.start(&self.controller_registry); // Builds event handlers and starts the subscriber.

        self.register_sword_built_in_layers(); // Initialization of built-in layers by the `sword_layers` crate.

        #[allow(unused_variables)]
        let ctx = EngineBuildContext {
            state: self.state,
            config: self.config.clone(),
            controllers: self.controller_registry,
            layer_stack: self.layer_stack,
        };

        // NOTE: With --all-features, only the first matching #[cfg] branch is compiled in.
        // The unreachable code and needless return warnings are suppressed here because
        // each branch is reachable under normal single-feature usage.
        cfg_select! {
            feature = "grpc" => {
                let grpc_app = sword_grpc::application::GrpcApplication::from(ctx);
                let engine = super::ApplicationEngine::Grpc(grpc_app);

                Application::new(engine, self.config)
            }

            any(feature = "web", feature = "socketio") => {
                let web_app = sword_web::application::WebApplication::from(ctx);
                let engine = super::ApplicationEngine::Web(web_app);

                Application::new(engine, self.config)
            }

            _ => {
                sword_error! {
                    title: "No application engine available",
                    reason: "No supported controller feature is enabled",
                    context: {
                        "source" => "ApplicationBuilder::build",
                    },
                    hints: ["Enable one of: web, socketio, grpc"],
                }
            }
        }
    }
}

impl Default for ApplicationBuilder {
    fn default() -> Self {
        Self::new()
    }
}
