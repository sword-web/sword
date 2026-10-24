#![allow(dead_code)]

mod core;
mod shared;

#[cfg(feature = "events")]
mod events;

#[cfg(feature = "grpc")]
mod grpc;

#[cfg(feature = "jobs")]
mod jobs;

#[cfg(feature = "socketio")]
mod socketio;

#[cfg(feature = "web")]
mod web;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

/// Defines a configuration struct for the application.
/// This macro generates the necessary code to deserialize the struct from
/// the configuration toml file.
///
/// The struct must derive `Deserialize` from `serde`.
///
/// ### Parameters
/// - `key`: The key in the configuration file where the struct is located.
///
/// ### Usage
///
/// ```rust,ignore
/// #[config(key = "my-section")]
/// #[derive(Debug, Deserialize)]
/// struct MyConfig {
///     my_key: String,
/// }
/// ```
#[proc_macro_attribute]
pub fn config(attr: TokenStream, item: TokenStream) -> TokenStream {
    shared::emit(core::expand_config_struct(attr, item))
}

/// Marks a struct as injectable.
///
/// This macro generates the necessary code to register the struct
/// in the dependency injection container. It can be used with or without
/// parameters.
///
/// ### Parameters
///
/// - `kind`: (Optional) Specifies the kind of injectable.
///   It can be either `provider` or `component`.
///
///  `provider`: The struct that has to be instantiated manually and
///   registered in the container. The struct will be treated as a singleton by default.
///
///  `component`: The struct will be instantiated automatically by the container
///   based on its dependencies. It's also treated as a singleton by default.
///
///   By default, if no kind is provided, it will be treated as `component`.
///
/// - `no_derive_clone`: (Optional) If provided, the struct will not derive the `Clone` automatically.
///   By default, the struct will derive `Clone` if all its fields implement `Clone`.
///
/// ### Usage of `#[injectable]` without parameters (same as #[injectable(component)])
///
/// ```rust,ignore
/// #[injectable]
/// pub struct TaskRepository {
///     db: Database,
/// }
///
/// impl TaskRepository {
///     pub async fn create(&self, task: Value) {
///         self.db.insert("tasks", task).await;
///     }
///
///     pub async fn find_all(&self) -> Option<Vec<Value>> {
///         self.db.get_all("tasks").await
///     }
/// }
/// ```
///
/// ### Usage of `#[injectable(provider)]` with parameters
///
/// ```rust,ignore
/// #[injectable(provider)]
/// pub struct Database {
///     db: Store,
/// }
///
/// impl Database {
///     pub async fn new(db_conf: DatabaseConfig) -> Self {
///         let db = Arc::new(RwLock::new(HashMap::new()));
///
///         db.write().await.insert(db_conf.collection_name, Vec::new());
///
///         Self { db }
///     }
///
///     pub async fn insert(&self, table: &'static str, record: Value) {
///         let mut db = self.db.write().await;
///
///         if let Some(table_data) = db.get_mut(table) {
///             table_data.push(record);
///         }
///     }
///
///     pub async fn get_all(&self, table: &'static str) -> Option<Vec<Value>> {
///         let db = self.db.read().await;
///
///         db.get(table).cloned()
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn injectable(attr: TokenStream, item: TokenStream) -> TokenStream {
    shared::emit(core::expand_injectable(attr, item))
}

/// Defines a Sword controller.
///
/// The controller kind and its specific arguments are declared in the
/// attribute, e.g. `#[controller(kind = Controller::Web, path = "/api")]`.
#[proc_macro_attribute]
pub fn controller(attr: TokenStream, item: TokenStream) -> TokenStream {
    shared::emit(core::expand_controller(attr, item))
}

/// Derive macro for creating interceptors.
///
/// Generates implementations for the `Interceptor` trait.
///
/// # Usage
/// ```rust,ignore
/// use sword::prelude::*;
///
/// #[derive(Interceptor)]
/// struct MyInterceptor;
///
/// // then implement some Interceptor trait variants
/// // depending on the controller kind (e.g. OnRequest, OnConnect.)
/// ```
#[proc_macro_derive(Interceptor)]
pub fn derive_interceptor(item: TokenStream) -> TokenStream {
    shared::emit(core::expand_interceptor_derive(item))
}

/// Marks a route or controller with one or more interceptors.
/// This macro can be used to apply an `Interceptor` to different controller kinds,
/// such as web controllers or Socket.IO controllers.
#[proc_macro_attribute]
pub fn interceptor(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _ = attr;
    item
}

#[cfg(feature = "events")]
#[proc_macro_attribute]
pub fn event(attr: TokenStream, item: TokenStream) -> TokenStream {
    shared::emit(events::expand_event_struct(attr, item))
}

#[cfg(feature = "events")]
#[proc_macro_attribute]
pub fn subscribe(attr: TokenStream, item: TokenStream) -> TokenStream {
    shared::emit(events::expand_subscribe(attr, item))
}

/// Unified handler attribute for Socket.IO events.
///
/// ### Event Types
/// - `#[on("connection")]` - Called when a client connects
/// - `#[on("disconnection")]` - Called when a client disconnects
/// - `#[on("fallback")]` - Called for unhandled events
/// - `#[on("custom_event")]` - Called for custom event names
///
/// ### Parameters
/// All handlers receive `&self` and `ctx: SocketContext` which provides access to:
/// - Socket operations via `ctx`
/// - Message data via `ctx.try_data::<T>()`
/// - Event name via `ctx.event()`
/// - Acknowledgments via `ctx.ack()`
///
/// ### Usage
/// ```rust,ignore
/// #[controller(kind = Controller::SocketIo, namespace = "/chat")]
/// pub struct ChatController { ... }
///
/// impl ChatController {
///     #[on("connection")]
///     async fn on_connect(&self, ctx: SocketContext) {
///         println!("Client connected: {}", ctx.id());
///     }
///
///     #[on("message")]
///     async fn handle_message(&self, ctx: SocketContext) {
///         let msg: String = ctx.try_data().unwrap();
///         println!("Received: {}", msg);
///     }
/// }
/// ```
#[cfg(feature = "socketio")]
#[proc_macro_attribute]
pub fn on(attr: TokenStream, item: TokenStream) -> TokenStream {
    shared::emit(socketio::expand_on_handler(attr, item))
}

#[cfg(feature = "web")]
#[proc_macro_derive(HttpError, attributes(http, http_error, tracing))]
pub fn derive_http_error(input: TokenStream) -> TokenStream {
    shared::emit(web::derive_http_error(input))
}

/// Derive macro for gRPC error enums.
///
/// Generates:
/// - `From<Self> for tonic::Status`
///
/// Enum-level defaults can be declared with `#[grpc_error(...)]` and overridden per
/// variant with `#[grpc(...)]`.
///
/// Supported attributes:
/// - `code = "invalid_argument"`
/// - `message = "custom text"`
/// - `message = field_name`
/// - `transparent` (variant-only)
/// - `tracing = <level>` inside `#[grpc_error(...)]` or `#[grpc(...)]`
/// - `#[tracing(level)]`: backward-compatible shorthand at variant level
/// - When `tracing` is not specified, the level defaults from the gRPC code
///   (`ok` → `info`, client errors → `warn`, server errors → `error`), matching the
///   gRPC access logger's `auto` policy.
///
/// gRPC code values accepted by `#[grpc(code = "...")]`:
///
/// - `ok`
/// - `cancelled`
/// - `unknown`
/// - `invalid_argument`
/// - `deadline_exceeded`
/// - `not_found`
/// - `already_exists`
/// - `permission_denied`
/// - `resource_exhausted`
/// - `failed_precondition`
/// - `aborted`
/// - `out_of_range`
/// - `unimplemented`
/// - `internal`
/// - `unavailable`
/// - `data_loss`
/// - `unauthenticated`
///
/// # Example
///
/// ```rust,ignore
/// use sword::prelude::*;
/// use thiserror::Error;
///
/// #[derive(Debug, Error, GrpcError)]
/// #[grpc_error(code = "internal", tracing = error)]
/// enum UserError {
///     #[grpc(code = "not_found", tracing = info)]
///     #[error("User not found: {id}")]
///     NotFound { id: String },
///
///     #[grpc(code = "invalid_argument", message = client_message)]
///     #[error("Validation error: {internal}")]
///     Validation {
///         client_message: String,
///         internal: String,
///     },
///
///     #[grpc(transparent)]
///     #[error("Database error: {0}")]
///     Database(#[from] anyhow::Error),
/// }
/// ```
#[cfg(feature = "grpc")]
#[proc_macro_derive(GrpcError, attributes(grpc, grpc_error, tracing))]
pub fn derive_grpc_error(input: TokenStream) -> TokenStream {
    shared::emit(grpc::derive_grpc_error(input))
}

#[cfg(feature = "jobs")]
#[proc_macro_attribute]
pub fn cron(attr: TokenStream, item: TokenStream) -> TokenStream {
    shared::emit(jobs::schedule::expand_job_task(
        jobs::schedule::ScheduleKind::Cron,
        attr,
        item,
    ))
}

#[cfg(feature = "jobs")]
#[proc_macro_attribute]
pub fn repeated(attr: TokenStream, item: TokenStream) -> TokenStream {
    shared::emit(jobs::schedule::expand_job_task(
        jobs::schedule::ScheduleKind::Repeated,
        attr,
        item,
    ))
}

#[cfg(feature = "jobs")]
#[proc_macro_attribute]
pub fn once(attr: TokenStream, item: TokenStream) -> TokenStream {
    shared::emit(jobs::schedule::expand_job_task(
        jobs::schedule::ScheduleKind::Once,
        attr,
        item,
    ))
}

/// Runs an async `main` on a multi-threaded Tokio runtime.
///
/// The body must return `Result<(), Error>`. If it returns an error, the
/// diagnostic is reported once and the process exits with a failure code. That
/// is what lets bootstrap failures reach the user as a single report instead of
/// a panic or a stray log line.
///
/// ```ignore
/// #[sword::main]
/// async fn main() -> Result<(), Error> {
///     Application::from_config_path("Config.toml")?.run().await
/// }
/// ```
///
/// The attribute takes no arguments. A runtime that fails to start is reported
/// the same way.
#[proc_macro_attribute]
pub fn main(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemFn);

    let body = input.block;
    let attrs = input.attrs;
    let vis = input.vis;

    let output = if cfg!(feature = "hot-reload") {
        quote! {
            async fn __internal_main() {
                let result: ::std::result::Result<(), ::sword::diagnostic::Error> =
                    (async #body).await;

                if let Err(error) = result {
                    error.report();
                    ::std::process::exit(1);
                }
            }

            #(#attrs)*
            #vis fn main() -> ::std::process::ExitCode {
                let runtime = match ::sword::internal::tokio_runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(err) => {
                        let error = ::sword::internal::core::sword_diagnostic! {
                            code: "sword.runtime.main",
                            phase: ::sword::internal::core::diagnostic::DiagnosticPhase::Runtime,
                            target: "#[sword::main]",
                            title: "Failed to build Tokio runtime",
                            reason: err,
                            hints: [
                                "Check the runtime configuration and available system resources",
                            ],
                        };

                        error.report();

                        return ::std::process::ExitCode::FAILURE;
                    }
                };

                runtime.block_on(
                    ::sword::internal::dioxus_devtools::serve_subsecond(__internal_main),
                );

                ::std::process::ExitCode::SUCCESS
            }
        }
    } else {
        quote! {
            #(#attrs)*
            #vis fn main() -> ::std::process::ExitCode {
                let result: ::std::result::Result<(), ::sword::diagnostic::Error> =
                    match ::sword::internal::tokio_runtime::Builder::new_multi_thread()
                        .enable_all()
                        .build()
                    {
                        Ok(runtime) => runtime.block_on(async #body),
                        Err(err) => Err(::sword::internal::core::sword_diagnostic! {
                            code: "sword.runtime.main",
                            phase: ::sword::internal::core::diagnostic::DiagnosticPhase::Runtime,
                            target: "#[sword::main]",
                            title: "Failed to build Tokio runtime",
                            reason: err,
                            hints: [
                                "Check the runtime configuration and available system resources",
                            ],
                        }),
                    };

                match result {
                    Ok(()) => ::std::process::ExitCode::SUCCESS,
                    Err(error) => {
                        error.report();
                        ::std::process::ExitCode::FAILURE
                    }
                }
            }
        }
    };

    output.into()
}

// ---- web route attributes --------------------------------------------------

#[cfg(feature = "web")]
macro_rules! define_routes {
    ($($fn_name:ident => $method:literal),* $(,)?) => {
        $(
            #[proc_macro_attribute]
            pub fn $fn_name(attr: TokenStream, item: TokenStream) -> TokenStream {
                shared::emit(web::expand_route($method, attr, item))
            }
        )*
    };
}

#[cfg(feature = "web")]
define_routes!(
    get => "GET",
    post => "POST",
    put => "PUT",
    patch => "PATCH",
    delete => "DELETE",
    head => "HEAD",
    options => "OPTIONS",
    trace => "TRACE",
    connect => "CONNECT",
    sse => "SSE",
);
