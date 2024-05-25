# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.1](https://github.com/near/near-sandbox/compare/v0.7.0...v0.7.1) - 2024-04-01

### Added
- Update default nearcore version to v1.38.0 ([#81](https://github.com/near/near-sandbox/pull/81))

## [0.7.0](https://github.com/near/near-sandbox/compare/v0.6.3...v0.7.0) - 2023-10-04

### Added
- use tokio instead of async-process as dependants use tokio runtime anyway ([#68](https://github.com/near/near-sandbox/pull/68))

### Fixed
- pin async-process crate ([#66](https://github.com/near/near-sandbox/pull/66))

### Other
- use SANDBOX_ARTIFACT_URL ([#74](https://github.com/near/near-sandbox/pull/74))

## [0.6.3](https://github.com/near/sandbox/compare/v0.6.2...v0.6.3) - 2023-09-30

### Added
- Expose DEFAULT_NEAR_SANDBOX_VERSION const
- run sandbox instance with --fast flag ([#56](https://github.com/near/sandbox/pull/56))
- Allow to specify verion of neard-sandbox ([#63](https://github.com/near/sandbox/pull/63))

### Other
- Fixed linting warnings
- point nearcore to latest mainnet release 1.35.0 ([#61](https://github.com/near/sandbox/pull/61))
- Update crate/Cargo.toml
- update dependencies
