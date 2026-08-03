# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.1](https://github.com/sword-web/sword/compare/sword-socketio-v0.3.0...sword-socketio-v0.3.1) - 2026-08-03

### Added

- *(socketio)* add cookie support to SocketContext and fix CookieManagerLayer ordering
- *(socketio)* update socketioxide dep to the last release
- *(socketio)* added http headers and authorization shortcuts

### Fixed

- *(sword-socketio)* remove invalid crates.io keyword with a dot
- *(release)* per-crate READMEs, inherited MIT license, publish=false for unpublished crates
- *(socketio)* added missing Sized trait bound for emit method
- ci fmt error
- apply socketio layer after router prefix and enable CORS in example
- *(logging)* added missing description for sword-layer logs

### Other

- *(release)* publish all sword crates to make the sword facade usable
- update nested if let syntax to new standard
- *(socketio)* flatten SocketRef into SocketContext and add SocketKind enum
- *(web)* add extension context lifecycle
- change some display config behaviors
- *(socketio)* remove 'enabled' config option
- *(features)* align reflection and crate feature flags
- reorganize sword into internal crates and split tests by controller type
