# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0](https://github.com/sword-web/sword/compare/sword-macros-v0.2.1...sword-macros-v0.3.0) - 2026-08-03

### Added

- *(events)* rework EventHandler API and fix CMetaStack
- added in memory event handler with tokio mpsc
- add interpolated message support in #[http_error] and #[grpc_error]
- add InjectableTrait for trait-object DI and #[contract] macro
- auto-wrap handler return values with Result<T, JsonResponse>
- auto-register compression/cors layers via SwordLayerRegistrar
- *(runtime)* support tokio runtime config from application metadata

### Fixed

- *(sword-macros)* compile with no controller features enabled
- *(release)* per-crate READMEs, inherited MIT license, publish=false for unpublished crates
- remove multiple pieces of dead code
- *(events)* gate EventSourceKind behind event-handlers feature
- expansion ordering with 'CmetaStack'
- *(macros)* allow dead_code in cmeta.rs
- *(config)* align tracing keys and improve startup diagnostics
- *(runtime)* parse runtime in macro
- error in layer and interceptor chain
- align clippy checks with feature matrices
- added fatal log display when tracing is disabled

### Other

- update nested if let syntax to new standard
- remove #[contract] macro and trait-object DI bindings
- remove 'async_trait' from public reexport and update docs
- remove  from public reexport and update docs
- *(socketio)* reduce Arc clones in connection handler + fix(interceptor): add missing HasDeps
- *(socketio)* flatten SocketRef into SocketContext and add SocketKind enum
- remove runtime configuration with toml
- change some display config behaviors
- *(errors)* improve error handling macros
- *(errors)* improve error handling macros
- *(config)* separate engine settings from application
- *(features)* align reflection and crate feature flags
- reorganize sword into internal crates and split tests by controller type
