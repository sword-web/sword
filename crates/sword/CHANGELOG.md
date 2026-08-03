# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.1](https://github.com/sword-web/sword/compare/sword-v0.2.1...sword-v0.3.1) - 2026-08-03

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

- release v0.3.0
- remove #[contract] macro and trait-object DI bindings
- remove  from public reexport and update docs
- remove runtime configuration with toml
- improve ApplicationBuilder process
- *(errors)* improve error handling macros
- *(config)* separate engine settings from application
- *(features)* align reflection and crate feature flags
- reorganize sword into internal crates and split tests by controller type

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
