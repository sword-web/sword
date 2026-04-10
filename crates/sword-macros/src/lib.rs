#![allow(irrefutable_let_patterns)]

mod controllers;
mod core;
mod errors;
mod interceptor_derive;
mod shared;
mod interceptor {
    mod parse;
    pub use parse::InterceptorArgs;
}

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[cfg(feature = "web-controllers")]
#[proc_macro_attribute]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    controllers::web::attributes::attribute("GET", attr, item)
}

#[cfg(not(feature = "web-controllers"))]
#[proc_macro_attribute]
pub fn get(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    syn::Error::new(
        proc_macro2::Span::call_site(),
        "#[get] requires enabling the `web-controllers` feature",
    )
    .to_compile_error()
    .into()
}

#[cfg(feature = "web-controllers")]
#[proc_macro_attribute]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream {
    controllers::web::attributes::attribute("POST", attr, item)
}

#[cfg(not(feature = "web-controllers"))]
#[proc_macro_attribute]
pub fn post(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    syn::Error::new(
        proc_macro2::Span::call_site(),
        "#[post] requires enabling the `web-controllers` feature",
    )
    .to_compile_error()
    .into()
}

#[cfg(feature = "web-controllers")]
#[proc_macro_attribute]
pub fn put(attr: TokenStream, item: TokenStream) -> TokenStream {
    controllers::web::attributes::attribute("PUT", attr, item)
}

#[cfg(not(feature = "web-controllers"))]
#[proc_macro_attribute]
pub fn put(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    syn::Error::new(
        proc_macro2::Span::call_site(),
        "#[put] requires enabling the `web-controllers` feature",
    )
    .to_compile_error()
    .into()
}

#[cfg(feature = "web-controllers")]
#[proc_macro_attribute]
pub fn delete(attr: TokenStream, item: TokenStream) -> TokenStream {
    controllers::web::attributes::attribute("DELETE", attr, item)
}

#[cfg(not(feature = "web-controllers"))]
#[proc_macro_attribute]
pub fn delete(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    syn::Error::new(
        proc_macro2::Span::call_site(),
        "#[delete] requires enabling the `web-controllers` feature",
    )
    .to_compile_error()
    .into()
}

#[cfg(feature = "web-controllers")]
#[proc_macro_attribute]
pub fn patch(attr: TokenStream, item: TokenStream) -> TokenStream {
    controllers::web::attributes::attribute("PATCH", attr, item)
}

#[cfg(not(feature = "web-controllers"))]
#[proc_macro_attribute]
pub fn patch(_attr: TokenStream, _item: TokenStream) -> TokenStream {
    syn::Error::new(
        proc_macro2::Span::call_site(),
        "#[patch] requires enabling the `web-controllers` feature",
    )
    .to_compile_error()
    .into()
}

/// Defines a Sword controller.
/// Route handlers are declared directly inside the `impl` block using method attributes
/// such as `#[get]`, `#[post]`, `#[put]`, `#[patch]`, and `#[delete]`.
///
/// ### Parameters
/// - `kind`: Controller kind. Use `Controller::Web` or `Controller::SocketIo`.
/// - `path`: Required when `kind = Controller::Web`.
/// - `namespace`: Required when `kind = Controller::SocketIo`.
///
/// ### Usage
/// ```rust,ignore
/// #[controller(kind = Controller::Web, path = "/base_path")]
/// struct MyController {}
///
/// impl MyController {
///     #[get("/sub_path")]
///     async fn my_handler(&self) -> WebResult {
///        Ok(JsonResponse::Ok().message("Hello from MyController"))
///     }
/// }
/// ```
///
/// ```rust,ignore
/// #[controller(kind = Controller::SocketIo, namespace = "/chat")]
/// struct ChatController;
///
/// impl ChatController {
///     #[on("connection")]
///     async fn on_connect(&self, _ctx: SocketContext) {}
/// }
/// ```
#[proc_macro_attribute]
pub fn controller(attr: TokenStream, item: TokenStream) -> TokenStream {
    controllers::expand_controller(attr, item).unwrap_or_else(|err| err.to_compile_error().into())
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
pub fn derive_interceptor(input: TokenStream) -> TokenStream {
    interceptor_derive::derive_interceptor(input)
        .unwrap_or_else(|err| err.to_compile_error().into())
}

/// Marks a route or controller with one or more interceptors.
/// This macro can be used to apply an `Interceptor` to different controller kinds,
/// such as web controllers or Socket.IO controllers.
#[proc_macro_attribute]
pub fn interceptor(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _ = attr;
    item
}

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
pub fn config(args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    match core::config::expand_config_struct(args, &input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
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
    core::injectable::expand_injectable(attr, item)
        .unwrap_or_else(|err| err.to_compile_error().into())
}

/// Derive macro for HTTP error enums.
///
/// Generates implementations for:
/// - `From<Self> for JsonResponse` - Converts error to JSON response
/// - `IntoResponse` - Allows returning error directly from handlers
///
/// **Note**: Use with `thiserror::Error` for `Display`, `Error`, and `#[from]`.
///
/// # Attributes
///
/// Each variant must have `#[http(...)]` with one of:
///
/// **For direct responses:**
/// - `code = <u16>`: HTTP status code (required)
/// - `message = "<string>"`: Custom message (optional, defaults to canonical reason)
/// - `error = <field>`: Single error field to include (optional, named fields only)
/// - `errors = <field>`: Multiple errors field to include (optional, named fields only)
///
/// **For delegation:**
/// - `transparent`: Delegate to inner type's `From<T> for Json` (for wrapping other `HttpError` types)
///
/// **Tracing:**
/// - `#[tracing(level)]`: Adds structured logging when the error occurs (optional)
///   - `level`: One of `trace`, `debug`, `info`, `warn`, `error`
///   - Generates `tracing::*!(...)` calls with error details
///   - Compatible with `RUST_LOG` for filtering
///   - Not allowed with `transparent` variants
///
/// ### Tracing Output
/// The generated logs include:
/// - `error_type`: The variant name as string
/// - `status_code`: The HTTP status code
/// - For unnamed variants (single field): `error = ?field` (debug format)
/// - For named variants: Each field as `field_name = ?field_value`
/// - Unit variants: Only `error_type` and `status_code`
///
/// # Example
///
/// ```rust,ignore
/// use sword::prelude::*;
/// use thiserror::Error;
///
/// #[derive(Debug, Error, HttpError)]
/// pub enum ApiError {
///     #[error("Not found")]
///     #[http(code = 404)]
///     #[tracing(info)]  // Log: error_type="NotFound", status_code=404
///     NotFound,
///
///     #[error("Forbidden: requires {role}")]
///     #[http(code = 403, error = role)]
///     #[tracing(warn)]  // Log: error_type="Forbidden", status_code=403, role=?role
///     Forbidden { role: String },
///
///     #[error("IO Error: {0}")]
///     #[http(code = 500)]
///     #[tracing(error)]  // Log: error_type="Io", status_code=500, error=?_inner
///     Io(#[from] std::io::Error),
///
///     #[error("Auth Error: {0}")]
///     #[http(transparent)]  // Delegates to other "HttpError" derivation
///     Auth(#[from] AuthError),
/// }
/// ```
#[proc_macro_derive(HttpError, attributes(http, tracing))]
pub fn derive_http_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match errors::derive_http_error(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Derive macro for gRPC error enums.
///
/// Generates:
/// - `From<Self> for tonic::Status`
///
/// Supported attributes per variant:
/// - `#[grpc(code = "invalid_argument")]`
/// - `#[grpc(message = "custom text")]`
/// - `#[grpc(message = field_name)]`
/// - `#[grpc(transparent)]`
/// - `#[tracing(level)]`
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
/// enum UserError {
///     #[grpc(code = "not_found")]
///     #[error("User not found: {id}")]
///     NotFound { id: String },
///
///     #[grpc(code = "invalid_argument", message = "Invalid input")]
///     #[error("Validation error: {0}")]
///     Validation(String),
///
///     #[grpc(transparent)]
///     #[error("Database error: {0}")]
///     Database(#[from] anyhow::Error),
/// }
/// ```
#[proc_macro_derive(GrpcError, attributes(grpc, tracing))]
pub fn derive_grpc_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match errors::derive_grpc_error(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// ### This is just a re-export of `tokio::main` to simplify the initial setup of
/// ### Sword, you can use your own version of tokio adding it to your
/// ### `Cargo.toml`, we are providing this initial base by default
///
/// ---
///
/// Marks async function to be executed by the selected runtime. This macro
/// helps set up a `Runtime` without requiring the user to use
/// [Runtime](../tokio/runtime/struct.Runtime.html) or
/// [Builder](../tokio/runtime/struct.Builder.html) directly.
///
/// Note: This macro is designed to be simplistic and targets applications that
/// do not require a complex setup. If the provided functionality is not
/// sufficient, you may be interested in using
/// [Builder](../tokio/runtime/struct.Builder.html), which provides a more
/// powerful interface.
///
/// Note: This macro can be used on any function and not just the `main`
/// function. Using it on a non-main function makes the function behave as if it
/// was synchronous by starting a new runtime each time it is called. If the
/// function is called often, it is preferable to create the runtime using the
/// runtime builder so the runtime can be reused across calls.
///
/// # Non-worker async function
///
/// Note that the async function marked with this macro does not run as a
/// worker. The expectation is that other tasks are spawned by the function here.
/// Awaiting on other futures from the function provided here will not
/// perform as fast as those spawned as workers.
///
/// # Multi-threaded runtime
///
/// To use the multi-threaded runtime, the macro can be configured using
///
/// ```rust,ignore
/// #[tokio::main(flavor = "multi_thread", worker_threads = 10)]
/// # async fn main() {}
/// ```
///
/// The `worker_threads` option configures the number of worker threads, and
/// defaults to the number of cpus on the system. This is the default flavor.
///
/// Note: The multi-threaded runtime requires the `rt-multi-thread` feature
/// flag.
///
/// # Current thread runtime
///
/// To use the single-threaded runtime known as the `current_thread` runtime,
/// the macro can be configured using
///
/// ```rust,ignore
/// #[tokio::main(flavor = "current_thread")]
/// # async fn main() {}
/// ```
///
/// ## Function arguments:
///
/// Arguments are allowed for any functions aside from `main` which is special
///
/// ## Usage
///
/// ### Using the multi-thread runtime
///
/// ```ignore
/// #[tokio::main]
/// async fn main() {
///     println!("Hello world");
/// }
/// ```
///
/// Equivalent code not using `#[tokio::main]`
///
/// ```ignore
/// fn main() {
///     tokio::runtime::Builder::new_multi_thread()
///         .enable_all()
///         .build()
///         .unwrap()
///         .block_on(async {
///             println!("Hello world");
///         })
/// }
/// ```
///
/// ### Using current thread runtime
///
/// The basic scheduler is single-threaded.
///
/// ```ignore
/// #[tokio::main(flavor = "current_thread")]
/// async fn main() {
///     println!("Hello world");
/// }
/// ```
///
/// Equivalent code not using `#[tokio::main]`
///
/// ```ignore
/// fn main() {
///     tokio::runtime::Builder::new_current_thread()
///         .enable_all()
///         .build()
///         .unwrap()
///         .block_on(async {
///             println!("Hello world");
///         })
/// }
/// ```
///
/// ### Set number of worker threads
///
/// ```ignore
/// #[tokio::main(worker_threads = 2)]
/// async fn main() {
///     println!("Hello world");
/// }
/// ```
///
/// Equivalent code not using `#[tokio::main]`
///
/// ```ignore
/// fn main() {
///     tokio::runtime::Builder::new_multi_thread()
///         .worker_threads(2)
///         .enable_all()
///         .build()
///         .unwrap()
///         .block_on(async {
///             println!("Hello world");
///         })
/// }
/// ```
///
/// ### Configure the runtime to start with time paused
///
/// ```ignore
/// #[tokio::main(flavor = "current_thread", start_paused = true)]
/// async fn main() {
///     println!("Hello world");
/// }
/// ```
///
/// Equivalent code not using `#[tokio::main]`
///
/// ```ignore
/// fn main() {
///     tokio::runtime::Builder::new_current_thread()
///         .enable_all()
///         .start_paused(true)
///         .build()
///         .unwrap()
///         .block_on(async {
///             println!("Hello world");
///         })
/// }
/// ```
///
/// Note that `start_paused` requires the `test-util` feature to be enabled.
///
/// ### Rename package
///
/// ```ignore
/// use tokio as tokio1;
///
/// #[tokio1::main(crate = "tokio1")]
/// async fn main() {
///     println!("Hello world");
/// }
/// ```
///
/// Equivalent code not using `#[tokio::main]`
///
/// ```ignore
/// use tokio as tokio1;
///
/// fn main() {
///     tokio1::runtime::Builder::new_multi_thread()
///         .enable_all()
///         .build()
///         .unwrap()
///         .block_on(async {
///             println!("Hello world");
///         })
/// }
/// ```
///
/// ### Configure unhandled panic behavior
///
/// Available options are `shutdown_runtime` and `ignore`. For more details, see
/// [`Builder::unhandled_panic`].
///
/// This option is only compatible with the `current_thread` runtime.
///
/// ```no_run, ignore
/// # #![allow(unknown_lints, unexpected_cfgs)]
/// #[cfg(tokio_unstable)]
/// #[tokio::main(flavor = "current_thread", unhandled_panic = "shutdown_runtime")]
/// async fn main() {
///     let _ = tokio::spawn(async {
///         panic!("This panic will shutdown the runtime.");
///     }).await;
/// }
/// # #[cfg(not(tokio_unstable))]
/// # fn main() { }
/// ```
///
/// Equivalent code not using `#[tokio::main]`
///
/// ```no_run, ignore
/// # #![allow(unknown_lints, unexpected_cfgs)]
/// #[cfg(tokio_unstable)]
/// fn main() {
///     tokio::runtime::Builder::new_current_thread()
///         .enable_all()
///         .unhandled_panic(UnhandledPanic::ShutdownRuntime)
///         .build()
///         .unwrap()
///         .block_on(async {
///             let _ = tokio::spawn(async {
///                 panic!("This panic will shutdown the runtime.");
///             }).await;
///         })
/// }
/// # #[cfg(not(tokio_unstable))]
/// # fn main() { }
/// ```
///
/// **Note**: This option depends on Tokio's [unstable API][unstable]. See [the
/// documentation on unstable features][unstable] for details on how to enable
/// Tokio's unstable features.
///
/// [`Builder::unhandled_panic`]: ../tokio/runtime/struct.Builder.html#method.unhandled_panic
/// [unstable]: ../tokio/index.html#unstable-features
#[proc_macro_attribute]
pub fn main(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemFn);

    let fn_body = input.block.clone();
    let fn_attrs = input.attrs.clone();
    let fn_vis = input.vis.clone();
    let _fn_sig = input.sig;

    #[allow(unused)]
    let mut output = quote! {};

    if cfg!(feature = "hot-reload") {
        output = quote! {

            async fn __internal_main() {
                #fn_body
            }

            #(#fn_attrs)*
            #fn_vis fn main() {
                ::sword::internal::tokio_runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap_or_else(|err| {
                        ::sword::internal::core::sword_error!(
                            title: "Failed to build Tokio runtime",
                            reason: err,
                            context: {
                                "source" => "#[sword::main]",
                            },
                        )
                    })
                    .block_on(::sword::internal::dioxus_devtools::serve_subsecond(__internal_main))
            }
        };
    } else {
        output = quote! {
            #(#fn_attrs)*
            #fn_vis fn main() {
                ::sword::internal::tokio_runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap_or_else(|err| {
                        ::sword::internal::core::sword_error!(
                            title: "Failed to build Tokio runtime",
                            reason: err,
                            context: {
                                "source" => "#[sword::main]",
                            },
                        )
                    })
                    .block_on( async #fn_body )
            }
        };
    }

    output.into()
}

#[cfg(feature = "socketio-controllers")]
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
#[proc_macro_attribute]
pub fn on(attr: TokenStream, item: TokenStream) -> TokenStream {
    controllers::expand_on_handler(attr, item).unwrap_or_else(|err| err.to_compile_error().into())
}
