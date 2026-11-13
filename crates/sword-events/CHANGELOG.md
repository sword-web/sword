# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.0](https://github.com/sword-web/sword/compare/sword-events-v0.5.0...sword-events-v0.6.0) - 2026-11-13

### Added

- *(events)* add NATS connection, auth and TLS settings
- *(events)* expose NATS connection options
- *(events)* time out handler attempts
- *(events)* bound concurrent handler execution
- *(events)* add concurrency to the events config
- *(config)* validate sections before startup
- *(events)* implement NATS transport with queue groups
- prepare sword-events and its macros to NATs
- added events-nats feature flag

### Fixed

- *(events)* fail startup when a NATS subscription cannot be created
- *(events)* apply clippy idiomatic code suggestions
- *(events)* handle wildcards with &dyn Event

### Other

- *(events)* [**breaking**] rename source to transport
- *(events)* import paths at the top
- *(errors)* inline runtime sword_error diagnostics
- *(imports)* group by std, extern, sword and crate
- *(api)* [**breaking**] reorganize public, prelude and internal visibility
- *(validation)* update README.md
- *(runtime)* remove graceful shutdown
- *(diagnostics)* remove repeated text from reports
- *(diagnostics)* migrate facade and remove the legacy macro
- *(events)* migrate generated event diagnostics
- *(macros)* make config registration fallible
- *(events)* validate the events section
- *(events)* return diagnostics from run
- document application and sub-features in README
- [**breaking**] assemble the events engine and rename controller kinds
- *(events)* [**breaking**] make transports pluggable
- *(events)* [**breaking**] rename handle macro to subscribe

## [0.5.0](https://github.com/sword-web/sword/compare/sword-events-v0.4.1...sword-events-v0.5.0) - 2026-10-13

### Other

- unify crate metadata and README
- *(runtime)* unify engine orchestration
- *(macros)* replace sword-macros with the rewritten crate
- *(events)* move event init into EventApplicationRuntime
