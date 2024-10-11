---
sidebar_position: 1
---

# Module Genesis

:::note Synopsis
Modules generally handle a subset of the state and, as such, they need to define the related subset of the genesis file as well as methods to initialize, verify and export it.
:::

:::note Pre-requisite Readings

* [Module Manager](./01-module-manager.md)
* [Keepers](./06-keeper.md)

:::

## Type Definition

The subset of the genesis state defined from a given module is generally defined in a `genesis.proto` file ([more info](../../learn/advanced/05-encoding.md#gogoproto) on how to define protobuf messages). The struct defining the module's subset of the genesis state is usually called `GenesisState` and contains all the module-related values that need to be initialized during the genesis process.

See an example of `GenesisState` protobuf message definition from the `auth` module:

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/proto/cosmos/auth/v1beta1/genesis.proto
```

Next we present the main genesis-related methods that need to be implemented by module developers in order for their module to be used in Cosmos SDK applications.

### `DefaultGenesis`

The `DefaultGenesis()` method is a simple method that calls the constructor function for `GenesisState` with the default value for each parameter. See an example from the `auth` module:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/x/auth/module.go#L63-L67
```

### `ValidateGenesis`

The `ValidateGenesis(data GenesisState)` method is called to verify that the provided `genesisState` is correct. It should perform validity checks on each of the parameters listed in `GenesisState`. See an example from the `auth` module:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/x/auth/types/genesis.go#L62-L75
```

## Other Genesis Methods

Other than the methods related directly to `GenesisState`, module developers are expected to implement two other methods as part of the [`AppModuleGenesis` interface](./01-module-manager.md#appmodulegenesis) (only if the module needs to initialize a subset of state in genesis). These methods are [`InitGenesis`](#initgenesis) and [`ExportGenesis`](#exportgenesis).

### `InitGenesis`

The `InitGenesis` method is executed during [`InitChain`](../../learn/advanced/00-baseapp.md#initchain) when the application is first started. Given a `GenesisState`, it initializes the subset of the state managed by the module by using the module's [`keeper`](./06-keeper.md) setter function on each parameter within the `GenesisState`.

The [module manager](./01-module-manager.md#manager) of the application is responsible for calling the `InitGenesis` method of each of the application's modules in order. This order is set by the application developer via the manager's `SetOrderGenesisMethod`, which is called in the [application's constructor function](../../learn/beginner/00-app-anatomy.md#constructor-function).

See an example of `InitGenesis` from the `auth` module:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/452129d6aa45134f598f05be13f3fd961ff9734e/x/auth/keeper/genesis.go#L12-L43
```

### `ExportGenesis`

The `ExportGenesis` method is executed whenever an export of the state is made. It takes the latest known version of the subset of the state managed by the module and creates a new `GenesisState` out of it. This is mainly used when the chain needs to be upgraded via a hard fork.

See an example of `ExportGenesis` from the `auth` module.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/452129d6aa45134f598f05be13f3fd961ff9734e/x/auth/keeper/genesis.go#L45-L60
```

### GenesisTxHandler

`GenesisTxHandler` is a way for modules to submit state transitions prior to the first block. This is used by `x/genutil` to submit the genesis transactions for the validators to be added to staking. 

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/core/genesis/txhandler.go#L3-L6
```
