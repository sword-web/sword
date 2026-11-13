# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.0](https://github.com/sword-web/sword/compare/sword-core-v0.5.0...sword-core-v0.6.0) - 2026-11-13

### Added

- *(config)* validate sections before startup
- *(config)* apply the diagnostics mode from application config
- *(config)* add config validation primitives
- *(diagnostic)* add diagnostic model and renderer
- *(grpc)* add injectable gRPC clients

### Fixed

- *(di)* resolve component locations on stable

### Other

- *(imports)* group by std, extern, sword and crate
- *(api)* [**breaking**] reorganize public, prelude and internal visibility
- *(core)* format dependency type name only on lookup failure
- *(validation)* update README.md
- apply some clippy and nursery fixes
- *(runtime)* remove graceful shutdown
- *(diagnostics)* use a middle dot in the report header
- *(diagnostics)* remove repeated text from reports
- *(diagnostic)* inline reporting and use miette's layout
- *(diagnostics)* drop the catalog reference
- *(diagnostics)* migrate facade and remove the legacy macro
- *(macros)* make config registration fallible
- *(core)* return interceptor build failures
- *(layers)* validate the tracing section
- document application and sub-features in README
- [**breaking**] assemble the events engine and rename controller kinds

## [0.5.0](https://github.com/sword-web/sword/compare/sword-core-v0.4.1...sword-core-v0.5.0) - 2026-10-13

### Added

- *(jobs)* handlers auto-registering and runtime initialization
- *(jobs)* build base sword-job crate with register functions

### Other

- unify crate metadata and README
- *(runtime)* unify engine orchestration
- translate code comments to english
- *(events)* move event init into EventApplicationRuntime
