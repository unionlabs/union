---
sidebar_position: 1
---

# Keepers

:::note Synopsis
`Keeper`s refer to a Cosmos SDK abstraction whose role is to manage access to the subset of the state defined by various modules. `Keeper`s are module-specific, i.e. the subset of state defined by a module can only be accessed by a `keeper` defined in said module. If a module needs to access the subset of state defined by another module, a reference to the second module's internal `keeper` needs to be passed to the first one. This is done in `app.go` during the instantiation of module keepers.
:::

:::note Pre-requisite Readings

* [Introduction to Cosmos SDK Modules](./00-intro.md)

:::

## Motivation

The Cosmos SDK is a framework that makes it easy for developers to build complex decentralized applications from scratch, mainly by composing modules together. As the ecosystem of open-source modules for the Cosmos SDK expands, it will become increasingly likely that some of these modules contain vulnerabilities, as a result of the negligence or malice of their developer.

The Cosmos SDK adopts an [object-capabilities-based approach](https://docs.cosmos.network/main/learn/advanced/ocap#ocaps-in-practice) to help developers better protect their application from unwanted inter-module interactions, and `keeper`s are at the core of this approach. A `keeper` can be considered quite literally to be the gatekeeper of a module's store(s). Each store (typically an [`IAVL` Store](../../learn/advanced/04-store.md#iavl-store)) defined within a module comes with a `storeKey`, which grants unlimited access to it. The module's `keeper` holds this `storeKey` (which should otherwise remain unexposed), and defines [methods](#implementing-methods) for reading and writing to the store(s).

The core idea behind the object-capabilities approach is to only reveal what is necessary to get the work done. In practice, this means that instead of handling permissions of modules through access-control lists, module `keeper`s are passed a reference to the specific instance of the other modules' `keeper`s that they need to access (this is done in the [application's constructor function](../../learn/beginner/00-app-anatomy.md#constructor-function)). As a consequence, a module can only interact with the subset of state defined in another module via the methods exposed by the instance of the other module's `keeper`. This is a great way for developers to control the interactions that their own module can have with modules developed by external developers.

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
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/staking/keeper/keeper.go#L54-L115 
```

Let us go through the different parameters:

* An expected `keeper` is a `keeper` external to a module that is required by the internal `keeper` of said module. External `keeper`s are listed in the internal `keeper`'s type definition as interfaces. These interfaces are themselves defined in an `expected_keepers.go` file in the root of the module's folder. In this context, interfaces are used to reduce the number of dependencies, as well as to facilitate the maintenance of the module itself.
* `KVStoreService`s grant access to the store(s) of the [multistore](../../learn/advanced/04-store.md) managed by the module. They should always remain unexposed to external modules.
* `cdc` is the [codec](../../learn/advanced/05-encoding.md) used to marshal and unmarshal structs to/from `[]byte`. The `cdc` can be any of `codec.BinaryCodec`, `codec.JSONCodec` or `codec.Codec` based on your requirements. It can be either a proto or amino codec as long as they implement these interfaces.
* The authority listed is a module account or user account that has the right to change module level parameters. Previously this was handled by the param module, which has been deprecated.

Of course, it is possible to define different types of internal `keeper`s for the same module (e.g. a read-only `keeper`). Each type of `keeper` comes with its own constructor function, which is called from the [application's constructor function](../../learn/beginner/00-app-anatomy.md). This is where `keeper`s are instantiated, and where developers make sure to pass correct instances of modules' `keeper`s to other modules that require them.

## Implementing Methods

`Keeper`s primarily expose methods for business logic, as validity checks should have already been performed by the [`Msg` server](./03-msg-services.md) when `keeper`s' methods are called.

<!-- markdown-link-check-disable -->
State management is recommended to be done via [Collections](../packages/collections) 
<!-- The above link is created via the script to generate docs  -->

## State Management

In the Cosmos SDK, it is crucial to be methodical and selective when managing state within a module, as improper state management can lead to inefficiency, security risks, and scalability issues. Not all data belongs in the on-chain state; it's important to store only essential blockchain data that needs to be verified by consensus. Storing unnecessary information, especially client-side data, can bloat the state and slow down performance. Instead, developers should focus on using an off-chain database to handle supplementary data, extending the API as needed. This approach minimizes on-chain complexity, optimizes resource usage, and keeps the blockchain state lean and efficient, ensuring scalability and smooth operations.


The Cosmos SDK leverages Protocol Buffers (protobuf) for efficient state management, providing a well-structured, binary encoding format that ensures compatibility and performance across different modules. The SDK’s recommended approach for managing state is through the [collections package](../pacakges/02-collections.md), which simplifies state handling by offering predefined data structures like maps and indexed sets, reducing the complexity of managing raw state data. While users can opt for custom encoding schemes if they need more flexibility or have specialized requirements, they should be aware that such custom implementations may not integrate seamlessly with indexers that decode state data on the fly. This could lead to challenges in data retrieval, querying, and interoperability, making protobuf a safer and more future-proof choice for most use cases.
