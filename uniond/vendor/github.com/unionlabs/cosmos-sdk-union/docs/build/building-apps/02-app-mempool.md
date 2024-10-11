---
sidebar_position: 1
---

# Application Mempool

:::note Synopsis
This sections describes how the app-side mempool can be used and replaced. 
:::

Since `v0.47` the application has its own mempool to allow much more granular
block building than previous versions. This change was enabled by
[ABCI 1.0](https://docs.cometbft.com/v0.37/spec/abci/).
Notably it introduces the `PrepareProposal` and `ProcessProposal` steps of ABCI++. For more information please see [here](../abci/00-introduction.md)

:::note Pre-requisite Readings

* [BaseApp](../../learn/advanced/00-baseapp.md)
* [Abci](../abci/00-introduction.md)

:::

## Mempool

* Before we delve into `PrepareProposal` and `ProcessProposal`, let's first walk through the mempool concepts.

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
When this mempool is used, it is assumed that an application will rely on CometBFT's transaction ordering defined in `RequestPrepareProposal`,
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
