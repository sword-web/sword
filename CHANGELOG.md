# Sword web framework changelog

## [Unreleased]

### Added

- Added `StreamRequest` extractor and stream interceptor traits (`OnRequestStream`, `OnRequestStreamWithConfig`) for non-buffered request handling.
- Added app type feature naming foundation: `web-controllers` and `grpc-controllers`.

- New `next()` method on `Request` struct. See `Changed` section for more details.
- Added `Module` trait for creating and registering controllers and injectables as a modules.

- Added `ServeDir` middleware for serving static files from a directory. This middleware uses `tower_http::services::ServeDir` under the hood.

- Added cleanup for temporary data structures used in the application build process.

- Added `Cors` middleware based on `tower_http::cors::CorsLayer`. The configuration can be set in the config file under the `cors` key.

- Added `middlewares` field to config file. This allows to define global middlewares in the configuration file.

- Added `Compression` middleware based on `tower_http::compression::CompressionLayer`. The configuration can be set in the config file under the `compression` key.

- Added `ControllerRegistry` system for managing multiple controller instances in the application and modules.

- Added `RequestId` middleware for generating and attaching unique request IDs to each incoming request. This middleware uses the tower-http `RequestIdLayer` with a UUID generator under the hood.

- Added unified error handling macros that works with `thiserror` crate. It allows to map custom errors to standarized JSON error responses.

- Added `SocketIo` controllers for handling SocketIO connections.

- Added `Interceptor` trait for creating custom interceptors that can access and modify requests and responses.

### Changed

- Aligned current naming across docs/examples/changelog: engine config now lives in `[web]`, `[grpc]`, and `[socketio]`, the router prefix key is `router-prefix`, Socket.IO transport configuration uses `transports`, and Sword terminology now distinguishes Tower `layers` from typed `interceptors`.
- **BREAKING:** Renamed web interceptor return alias from `HttpInterceptorResult` to `WebInterceptorResult`.
- **BREAKING:** Renamed web controller result alias from `Result` to `WebResult`.
- Renamed adapter/runtime naming across the codebase: HTTP controllers now use `controllers` naming, and the primary application module is now `application::web`.
- Refactored runtime config so `[application]` keeps only global app settings while engine settings live under `[web]` and `[grpc]`.
- Runtime now applies only mandatory built-ins by default (`not_found`, `body_limit`, `request_timeout`); optional layers are explicit via `.with_layer(...)`.
- Replaced `HttpResult` usage with `WebResult` alias (`WebResult<T = JsonResponse, E = JsonResponse>`) in controller APIs.
- Updated macros/docs/examples to match new naming and APIs, including workspace/metadata consistency fixes.

- Replaced native `RwLock` with `parking_lot::RwLock` for better performance.

- The Application config was refactored and splitted into multiple smaller config structs.

- The `next!` macro has been removed. Instead, use the `req.next().await` method to pass control to the next middleware or handler in the chain. This change removes the need for a macro to do "magic" and makes the code more explicit and easier to understand.

- Middleware structs marked with `#[middleware]` macro now can omit the `next` parameter in their methods. Instead, they can call `req.next().await` to pass control to the next middleware or handler.

- The global router prefix is configured under `[web]` using `router-prefix`.

- The `Request Timeout` middleware now can be configured with more human-friendly duration format using `duration-str` crate.

- The associated type in `Module` trait is not required anymore. Use the `register_controllers` method to register web controllers and other controller kinds.

- Removed wrapper types for Axum extractors (`Method`, `Uri`, `Headers`, `Bytes`). These types now use Axum's native types directly with `FromRequest`/`FromRequestParts` trait implementations, eliminating unnecessary encapsulation.

## [0.2.0]

### Added

- Added `Non static handlers for controllers` support. Now, controllers must have `&self` as the first parameter in their methods. This allows to use struct fields that are extracted from the application state.

- Added schema validation with feature flags. Now the `validator` crate is included only under `validator` feature flag. This allows users to choose if they want to use `validator` crate or not. If not, you can implement your own trait for validation to the `Request` struct. e.g. with `garde`, `validify`.

- Added `global prefix` support. Now, you can set a global prefix for all routes in the application. This is useful for versioning or grouping routes under a common path.

- Added versioning support on controllers with `version` attribute.

- Added `hot-reload` feature flag to `sword`. This enables hot-reloading of the application during development. It uses the `subsecond` and `dioxus-devtools` crates for hot-reloading functionality. See the examples for usage.

- Non static middlewares support. Now, middlewares can also have `&self` as the first parameter in their methods. This allows to use struct fields that are extracted from the dependency injection system.

- `OnRequest` and `OnRequestWithConfig` traits for creating custom middlewares.

- `#[uses]` macro attribute to apply middlewares to controllers.

- Added injection support for middlewares. Now, middlewares can have dependencies injected from the DI system.

- Added injection support for custom configuration types marked with `#[config]` macro.

- Added `DependencyContainer` struct that allows to register components and providers. This struct is used internally by the DI system to manage dependencies.

### Fixed

- Fixed an issue where the middleware macro was not working correctly with some configuration types.

- Fixed the error messages when some macros failed to compile. Now, the error messages are more descriptive and helpful.

### Changed

- With the latest `axum_responses` release, the `data` field in error responses has been removed and replaced with either `error` or `errors`, depending on your configuration. By default, validation errors will be returned under `errors` fields.

- Changed global state scope. Now its necessary to use DI pattern.

- Changed `Context` by `Request`. This change was made because at the first time `Context` was used to handle request, state, and config together. But now, with the new features added, it was more appropriate to use `Request`.

- Change the purpose of `#[middleware]`. Now, it's used to declare a middleware struct instead of applying middlewares to controllers. To apply middlewares to controllers, use the new `#[uses]` attribute.

- Change `HttpResult` from `Result<T, HttpResponse>` to `Result<HttpResponse, HttpResponse>`. This change was made because the main usage of `HttpResult` is to return HTTP responses, not arbitrary types.

## [0.1.8]

### Added

- Added `helmet` feature to `sword`. This allows users to enable security-related HTTP headers for their applications. It's built on top of the `rust-helmet` and `axum-helmet` crates.

- Added built-in `Timeout` middleware based on `tower_http` TimeoutLayer. This middleware allows users to set a timeout duration for requests, enhancing the robustness of their applications. The number of seconds can be configured on the config .toml file at the `application` key. See the examples for usage.

- Added documentation comments to all public functions and structs in the `sword` crate. This improves code readability and helps users understand the functionality of the framework better.

- Added `cookies` feature flag to `sword`. This enables cookie parsing and management. It uses the `tower-cookies` crate for cookie handling. This feature allows users to use Cookies, PrivateCookies, and SignedCookies in their handlers. See the examples for usage.

- Added `multipart` feature flag to `sword`. This enables multipart form data handling using the `multipart` feature flag of `axum` crate. Basically it provides `Multipart` extractor for handling file uploads and form data.

- Added support for axum run with graceful shutdown. This allows the application to handle shutdown signals gracefully, ensuring that ongoing requests are completed before the server stops.

- Added `tower_http` layers support to `middleware macro`. This allows users to easily add middleware layers from the `tower_http` to controllers using the `#[middleware]` attribute.

### Changed

- Move `hot-reload` functionality to another branch due to its constantly changing development state.

- Changed the `serde_qs` dependency to native `axum` query extraction. This simplifies the codebase and reduces dependencies.

- Changed the `#[controller_impl]` macro to `#[routes]`. This change improves clarity and consistency in the codebase.

- Changed the builder pattern for `Application` struct. Now, all the build methods start with `with_` prefix. For example, `with_layer`, `with_controller`, etc. This change enhances code readability and consistency.
