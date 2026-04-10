use crate::application::{Application, ApplicationConfig, ApplicationEngine};
use crate::module::Module;
#[cfg(feature = "grpc-controllers")]
use sword_grpc::application::GrpcApplication;
#[cfg(all(
    any(feature = "web-controllers", feature = "socketio-controllers"),
    not(feature = "grpc-controllers")
))]
use sword_web::application::WebApplication;

use axum::{extract::Request as AxumRequest, response::IntoResponse, routing::Route};
use std::convert::Infallible;
use std::path::Path;
use sword_core::{
    Config, ConfigRegistrar, ControllerRegistry, DependencyContainer, InterceptorRegistrar,
    Provider, State, sword_error,
};
use sword_layers::layer_stack::LayerStack;
use sword_layers::tracing::{TracingConfig, TracingSubscriber};

use tower::{Layer, Service};

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
        M: Module,
    {
        futures_lite::future::block_on(M::register_providers(
            &self.config,
            self.container.provider_registry(),
        ));

        M::register_components(self.container.component_registry());
        M::register_controllers(&self.controller_registry);

        self
    }

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
        <L::Service as Service<AxumRequest>>::Error: Into<Infallible> + 'static,
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

    /// Build the `Application` instance with the configured options.
    ///
    /// This method ends the builder pattern and constructs the final `Application`
    /// instance ready to run.
    pub fn build(self) -> Application {
        if cfg!(feature = "grpc-controllers")
            && (cfg!(feature = "web-controllers") || cfg!(feature = "socketio-controllers"))
        {
            sword_error! {
                title: "Multiple application types enabled",
                reason: "Only one app type feature can be enabled at a time",
                hints: [
                    "Enable only one of `web-controllers` or `grpc-controllers`",
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

        #[cfg(feature = "grpc-controllers")]
        {
            let app_config = self.config.get_or_default::<ApplicationConfig>();
            let grpc_application = GrpcApplication::new(
                self.state.clone(),
                &self.config,
                app_config.grpc.clone(),
                app_config.graceful_shutdown,
                &self.controller_registry,
            );

            Application::new(ApplicationEngine::Grpc(grpc_application), self.config)
        }

        #[cfg(all(
            any(feature = "web-controllers", feature = "socketio-controllers"),
            not(feature = "grpc-controllers")
        ))]
        {
            let app_config = self.config.get_or_default::<ApplicationConfig>();
            let web_application = WebApplication::new(
                self.state.clone(),
                &self.config,
                app_config.web.clone(),
                app_config.graceful_shutdown,
                self.layer_stack,
                &self.controller_registry,
            );

            Application::new(ApplicationEngine::Web(web_application), self.config)
        }

        #[cfg(not(any(
            feature = "web-controllers",
            feature = "socketio-controllers",
            feature = "grpc-controllers"
        )))]
        sword_error! {
            title: "No application engine available",
            reason: "No supported controller feature is enabled",
            context: {
                "source" => "ApplicationBuilder::build",
            },
            hints: ["Enable one of: web-controllers, socketio-controllers, grpc-controllers"],
        }
    }
}

impl Default for ApplicationBuilder {
    fn default() -> Self {
        Self::new()
    }
}
