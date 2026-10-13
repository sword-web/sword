# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0](https://github.com/sword-web/sword/compare/sword-web-v0.4.1...sword-web-v0.5.0) - 2026-10-13

### Added

- *(web)* add configurable web access logger

### Fixed

- apply clippy lint rules fixes
- *(docs)* replace axum_responses import by sword::web in web docs

### Other

- unify crate metadata and README
- *(runtime)* unify engine orchestration
- Merge pull request #65 from sword-web/dependabot/cargo/dependencies-490f5ea562
- *(macros)* replace sword-macros with the rewritten crate
- *(deps)* bump the dependencies group across 1 directory with 3 updates
- *(web)* document application composition flow
- *(deps)* remove sword-web unused dependencies

## [0.4.1](https://github.com/sword-web/sword/compare/sword-web-v0.4.0...sword-web-v0.4.1) - 2026-09-06

### Other

- update Cargo.toml dependencies

## [0.4.0](https://github.com/sword-web/sword/compare/sword-web-v0.3.0...sword-web-v0.4.0) - 2026-08-10

### Added

- *(web)* added sse support for web controllers

### Other

- *(web)* use EventStream trait instead of SseResult alias
