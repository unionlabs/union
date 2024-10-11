---
sidebar_position: 0
---

# Packages

The Cosmos SDK is a collection of Go modules. This section provides documentation on various packages that can used when developing a Cosmos SDK chain.
It lists all standalone Go modules that are part of the Cosmos SDK.

:::tip
For more information on SDK modules, see the [SDK Modules](https://docs.cosmos.network/main/modules) section.
For more information on SDK tooling, see the [Tooling](https://docs.cosmos.network/main/build/tooling) section.
:::

## Core

* [Core](https://pkg.go.dev/cosmossdk.io/core) - Core library defining SDK modules and Server core interfaces ([ADR-063](https://docs.cosmos.network/main/architecture/adr-063-core-module-api))
* [API](https://pkg.go.dev/cosmossdk.io/api) - API library containing generated SDK Pulsar API
* [Store](https://pkg.go.dev/cosmossdk.io/store) - Implementation of the Cosmos SDK store
* [Store/v2](https://pkg.go.dev/cosmossdk.io/store/v2) - Implementation of the Cosmos SDK store

## V2

* [Server/v2/stf](https://pkg.go.dev/cosmossdk.io/server/v2/stf) - State Transition Function (STF) library for Cosmos SDK v2
* [Server/v2/appmanager](https://pkg.go.dev/cosmossdk.io/server/v2/appmanager) - App coordinator for Cosmos SDK v2
* [runtime/v2](https://pkg.go.dev/cosmossdk.io/runtime/v2) - Runtime library for Cosmos SDK v2
* [Server/v2](https://pkg.go.dev/cosmossdk.io/server/v2) - Global server library for Cosmos SDK v2
* [Server/v2/cometbft](https://pkg.go.dev/cosmossdk.io/server/v2/cometbft) - CometBFT Server implementation for Cosmos SDK v2

## State Management

* [Collections](./02-collections.md) - State management library
* [ORM](./03-orm.md) - State management library
* [Schema](https://pkg.go.dev/cosmossdk.io/schema) - Logical representation of module state schemas
* [PostgreSQL indexer](https://pkg.go.dev/cosmossdk.io/indexer/postgres) - PostgreSQL indexer for Cosmos SDK modules

## UX

* [Depinject](./01-depinject.md) - Dependency injection framework
* [Client/v2](https://pkg.go.dev/cosmossdk.io/client/v2) - Library powering [AutoCLI](https://docs.cosmos.network/main/core/autocli)

## Utilities

* [Core/Testing](https://pkg.go.dev/cosmossdk.io/core/testing) - Mocking library for SDK modules
* [Log](https://pkg.go.dev/cosmossdk.io/log) - Logging library
* [Errors](https://pkg.go.dev/cosmossdk.io/errors) - Error handling library
* [Errors/v2](https://pkg.go.dev/cosmossdk.io/errors/v2) - Error handling library
* [Math](https://pkg.go.dev/cosmossdk.io/math) - Math library for SDK arithmetic operations

## Example

* [SimApp v2](https://pkg.go.dev/cosmossdk.io/simapp/v2) - SimApp/v2 is **the** sample Cosmos SDK v2 chain. This package should not be imported in your application.
* [SimApp](https://pkg.go.dev/cosmossdk.io/simapp) - SimApp is **the** sample Cosmos SDK chain. This package should not be imported in your application.
