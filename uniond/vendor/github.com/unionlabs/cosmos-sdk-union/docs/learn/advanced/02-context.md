---
sidebar_position: 1
---

# Context

:::note Synopsis
The `context` is a data structure intended to be passed from function to function that carries information about the current state of the application. It provides access to a branched storage (a safe branch of the entire state) as well as useful objects and information like `gasMeter`, `block height`, `consensus parameters` and more.
:::

:::note Pre-requisite Readings

* [Anatomy of a Cosmos SDK Application](../beginner/00-app-anatomy.md)
* [Lifecycle of a Transaction](../beginner/01-tx-lifecycle.md)

:::

## Context Definition

The Cosmos SDK `Context` is a custom data structure that contains Go's stdlib [`context`](https://pkg.go.dev/context) as its base, and has many additional types within its definition that are specific to the Cosmos SDK. The `Context` is integral to transaction processing in that it allows modules to easily access their respective [store](./04-store.md#base-layer-kvstores) in the [`multistore`](./04-store.md#multistore) and retrieve transactional context such as the block header and gas meter.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.50.0-alpha.0/types/context.go#L41-L67
```

* **Base Context:** The base type is a Go [Context](https://pkg.go.dev/context), which is explained further in the [Go Context Package](#go-context-package) section below.
* **Multistore:** Every application's `BaseApp` contains a [`CommitMultiStore`](./04-store.md#multistore) which is provided when a `Context` is created. Calling the `KVStore()` and `TransientStore()` methods allows modules to fetch their respective [`KVStore`](./04-store.md#base-layer-kvstores) using their unique `StoreKey`.
* **Header:** The [header](https://docs.cometbft.com/v1.0/spec/core/data_structures#header) is a Blockchain type. It carries important information about the state of the blockchain, such as block height and proposer of the current block.
* **Header Hash:** The current block header hash, obtained during `abci.FinalizeBlock`.
* **Chain ID:** The unique identification number of the blockchain a block pertains to.
* **Transaction Bytes:** The `[]byte` representation of a transaction being processed using the context. Every transaction is processed by various parts of the Cosmos SDK and consensus engine (e.g. CometBFT) throughout its [lifecycle](../beginner/01-tx-lifecycle.md), some of which do not have any understanding of transaction types. Thus, transactions are marshaled into the generic `[]byte` type using some kind of [encoding format](https://docs.cosmos.network/main/learn/advanced/encoding) such as [Amino](https://docs.cosmos.network/main/learn/advanced/encoding).
* **Logger:** A `logger` from the CometBFT libraries. Learn more about logs [here](https://docs.cometbft.com/v1.0/references/config/config.toml#log_level). Modules call this method to create their own unique module-specific logger.
* **VoteInfo:** A list of the ABCI type [`VoteInfo`](https://docs.cometbft.com/v1.0/spec/abci/abci++_methods#voteinfo), which includes the name of a validator and a boolean indicating whether they have signed the block.
* **Gas Meters:** Specifically, a [`gasMeter`](../beginner/04-gas-fees.md#main-gas-meter) for the transaction currently being processed using the context and a [`blockGasMeter`](../beginner/04-gas-fees.md#block-gas-meter) for the entire block it belongs to. Users specify how much in fees they wish to pay for the execution of their transaction; these gas meters keep track of how much [gas](../beginner/04-gas-fees.md) has been used in the transaction or block so far. If the gas meter runs out, execution halts.
* **CheckTx Mode:** A boolean value indicating whether a transaction should be processed in `CheckTx` or `DeliverTx` mode.
* **Min Gas Price:** The minimum [gas](../beginner/04-gas-fees.md) price a node is willing to take in order to include a transaction in its block. This price is a local value configured by each node individually, and should therefore **not be used in any functions used in sequences leading to state-transitions**.
* **Consensus Params:** The ABCI type [Consensus Parameters](https://docs.cometbft.com/v1.0/spec/abci/abci++_app_requirements#consensus-parameters), which specify certain limits for the blockchain, such as maximum gas for a block.
* **Event Manager:** The event manager allows any caller with access to a `Context` to emit [`Events`](./08-events.md). Modules may define module specific
  `Events` by defining various `Types` and `Attributes` or use the common definitions found in `types/`. Clients can subscribe or query for these `Events`. These `Events` are collected throughout `FinalizeBlock` and are returned to CometBFT for indexing.
* **Priority:** The transaction priority, only relevant in `CheckTx`.
* **KV `GasConfig`:** Enables applications to set a custom `GasConfig` for the `KVStore`.
* **Transient KV `GasConfig`:** Enables applications to set a custom `GasConfig` for the transient `KVStore`.
* **StreamingManager:** The streamingManager field provides access to the streaming manager, which allows modules to subscribe to state changes emitted by the blockchain. The streaming manager is used by the state listening API, which is described in [ADR 038](https://docs.cosmos.network/main/architecture/adr-038-state-listening).
* **CometInfo:** A lightweight field that contains information about the current block, such as the block height, time, and hash. This information can be used for validating evidence, providing historical data, and enhancing the user experience. For further details see [here](https://github.com/cosmos/cosmos-sdk/blob/main/core/comet/service.go#L14).
* **HeaderInfo:** The `headerInfo` field contains information about the current block header, such as the chain ID, gas limit, and timestamp. For further details see [here](https://github.com/cosmos/cosmos-sdk/blob/main/core/header/service.go#L14).

## Go Context Package

A basic `Context` is defined in the [Golang Context Package](https://pkg.go.dev/context). A `Context`
is an immutable data structure that carries request-scoped data across APIs and processes. Contexts
are also designed to enable concurrency and to be used in goroutines.

Contexts are intended to be **immutable**; they should never be edited. Instead, the convention is
to create a child context from its parent using a `With` function. For example:

```go
childCtx := parentCtx.WithBlockHeader(header)
```

The [Golang Context Package](https://pkg.go.dev/context) documentation instructs developers to
explicitly pass a context `ctx` as the first argument of a process.

## Store branching

The `Context` contains a `MultiStore`, which allows for branching and caching functionality using `CacheMultiStore`
(queries in `CacheMultiStore` are cached to avoid future round trips).
Each `KVStore` is branched in a safe and isolated ephemeral storage. Processes are free to write changes to
the `CacheMultiStore`. If a state-transition sequence is performed without issue, the store branch can
be committed to the underlying store at the end of the sequence or disregard them if something
goes wrong. The pattern of usage for a Context is as follows:

1. A process receives a Context `ctx` from its parent process, which provides information needed to
   perform the process.
2. The `ctx.ms` is a **branched store**, i.e. a branch of the [multistore](./04-store.md#multistore) is made so that the process can make changes to the state as it executes, without changing the original `ctx.ms`. This is useful to protect the underlying multistore in case the changes need to be reverted at some point in the execution.
3. The process may read and write from `ctx` as it is executing. It may call a subprocess and pass
   `ctx` to it as needed.
4. When a subprocess returns, it checks if the result is a success or failure. If a failure, nothing
   needs to be done - the branch `ctx` is simply discarded. If successful, the changes made to
   the `CacheMultiStore` can be committed to the original `ctx.ms` via `Write()`.

For example, here is a snippet from the [`runTx`](./00-baseapp.md#runtx-antehandler-runmsgs-posthandler) function in [`baseapp`](./00-baseapp.md):

```go
runMsgCtx, msCache := app.cacheTxContext(ctx, txBytes)
result = app.runMsgs(runMsgCtx, msgs, mode)
result.GasWanted = gasWanted
if mode != runTxModeDeliver {
  return result
}
if result.IsOK() {
  msCache.Write()
}
```

Here is the process:

1. Prior to calling `runMsgs` on the message(s) in the transaction, it uses `app.cacheTxContext()`
   to branch and cache the context and multistore.
2. `runMsgCtx` - the context with branched store, is used in `runMsgs` to return a result.
3. If the process is running in [`checkTxMode`](./00-baseapp.md#checktx), there is no need to write the
   changes - the result is returned immediately.
4. If the process is running in [`deliverTxMode`](./00-baseapp.md#delivertx) and the result indicates
   a successful run over all the messages, the branched multistore is written back to the original.
