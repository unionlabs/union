---
sidebar_position: 1
---

# Invariants

:::note Synopsis
An invariant is a property of the application that should always be true. In the context of the Cosmos SDK, an `Invariant` is a function that checks for a particular invariant. These functions are useful to detect bugs early on and act upon them to limit their potential consequences (e.g. by halting the chain). They are also useful in the development process of the application to detect bugs via simulations.
:::

:::note Pre-requisite Readings

* [Keepers](./06-keeper.md)

:::

## Implementing `Invariant`s

An `Invariant` is a function that checks for a particular invariant within a module. Module `Invariant`s must follow the `Invariant` type:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/types/invariant.go#L9
```

The `string` return value is the invariant message, which can be used when printing logs, and the `bool` return value is the actual result of the invariant check.

In practice, each module implements `Invariant`s in a `keeper/invariants.go` file within the module's folder. The standard is to implement one `Invariant` function per logical grouping of invariants with the following model:

```go
// Example for an Invariant that checks balance-related invariants

func BalanceInvariants(k Keeper) sdk.Invariant {
	return func(ctx sdk.Context) (string, bool) {
        // Implement checks for balance-related invariants
    }
}
```

Additionally, module developers should generally implement an `AllInvariants` function that runs all the `Invariant`s functions of the module:

```go
// AllInvariants runs all invariants of the module.
// In this example, the module implements two Invariants: BalanceInvariants and DepositsInvariants

func AllInvariants(k Keeper) sdk.Invariant {

	return func(ctx sdk.Context) (string, bool) {
		res, stop := BalanceInvariants(k)(ctx)
		if stop {
			return res, stop
		}

		return DepositsInvariant(k)(ctx)
	}
}
```

Finally, module developers need to implement the `RegisterInvariants` method as part of the [`AppModule` interface](./01-module-manager.md#appmodule). Indeed, the `RegisterInvariants` method of the module, implemented in the `module/module.go` file, typically only defers the call to a `RegisterInvariants` method implemented in the `keeper/invariants.go` file. The `RegisterInvariants` method registers a route for each `Invariant` function in the [`InvariantRegistry`](#invariant-registry):

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/staking/keeper/invariants.go#L12-L22
```

For more, see an example of [`Invariant`s implementation from the `staking` module](https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/staking/keeper/invariants.go).

## Invariant Registry

The `InvariantRegistry` is a registry where the `Invariant`s of all the modules of an application are registered. There is only one `InvariantRegistry` per **application**, meaning module developers need not implement their own `InvariantRegistry` when building a module. **All module developers need to do is to register their modules' invariants in the `InvariantRegistry`, as explained in the section above**. The rest of this section gives more information on the `InvariantRegistry` itself, and does not contain anything directly relevant to module developers.

At its core, the `InvariantRegistry` is defined in the Cosmos SDK as an interface:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/types/invariant.go#L14-L17
```

Typically, this interface is implemented in the `keeper` of a specific module. The most used implementation of an `InvariantRegistry` can be found in the `crisis` module:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/crisis/keeper/keeper.go#L57-L61
```

The `InvariantRegistry` is therefore typically instantiated by instantiating the `keeper` of the `crisis` module in the [application's constructor function](../basics/00-app-anatomy.md#constructor-function).

`Invariant`s can be checked manually via [`message`s](./02-messages-and-queries.md), but most often they are checked automatically at the end of each block. Here is an example from the `crisis` module:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/crisis/abci.go#L12-L21
```

In both cases, if one of the `Invariant`s returns false, the `InvariantRegistry` can trigger special logic (e.g. have the application panic and print the `Invariant`s message in the log).
