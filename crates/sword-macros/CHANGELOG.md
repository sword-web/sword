# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.0](https://github.com/sword-web/sword/compare/sword-macros-v0.5.0...sword-macros-v0.6.0) - 2026-11-13

### Added

- *(main)* return ExitCode and report once
- *(events)* added generic types check for event macro
- *(events)* handle both empty and invalid chars from event macro
- *(events)* implement NATS transport with queue groups
- prepare sword-events and its macros to NATs
- added events-nats feature flag

### Fixed

- *(macros)* add stable locations to grpc, events and jobs
- *(macros)* add stable locations to generated diagnostics
- *(di)* resolve component locations on stable
- *(events)* apply clippy idiomatic code suggestions
- *(events)* handle wildcards with &dyn Event

### Other

- *(events)* [**breaking**] rename source to transport
- *(imports)* group by std, extern, sword and crate
- *(api)* [**breaking**] reorganize public, prelude and internal visibility
- *(validation)* update README.md
- apply some clippy and nursery fixes
- *(diagnostics)* remove repeated text from reports
- *(diagnostics)* migrate facade and remove the legacy macro
- *(socketio)* migrate generated handler diagnostics
- *(jobs)* migrate generated job diagnostics
- *(events)* migrate generated event diagnostics
- *(grpc)* migrate generated grpc diagnostics
- *(web)* migrate generated web diagnostics
- *(macros)* make config registration fallible
- document application and sub-features in README
- [**breaking**] assemble the events engine and rename controller kinds
- *(events)* [**breaking**] make transports pluggable
- *(events)* [**breaking**] rename handle macro to subscribe

## [0.5.0](https://github.com/sword-web/sword/compare/sword-macros-v0.4.1...sword-macros-v0.5.0) - 2026-10-13

### Added

- *(jobs)* handlers auto-registering and runtime initialization
- *(errors)* derive tracing level from status code

### Other

- unify crate metadata and README
- *(macros)* replace sword-macros with the rewritten crate

## [0.4.1](https://github.com/sword-web/sword/compare/sword-macros-v0.4.0...sword-macros-v0.4.1) - 2026-09-06

### Other

- *(deps)* bump syn from 2.0.118 to 3.0.3

## [0.4.0](https://github.com/sword-web/sword/compare/sword-macros-v0.3.0...sword-macros-v0.4.0) - 2026-08-10

### Added

- *(web)* added sse support for web controllers

### Other

- *(web)* use EventStream trait instead of SseResult alias
- release v0.3.0

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
