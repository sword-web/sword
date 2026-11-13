# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.0](https://github.com/sword-web/sword/compare/sword-v0.5.0...sword-v0.6.0) - 2026-11-13

### Added

- *(config)* validate sections before startup
- *(config)* apply the diagnostics mode from application config
- *(diagnostic)* add diagnostic model and renderer
- added events-nats feature flag

### Fixed

- *(di)* resolve component locations on stable

### Other

- *(errors)* inline runtime sword_error diagnostics
- *(imports)* group by std, extern, sword and crate
- *(api)* [**breaking**] reorganize public, prelude and internal visibility
- *(validation)* update README.md
- *(validation)* [**breaking**] rename feature-flags across sword crates
- *(runtime)* remove graceful shutdown
- *(diagnostics)* remove repeated text from reports
- *(diagnostics)* migrate facade and remove the legacy macro
- *(events)* migrate generated event diagnostics
- *(macros)* make config registration fallible
- *(core)* return interceptor build failures
- *(events)* validate the events section
- *(grpc)* validate the grpc section
- *(web)* validate the web section and make extensions fallible
- *(layers)* validate the tracing section
- *(jobs)* return diagnostics from run
- *(events)* return diagnostics from run
- *(grpc)* return diagnostics from run
- *(web)* return diagnostics from run
- *(runtime)* run engines as fallible tasks
- *(runtime)* make router fallible
- document application and sub-features in README
- [**breaking**] assemble the events engine and rename controller kinds
- *(events)* [**breaking**] make transports pluggable
- *(grpc)* cover client injection and end-to-end call

## [0.5.0](https://github.com/sword-web/sword/compare/sword-v0.4.1...sword-v0.5.0) - 2026-10-13

### Added

- *(jobs)* handlers auto-registering and runtime initialization
- *(jobs)* build base sword-job crate with register functions

### Other

- unify crate metadata and README
- *(runtime)* unify engine orchestration
- *(macros)* replace sword-macros with the rewritten crate
- translate code comments to english
- *(events)* move event init into EventApplicationRuntime

## [0.4.1](https://github.com/sword-web/sword/compare/sword-v0.4.0...sword-v0.4.1) - 2026-09-06

### Added

- *(config)* load per-environment config files

## [0.4.0](https://github.com/sword-web/sword/compare/sword-v0.3.0...sword-v0.4.0) - 2026-08-10

### Added

- *(grpc)* add GrpcStatus builder for richer gRPC errors

### Other

- *(deps)* bump safe and low-risk dependencies

## [0.3.0](https://github.com/sword-web/sword/compare/sword-v0.2.1...sword-v0.3.0) - 2026-08-03

### Added

- *(events)* rework EventHandler API and fix CMetaStack
- added in memory event handler with tokio mpsc
- add InjectableTrait for trait-object DI and #[contract] macro
- auto-register compression/cors layers via SwordLayerRegistrar
- add OpenAPI/Swagger UI support with multi-spec configuration
- *(runtime)* support tokio runtime config from application metadata

### Fixed

- *(release)* per-crate READMEs, inherited MIT license, publish=false for unpublished crates
- remove multiple pieces of dead code
- expansion ordering with 'CmetaStack'
- *(events)* always create EventPublisher before build_all() so DI resolves it
- *(runtime)* parse runtime in macro
- error in layer and interceptor chain
- align clippy checks with feature matrices

### Other

- remove #[contract] macro and trait-object DI bindings
- remove  from public reexport and update docs
- remove runtime configuration with toml
- improve ApplicationBuilder process
- *(errors)* improve error handling macros
- *(config)* separate engine settings from application
- *(features)* align reflection and crate feature flags
- reorganize sword into internal crates and split tests by controller type
