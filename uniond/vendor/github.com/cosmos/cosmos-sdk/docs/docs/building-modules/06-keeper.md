---
sidebar_position: 1
---

# Keepers

:::note Synopsis
`Keeper`s refer to a Cosmos SDK abstraction whose role is to manage access to the subset of the state defined by various modules. `Keeper`s are module-specific, i.e. the subset of state defined by a module can only be accessed by a `keeper` defined in said module. If a module needs to access the subset of state defined by another module, a reference to the second module's internal `keeper` needs to be passed to the first one. This is done in `app.go` during the instantiation of module keepers.
:::

:::note Pre-requisite Readings

* [Introduction to Cosmos SDK Modules](./01-intro.md)

:::

## Motivation

The Cosmos SDK is a framework that makes it easy for developers to build complex decentralized applications from scratch, mainly by composing modules together. As the ecosystem of open-source modules for the Cosmos SDK expands, it will become increasingly likely that some of these modules contain vulnerabilities, as a result of the negligence or malice of their developer.

The Cosmos SDK adopts an [object-capabilities-based approach](../core/10-ocap.md) to help developers better protect their application from unwanted inter-module interactions, and `keeper`s are at the core of this approach. A `keeper` can be considered quite literally to be the gatekeeper of a module's store(s). Each store (typically an [`IAVL` Store](../core/04-store.md#iavl-store)) defined within a module comes with a `storeKey`, which grants unlimited access to it. The module's `keeper` holds this `storeKey` (which should otherwise remain unexposed), and defines [methods](#implementing-methods) for reading and writing to the store(s).

The core idea behind the object-capabilities approach is to only reveal what is necessary to get the work done. In practice, this means that instead of handling permissions of modules through access-control lists, module `keeper`s are passed a reference to the specific instance of the other modules' `keeper`s that they need to access (this is done in the [application's constructor function](../basics/00-app-anatomy.md#constructor-function)). As a consequence, a module can only interact with the subset of state defined in another module via the methods exposed by the instance of the other module's `keeper`. This is a great way for developers to control the interactions that their own module can have with modules developed by external developers.

## Type Definition

`keeper`s are generally implemented in a `/keeper/keeper.go` file located in the module's folder. By convention, the type `keeper` of a module is simply named `Keeper` and usually follows the following structure:

```go
type Keeper struct {
    // External keepers, if any

    // Store key(s)

    // codec

    // authority 
}
```

For example, here is the type definition of the `keeper` from the `staking` module:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/staking/keeper/keeper.go#L23-L31
```

Let us go through the different parameters:

* An expected `keeper` is a `keeper` external to a module that is required by the internal `keeper` of said module. External `keeper`s are listed in the internal `keeper`'s type definition as interfaces. These interfaces are themselves defined in an `expected_keepers.go` file in the root of the module's folder. In this context, interfaces are used to reduce the number of dependencies, as well as to facilitate the maintenance of the module itself.
* `storeKey`s grant access to the store(s) of the [multistore](../core/04-store.md) managed by the module. They should always remain unexposed to external modules.
* `cdc` is the [codec](../core/05-encoding.md) used to marshall and unmarshall structs to/from `[]byte`. The `cdc` can be any of `codec.BinaryCodec`, `codec.JSONCodec` or `codec.Codec` based on your requirements. It can be either a proto or amino codec as long as they implement these interfaces. The authority listed is a module account or user account that has the right to change module level parameters. Previously this was handled by the param module, which has been deprecated.

Of course, it is possible to define different types of internal `keeper`s for the same module (e.g. a read-only `keeper`). Each type of `keeper` comes with its own constructor function, which is called from the [application's constructor function](../basics/00-app-anatomy.md). This is where `keeper`s are instantiated, and where developers make sure to pass correct instances of modules' `keeper`s to other modules that require them.

## Implementing Methods

`Keeper`s primarily expose getter and setter methods for the store(s) managed by their module. These methods should remain as simple as possible and strictly be limited to getting or setting the requested value, as validity checks should have already been performed via the `ValidateBasic()` method of the [`message`](./02-messages-and-queries.md#messages) and the [`Msg` server](./03-msg-services.md) when `keeper`s' methods are called.

Typically, a *getter* method will have the following signature

```go
func (k Keeper) Get(ctx sdk.Context, key string) returnType
```

and the method will go through the following steps:

1. Retrieve the appropriate store from the `ctx` using the `storeKey`. This is done through the `KVStore(storeKey sdk.StoreKey)` method of the `ctx`. Then it's preferred to use the `prefix.Store` to access only the desired limited subset of the store for convenience and safety.
2. If it exists, get the `[]byte` value stored at location `[]byte(key)` using the `Get(key []byte)` method of the store.
3. Unmarshall the retrieved value from `[]byte` to `returnType` using the codec `cdc`. Return the value.

Similarly, a *setter* method will have the following signature

```go
func (k Keeper) Set(ctx sdk.Context, key string, value valueType)
```

and the method will go through the following steps:

1. Retrieve the appropriate store from the `ctx` using the `storeKey`. This is done through the `KVStore(storeKey sdk.StoreKey)` method of the `ctx`. It's preferred to use the `prefix.Store` to access only the desired limited subset of the store for convenience and safety.
2. Marshal `value` to `[]byte` using the codec `cdc`.
3. Set the encoded value in the store at location `key` using the `Set(key []byte, value []byte)` method of the store.

For more, see an example of `keeper`'s [methods implementation from the `staking` module](https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/staking/keeper/keeper.go).

The [module `KVStore`](../core/04-store.md#kvstore-and-commitkvstore-interfaces) also provides an `Iterator()` method which returns an `Iterator` object to iterate over a domain of keys.

This is an example from the `auth` module to iterate accounts:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/auth/keeper/account.go#L94-L108
```
