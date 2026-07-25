use crate::application::Application;

#[cfg(feature = "events-in-memory")]
use std::sync::Arc;

#[cfg(any(feature = "web", feature = "socketio"))]
use sword_web::internal::{
    AxumRequest, IntoResponse, TowerLayer as Layer, TowerService as Service, routing::Route,
};

use std::any::Any;
use std::path::Path;
use sword_core::*;
use sword_layers::tracing::{TracingConfig, TracingSubscriber};

pub struct ApplicationBuilder {
    state: State,
    container: DependencyContainer,
    controller_registry: ControllerRegistry,
    layer_stack: LayerStack<State>,
    pub config: Config,
}

const DEFAULT_CONFIG_PATH: &str = "config/config.toml";

impl ApplicationBuilder {
    fn load_required_config(path: &str) -> Config {
        Config::builder()
            .add_required_file(Path::new(path))
            .build()
            .unwrap_or_else(|err| {
                sword_error! {
                    title: "Failed to load required configuration file",
                    reason: err,
                    context: {
                        "path" => path,
                        "source" => "Application initialization"
                    },
                    hints: ["Ensure the file exists and contains valid TOML"],
                }
            })
    }

    fn load_default_config() -> Config {
        Self::load_required_config(DEFAULT_CONFIG_PATH)
    }

    pub fn new() -> Self {
        Self::from_config(Self::load_default_config())
    }

    pub fn from_config(config: Config) -> Self {
        let state = State::new();

        state.insert(config.clone());

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

    #[cfg(feature = "events-in-memory")]
    fn init_event_publisher(&self) -> tokio::sync::mpsc::Receiver<Arc<dyn sword_events::Event>> {
        use sword_events::EventQueueConfig;
        use sword_events::in_memory::EventPublisher;

        let config = self.state.get::<EventQueueConfig>().unwrap_or_else(|_| {
            let config = EventQueueConfig::default();
            self.state.insert(config.clone());
            config
        });

        let (tx, rx) =
            tokio::sync::mpsc::channel::<Arc<dyn sword_events::Event>>(config.buffer_size);
        let publisher = EventPublisher::new(tx);
        self.state.insert(publisher);

        rx
    }

    #[cfg(feature = "events-in-memory")]
    fn init_event_subscriber(
        &self,
        rx: tokio::sync::mpsc::Receiver<Arc<dyn sword_events::Event>>,
    ) -> Option<tokio::sync::watch::Sender<bool>> {
        use std::any::TypeId;
        use std::collections::HashMap;

        use sword_events::in_memory::EventSubscriber;
        use sword_events::{
            EventHandlerFn, EventQueueConfig, MemEventControllerRegistrar, MemEventRouteRegistrar,
        };

        let event_controllers = self
            .controller_registry
            .get_by_kind(Controller::MemEventHandler);

        if event_controllers.is_empty() {
            return None;
        }

        let config = self.state.get::<EventQueueConfig>().unwrap_or_else(|_| {
            let config = EventQueueConfig::default();
            self.state.insert(config.clone());
            config
        });

        let controller_registrars: HashMap<TypeId, &MemEventControllerRegistrar> =
            inventory::iter::<MemEventControllerRegistrar>()
                .map(|r| (r.handler_type_id, r))
                .collect();

        let mut route_map: HashMap<TypeId, Vec<&MemEventRouteRegistrar>> = HashMap::new();
        for route in inventory::iter::<MemEventRouteRegistrar>() {
            route_map
                .entry(route.handler_type_id)
                .or_default()
                .push(route);
        }

        let mut handlers: HashMap<&'static str, Vec<EventHandlerFn>> = HashMap::new();

        for type_id in &event_controllers {
            let Some(registrar) = controller_registrars.get(type_id) else {
                tracing::warn!(
                    target: "sword.events",
                    "No MemEventControllerRegistrar found for handler type {:?}",
                    type_id,
                );
                continue;
            };

            (registrar.build)(&self.state);

            let Some(routes) = route_map.get(type_id) else {
                tracing::warn!(
                    target: "sword.events",
                    "No event routes registered for handler type {:?}",
                    type_id,
                );
                continue;
            };

            for route in routes {
                let handle_fn = (route.build_and_handle)(&self.state);
                handlers.entry(route.event_key).or_default().push(handle_fn);
            }
        }

        if handlers.is_empty() {
            return None;
        }

        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let subscriber = EventSubscriber::new(rx, handlers, config);
        subscriber.run(shutdown_rx);
        tracing::info!(target: "sword.events", "Event queue initialized");

        Some(shutdown_tx)
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
    /// instance ready to run.
    pub fn build(mut self) -> Application {
        // Runtime check — fires only if both features are enabled AND build() is called.
        // This preserves dev experience for users who enable all features in their IDE.
        #[cfg(feature = "events-in-memory")]
        let event_rx = self.init_event_publisher();

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

        for InterceptorRegistrar { register } in inventory::iter::<InterceptorRegistrar> {
            register(&self.state);
        }

        #[cfg(feature = "events-in-memory")]
        let event_shutdown_tx = self.init_event_subscriber(event_rx);

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

                Application::new(
                    engine,
                    self.config,
                    #[cfg(feature = "events-in-memory")]
                    event_shutdown_tx,
                )
            }

            any(feature = "web", feature = "socketio") => {
                let web_app = sword_web::application::WebApplication::from(ctx);
                let engine = super::ApplicationEngine::Web(web_app);

                Application::new(
                    engine,
                    self.config,
                    #[cfg(feature = "events-in-memory")]
                    event_shutdown_tx,
                )
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
