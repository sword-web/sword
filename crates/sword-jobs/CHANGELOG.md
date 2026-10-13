# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0](https://github.com/sword-web/sword/compare/sword-jobs-v0.4.1...sword-jobs-v0.5.0) - 2026-10-13

### Added

- *(jobs)* handlers auto-registering and runtime initialization
- add Axum-style extractors with fixed State and returns macro
- :label: added multi lib validation system with `garde` and `validator`
- :sparkles: added project related markdown files (contributing, roadmap)
- :ambulance: use http body util to check body size limit
- :zap: better multipart handling with body limit and mime type allow list
- add hot reloading support
- :art: stable multipart abstraction api
- added body limit and content type check
- :ambulance: improve error handling with `thiserror` crate
- :fire: stable middleware support and state extraction via `TypeId`
- add controller macro and remove use of panic!
- :zap: First commit

### Fixed

- :memo: remove hot-reload features to publish into crates.io
- :ambulance: examples update and others
- :art: optional query support and validated query now is also optional
- :adhesive_bandage: remove global config support to improve it in future versions

### Other

- unify crate metadata and README
- *(runtime)* unify engine orchestration
- *(macros)* replace sword-macros with the rewritten crate
- change some display config behaviors
- Revise README with updated features and remove 'Coming Soon'
- *(docs)* update workflow and align project docs
- rename web result aliases
- reorganize application engines and simplify startup config output
- rename examples and align docs with controllers
- rename web runtime and controllers adapter
- release v0.2.1
- update README.md and CHANGELOG.md
- update .md files
- remove cookies feature flag
- update `.md` files including new release features
- :fire: use of Arc<T> to inject dependencies without cloning them
- Update README with active development notice
- Revise README formatting and content
- update README.md
- :art: added changelog and readme file update
- Merge branch 'main' into dev
- Update README.md
- Update README.md
- Update README.md
- Update README.md
- :art: pre-release commit
- :ambulance: `0.1.3` version released
- :art: added `docs` and 0.1.3 release
- :zap: general refactor `context` api and `di` pattern
- update README.md
