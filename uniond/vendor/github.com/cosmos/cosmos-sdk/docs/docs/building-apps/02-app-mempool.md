---
sidebar_position: 1
---

# Application mempool

:::note Synopsis
This sections describes how the app-side mempool can be used and replaced. 
:::

Since `v0.47` the application has its own mempool to allow much more granular
block building than previous versions. This change was enabled by
[ABCI 1.0](https://github.com/cometbft/cometbft/blob/v0.37.0/spec/abci).
Notably it introduces the `PrepareProposal` and `ProcessProposal` steps of ABCI++.

:::note Pre-requisite Readings

* [BaseApp](../core/00-baseapp.md)

:::

## Prepare Proposal

`PrepareProposal` handles construction of the block, meaning that when a proposer
is preparing to propose a block, it requests the application to evaluate a
`RequestPrepareProposal`, which contains a series of transactions from CometBFT's
mempool. At this point, the application has complete control over the proposal.
It can modify, delete, and inject transactions from it's own app-side mempool into
the proposal or even ignore all the transactions altogether. What the application
does with the transactions provided to it by `RequestPrepareProposal` have no
effect on CometBFT's mempool.

Note, that the application defines the semantics of the `PrepareProposal` and it
MAY be non-deterministic and is only executed by the current block proposer.

Now, reading mempool twice in the previous sentence is confusing, lets break it down.
CometBFT has a mempool that handles gossiping transactions to other nodes
in the network. How these transactions are ordered is determined by CometBFT's
mempool, typically FIFO. However, since the application is able to fully inspect
all transactions, it can provide greater control over transaction ordering.
Allowing the application to handle ordering enables the application to define how
it would like the block constructed. 

The Cosmos SDK defines the `DefaultProposalHandler` type, which provides applications with
`PrepareProposal` and `ProcessProposal` handlers. If you decide to implement your
own `PrepareProposal` handler, you must be sure to ensure that the transactions
selected DO NOT exceed the maximum block gas (if set) and the maximum bytes provided
by `req.MaxBytes`.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/baseapp/baseapp.go#L868-L916
```

This default implementation can be overridden by the application developer in
favor of a custom implementation in [`app.go`](./01-app-go-v2.md):

```go
prepareOpt := func(app *baseapp.BaseApp) {
	abciPropHandler := baseapp.NewDefaultProposalHandler(mempool, app)
	app.SetPrepareProposal(abciPropHandler.PrepareProposalHandler())
}

baseAppOptions = append(baseAppOptions, prepareOpt)
```

## Process Proposal

`ProcessProposal` handles the validation of a proposal from `PrepareProposal`,
which also includes a block header. Meaning, that after a block has been proposed
the other validators have the right to vote on a block. The validator in the
default implementation of `PrepareProposal` runs basic validity checks on each
transaction.

Note, `ProcessProposal` MAY NOT be non-deterministic, i.e. it must be deterministic.
This means if `ProcessProposal` panics or fails and we reject, all honest validator
processes will prevote nil and the CometBFT round will proceed again until a valid
proposal is proposed.

Here is the implementation of the default implementation:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/baseapp/baseapp.go#L927-L942
```

Like `PrepareProposal` this implementation is the default and can be modified by
the application developer in [`app.go`](./01-app-go-v2.md). If you decide to implement
your own `ProcessProposal` handler, you must be sure to ensure that the transactions
provided in the proposal DO NOT exceed the maximum block gas (if set).

```go
processOpt := func(app *baseapp.BaseApp) {
	abciPropHandler := baseapp.NewDefaultProposalHandler(mempool, app)
	app.SetProcessProposal(abciPropHandler.ProcessProposalHandler())
}

baseAppOptions = append(baseAppOptions, processOpt)
```

## Mempool

Now that we have walked through the `PrepareProposal` & `ProcessProposal`, we can move on to walking through the mempool. 

There are countless designs that an application developer can write for a mempool, the SDK opted to provide only simple mempool implementations.
Namely, the SDK provides the following mempools:

* [No-op Mempool](#no-op-mempool)
* [Sender Nonce Mempool](#sender-nonce-mempool)
* [Priority Nonce Mempool](#priority-nonce-mempool)

The default SDK is a [No-op Mempool](#no-op-mempool), but it can be replaced by the application developer in [`app.go`](./01-app-go-v2.md):

```go
nonceMempool := mempool.NewSenderNonceMempool()
mempoolOpt   := baseapp.SetMempool(nonceMempool)
baseAppOptions = append(baseAppOptions, mempoolOpt)
```

### No-op Mempool

A no-op mempool is a mempool where transactions are completely discarded and ignored when BaseApp interacts with the mempool.
When this mempool is used, it assumed that an application will rely on CometBFT's transaction ordering defined in `RequestPrepareProposal`,
which is FIFO-ordered by default.

> Note: If a NoOp mempool is used, PrepareProposal and ProcessProposal both should be aware of this as
> PrepareProposal could include transactions that could fail verification in ProcessProposal.

### Sender Nonce Mempool

The nonce mempool is a mempool that keeps transactions from an sorted by nonce in order to avoid the issues with nonces. 
It works by storing the transaction in a list sorted by the transaction nonce. When the proposer asks for transactions to be included in a block it randomly selects a sender and gets the first transaction in the list. It repeats this until the mempool is empty or the block is full. 

It is configurable with the following parameters:

#### MaxTxs

It is an integer value that sets the mempool in one of three modes, *bounded*, *unbounded*, or *disabled*.

* **negative**: Disabled, mempool does not insert new transaction and return early.
* **zero**: Unbounded mempool has no transaction limit and will never fail with `ErrMempoolTxMaxCapacity`.
* **positive**: Bounded, it fails with `ErrMempoolTxMaxCapacity` when `maxTx` value is the same as `CountTx()`

#### Seed

Set the seed for the random number generator used to select transactions from the mempool.

### Priority Nonce Mempool

The [priority nonce mempool](https://github.com/cosmos/cosmos-sdk/blob/main/types/mempool/priority_nonce_spec.md) is a mempool implementation that stores txs in a partially ordered set by 2 dimensions:

* priority
* sender-nonce (sequence number)

Internally it uses one priority ordered [skip list](https://pkg.go.dev/github.com/huandu/skiplist) and one skip list per sender ordered by sender-nonce (sequence number). When there are multiple txs from the same sender, they are not always comparable by priority to other sender txs and must be partially ordered by both sender-nonce and priority.

It is configurable with the following parameters:

#### MaxTxs

It is an integer value that sets the mempool in one of three modes, *bounded*, *unbounded*, or *disabled*.

* **negative**: Disabled, mempool does not insert new transaction and return early.
* **zero**: Unbounded mempool has no transaction limit and will never fail with `ErrMempoolTxMaxCapacity`.
* **positive**: Bounded, it fails with `ErrMempoolTxMaxCapacity` when `maxTx` value is the same as `CountTx()`

#### Callback

The priority nonce mempool provides mempool options allowing the application sets callback(s).

* **OnRead**: Set a callback to be called when a transaction is read from the mempool.
* **TxReplacement**: Sets a callback to be called when duplicated transaction nonce detected during mempool insert. Application can define a transaction replacement rule based on tx priority or certain transaction fields.

More information on the SDK mempool implementation can be found in the [godocs](https://pkg.go.dev/github.com/cosmos/cosmos-sdk/types/mempool).
