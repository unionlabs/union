# ADR 60: ABCI 1.0 Integration (Phase I)

## Changelog

* 2022-08-10: Initial Draft (@alexanderbez, @tac0turtle)
* Nov 12, 2022: Update `PrepareProposal` and `ProcessProposal` semantics per the
  initial implementation [PR](https://github.com/cosmos/cosmos-sdk/pull/13453) (@alexanderbez)

## Status

ACCEPTED

## Abstract

This ADR describes the initial adoption of [ABCI 1.0](https://github.com/tendermint/tendermint/blob/master/spec/abci%2B%2B/README.md),
the next evolution of ABCI, within the Cosmos SDK. ABCI 1.0 aims to provide
application developers with more flexibility and control over application and
consensus semantics, e.g. in-application mempools, in-process oracles, and
order-book style matching engines.

## Context

Tendermint will release ABCI 1.0. Notably, at the time of this writing,
Tendermint is releasing v0.37.0 which will include `PrepareProposal` and `ProcessProposal`.

The `PrepareProposal` ABCI method is concerned with a block proposer requesting
the application to evaluate a series of transactions to be included in the next
block, defined as a slice of `TxRecord` objects. The application can either
accept, reject, or completely ignore some or all of these transactions. This is
an important consideration to make as the application can essentially define and
control its own mempool allowing it to define sophisticated transaction priority
and filtering mechanisms, by completely ignoring the `TxRecords` Tendermint
sends it, favoring its own transactions. This essentially means that the Tendermint
mempool acts more like a gossip data structure.

The second ABCI method, `ProcessProposal`, is used to process the block proposer's
proposal as defined by `PrepareProposal`. It is important to note the following
with respect to `ProcessProposal`:

* Execution of `ProcessProposal` must be deterministic.
* There must be coherence between `PrepareProposal` and `ProcessProposal`. In
  other words, for any two correct processes *p* and *q*, if *q*'s Tendermint
	calls `RequestProcessProposal` on *u<sub>p</sub>*, *q*'s Application returns
	ACCEPT in `ResponseProcessProposal`.

It is important to note that in ABCI 1.0 integration, the application
is NOT responsible for locking semantics -- Tendermint will still be responsible
for that. In the future, however, the application will be responsible for locking,
which allows for parallel execution possibilities.

## Decision

We will integrate ABCI 1.0, which will be introduced in Tendermint
v0.37.0, in the next major release of the Cosmos SDK. We will integrate ABCI 1.0
methods on the `BaseApp` type. We describe the implementations of the two methods
individually below.

Prior to describing the implementation of the two new methods, it is important to
note that the existing ABCI methods, `CheckTx`, `DeliverTx`, etc, still exist and
serve the same functions as they do now.

### `PrepareProposal`

Prior to evaluating the decision for how to implement `PrepareProposal`, it is
important to note that `CheckTx` will still be executed and will be responsible
for evaluating transaction validity as it does now, with one very important
*additive* distinction.

When executing transactions in `CheckTx`, the application will now add valid
transactions, i.e. passing the AnteHandler, to its own mempool data structure.
In order to provide a flexible approach to meet the varying needs of application
developers, we will define both a mempool interface and a data structure utilizing
Golang generics, allowing developers to focus only on transaction
ordering. Developers requiring absolute full control can implement their own
custom mempool implementation.

We define the general mempool interface as follows (subject to change):

```go
type Mempool interface {
	// Insert attempts to insert a Tx into the app-side mempool returning
	// an error upon failure.
	Insert(sdk.Context, sdk.Tx) error

	// Select returns an Iterator over the app-side mempool. If txs are specified,
	// then they shall be incorporated into the Iterator. The Iterator must
	// closed by the caller.
	Select(sdk.Context, [][]byte) Iterator

	// CountTx returns the number of transactions currently in the mempool.
	CountTx() int

	// Remove attempts to remove a transaction from the mempool, returning an error
	// upon failure.
	Remove(sdk.Tx) error
}

// Iterator defines an app-side mempool iterator interface that is as minimal as
// possible. The order of iteration is determined by the app-side mempool
// implementation.
type Iterator interface {
	// Next returns the next transaction from the mempool. If there are no more
	// transactions, it returns nil.
	Next() Iterator

	// Tx returns the transaction at the current position of the iterator.
	Tx() sdk.Tx
}
```

We will define an implementation of `Mempool`, defined by `nonceMempool`, that
will cover most basic application use-cases. Namely, it will prioritize transactions
by transaction sender, allowing for multiple transactions from the same sender.

The default app-side mempool implementation, `nonceMempool`, will operate on a 
single skip list data structure. Specifically, transactions with the lowest nonce
globally are prioritized. Transactions with the same nonce are prioritized by
sender address.

```go
type nonceMempool struct {
	txQueue *huandu.SkipList
}
```

Previous discussions<sup>1</sup> have come to the agreement that Tendermint will
perform a request to the application, via `RequestPrepareProposal`, with a certain
amount of transactions reaped from Tendermint's local mempool. The exact amount
of transactions reaped will be determined by a local operator configuration.
This is referred to as the "one-shot approach" seen in discussions.

When Tendermint reaps transactions from the local mempool and sends them to the
application via `RequestPrepareProposal`, the application will have to evaluate
the transactions. Specifically, it will need to inform Tendermint if it should
reject and or include each transaction. Note, the application can even *replace*
transactions entirely with other transactions.

When evaluating transactions from `RequestPrepareProposal`, the application will
ignore *ALL* transactions sent to it in the request and instead reap up to
`RequestPrepareProposal.max_tx_bytes` from it's own mempool.

Since an application can technically insert or inject transactions on `Insert`
during `CheckTx` execution, it is recommended that applications ensure transaction
validity when reaping transactions during `PrepareProposal`. However, what validity
exactly means is entirely determined by the application.

The Cosmos SDK will provide a default `PrepareProposal` implementation that simply
select up to `MaxBytes` *valid* transactions.

However, applications can override this default implementation with their own
implementation and set that on `BaseApp` via `SetPrepareProposal`.


### `ProcessProposal`

The `ProcessProposal` ABCI method is relatively straightforward. It is responsible
for ensuring validity of the proposed block containing transactions that were
selected from the `PrepareProposal` step. However, how an application determines
validity of a proposed block depends on the application and its varying use cases.
For most applications, simply calling the `AnteHandler` chain would suffice, but
there could easily be other applications that need more control over the validation
process of the proposed block, such as ensuring txs are in a certain order or
that certain transactions are included. While this theoretically could be achieved
with a custom `AnteHandler` implementation, it's not the cleanest UX or the most
efficient solution.

Instead, we will define an additional ABCI interface method on the existing
`Application` interface, similar to the existing ABCI methods such as `BeginBlock`
or `EndBlock`. This new interface method will be defined as follows:

```go
ProcessProposal(sdk.Context, abci.ProcessProposalRequest) error {}
```

Note, we must call `ProcessProposal` with a new internal branched state on the
`Context` argument as we cannot simply just use the existing `checkState` because
`BaseApp` already has a modified `checkState` at this point. So when executing
`ProcessProposal`, we create a similar branched state, `processProposalState`,
off of `deliverState`. Note, the `processProposalState` is never committed and
is completely discarded after `ProcessProposal` finishes execution.

The Cosmos SDK will provide a default implementation of `ProcessProposal` in which
all transactions are validated using the CheckTx flow, i.e. the AnteHandler, and
will always return ACCEPT unless any transaction cannot be decoded.

### `DeliverTx`

Since transactions are not truly removed from the app-side mempool during
`PrepareProposal`, since `ProcessProposal` can fail or take multiple rounds and
we do not want to lose transactions, we need to finally remove the transaction
from the app-side mempool during `DeliverTx` since during this phase, the
transactions are being included in the proposed block.

Alternatively, we can keep the transactions as truly being removed during the
reaping phase in `PrepareProposal` and add them back to the app-side mempool in
case `ProcessProposal` fails.

## Consequences

### Backwards Compatibility

ABCI 1.0 is naturally not backwards compatible with prior versions of the Cosmos SDK
and Tendermint. For example, an application that requests `RequestPrepareProposal`
to the same application that does not speak ABCI 1.0 will naturally fail.

However, in the first phase of the integration, the existing ABCI methods as we
know them today will still exist and function as they currently do.

### Positive

* Applications now have full control over transaction ordering and priority.
* Lays the groundwork for the full integration of ABCI 1.0, which will unlock more
  app-side use cases around block construction and integration with the Tendermint
  consensus engine.

### Negative

* Requires that the "mempool", as a general data structure that collects and stores
  uncommitted transactions will be duplicated between both Tendermint and the
  Cosmos SDK.
* Additional requests between Tendermint and the Cosmos SDK in the context of
  block execution. Albeit, the overhead should be negligible.
* Not backwards compatible with previous versions of Tendermint and the Cosmos SDK.

## Further Discussions

It is possible to design the app-side implementation of the `Mempool[T MempoolTx]`
in many different ways using different data structures and implementations. All
of which have different tradeoffs. The proposed solution keeps things simple
and covers cases that would be required for most basic applications. There are
tradeoffs that can be made to improve performance of reaping and inserting into
the provided mempool implementation.

## References

* https://github.com/tendermint/tendermint/blob/master/spec/abci%2B%2B/README.md
* [1] https://github.com/tendermint/tendermint/issues/7750#issuecomment-1076806155
* [2] https://github.com/tendermint/tendermint/issues/7750#issuecomment-1075717151
