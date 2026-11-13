# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.0](https://github.com/sword-web/sword/compare/sword-socketio-v0.5.0...sword-socketio-v0.6.0) - 2026-11-13

### Added

- *(config)* validate sections before startup

### Other

- *(imports)* group by std, extern, sword and crate
- *(api)* [**breaking**] reorganize public, prelude and internal visibility
- *(socketio)* move the parser stack to socketioxide 0.18.9
- *(validation)* update README.md
- *(validation)* [**breaking**] rename feature-flags across sword crates
- *(diagnostics)* remove repeated text from reports
- *(diagnostics)* migrate facade and remove the legacy macro
- *(socketio)* migrate generated handler diagnostics
- *(macros)* make config registration fallible
- *(socketio)* validate the socketio section
- *(web)* validate the web section and make extensions fallible
- document application and sub-features in README

## [0.5.0](https://github.com/sword-web/sword/compare/sword-socketio-v0.4.1...sword-socketio-v0.5.0) - 2026-10-13

### Other

- unify crate metadata and README
- *(macros)* replace sword-macros with the rewritten crate

## [0.4.1](https://github.com/sword-web/sword/compare/sword-socketio-v0.4.0...sword-socketio-v0.4.1) - 2026-09-06

### Fixed

- bump socketioxide version and update socketioxide-parser builder assignment

## [0.4.0](https://github.com/sword-web/sword/compare/sword-socketio-v0.3.0...sword-socketio-v0.4.0) - 2026-08-10

### Other

- update Cargo.toml dependencies
