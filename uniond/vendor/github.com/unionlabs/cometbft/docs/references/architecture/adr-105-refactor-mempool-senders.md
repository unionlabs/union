# ADR 105: Refactor list of senders in mempool

## Changelog

- 2023-07-19: Choose callbacks option and mark as accepted (@hvanz)
- 2023-07-10: Add callback alternative (@hvanz)
- 2023-06-26: Initial draft (@hvanz)
- 2024-05-22: Reverted (@hvanz)

## Status

Reverted

## Context

Before adding a transaction to the mempool or deciding to keep it in the mempool after a block execution, we need to send a `CheckTx` message
to the application for validating the transaction. There are [two variants][CheckTxType] of
this message, distinguished by the value in message field `type`:
- `CheckTxType_New` is for transactions that need to be validated before adding
it to the mempool.
- `CheckTxType_Recheck` is for transactions that are in the mempool and need to
be re-validated after committing a block and advancing to the next height.

The mempool communicates with the ABCI server (that is, the application) by
sending `abci.Request`s through the proxy `AppConnMempool`. The proxy provides a
callback mechanism for handling `abci.Response`s. The current mempool
implementation `CListMempool` (also called v0) utilizes this mechanism for
`Recheck` transactions but not for `New` transactions. Instead `New`
transactions require an ad-hoc mechanism for each request. 

The reason behind this difference is that for `New` transactions we need to
record the ID of the peer that sent the transaction. However, this information
is not included in `RequestCheckTx` messages. Recording the sender's ID is
necessary for the transaction propagation protocol, which uses the recorded list
of senders to prevent sending the transaction back to these peers, thus avoiding
sending duplicated messages. More importantly, this mechanism serves as the only means
to stop propagating transactions.

There are two design problems with this implementation. First, there is a
complex pattern for handling callbacks on `New` requests. The following [code
snippet][CheckTxAsync] at the end of the `CheckTx` method, where transactions
received for the first time are processed, demonstrates the issue:
```golang
	reqRes, err := mem.proxyAppConn.CheckTxAsync(context.TODO(), &abci.RequestCheckTx{Tx: tx})
	reqRes.SetCallback(mem.reqResCb(tx, txInfo, cb))
```
When we send a request for validating a transaction via `CheckTxAsync`, it
returns a `ReqRes` object. To handle the response asynchronously, we set an
ad-hoc callback `reqResCb` on `ReqRes`. This callback is different for each
transaction `tx` because it's parameterized by `tx`, `txInfo` (which essentially
contains the sender ID), and another callback function `cb` provided by the
caller of `CheckTx` to be applied on the response.

Secondly, the list of senders for each transaction is recorded directly in the
mempool's `txs` data structure. However, the list of senders is an important
component of the propagation protocol and it should be part of the reactor,
while `txs` is part of the implementation of this specific version (v0) of the
mempool.

In this document, we propose a solution that involves moving the list of senders
from the mempool implementation to the reactor. This change will simplify the
code, establish a more clear separation of the propagation protocol and the data
structure, and allow for future improvements to the mempool as a whole.

## Detailed Design
 
We propose the following changes to the mempool's reactor and the `CListMempool`
implementation.

- In the `Mempool` interface, change the signature of `CheckTx` from
    ``` golang
    CheckTx(tx types.Tx, cb func(*abci.ResponseCheckTx), txInfo TxInfo) error
    ```
    to
    ``` golang
    CheckTx(tx types.Tx) (abcicli.ReqRes, error)
    ```
  - The returning `ReqRes` object can be used to set and invoke a callback to
    handle the response, if needed.
  - The callback parameter `cb` is no longer needed. Currently, this is mainly
    used by the RPC endpoints `broadcast_tx_sync` and `broadcast_tx_commit`, and
    in tests for checking that the response is valid. However, the same
    functionality can be obtained with `ReqRes` response.
  - `txInfo` contains information about the sender, which is also no longer
    needed, as justified by the next point.
- The list of senders for each transaction is currently stored in `mempoolTx`,
  the data structure for the entries of `txs`. Move the senders out of
  `mempoolTx` to a new map `txSenders` of type `map[types.TxKey]map[uint16]bool`
  in the mempool reactor. `txSenders` would map transaction keys to a set of
  peer ids (of type `uint16`). Add also a `cmtsync.RWMutex` lock to handle
  concurrent accesses to the map.
  - This refactoring should not change the fact that the list of senders live as
    long as the transactions are in the mempool. When a transaction is received
    by the reactor (either via RPC or P2P), we call `CheckTx`. We know whether a
    transaction is valid and was included in the mempool by reading the `ReqRes`
    response. If this is the case, record the list of senders in `txSenders`.
    When a transaction is removed from the mempool, notify the reactor to remove
    the list of senders for that transaction, with the channel described below.
- In `CListMempool`, `resCbFirstTime` is the function that handles responses of
  type `CheckTxType_New`. Instead of setting it as an ad-hoc callback on each
  transaction, we could now call it directly from `globalCb`, where responses of
  type `CheckTxType_Recheck` are already being handled.


## Alternatives
### Communicating that a transaction was removed from the mempool

We have identified two approaches for communicating the removal of a transaction
from the mempool to the reactor.

1. With a channel and an infinite loop in a goroutine.
- In `CListMempool`, introduce a new channel `txsRemoved` of type `chan
  types.TxKey` to notify the mempool reactor that a transaction was removed from
  the mempool.
- In the mempool reactor, spawn a goroutine to handle incoming transaction keys
  from the `txsRemoved` channel. For each key received, update `txSenders`.
- Add methods `TxsRemoved() <-chan types.TxKey` and `EnableTxsRemoved()` to the
  `Mempool` interface.
2. With a callback.
- In the mempool reactor's constructor, set a callback function in `CListMempool`.
  The callback takes a transaction key as parameter. When invoked, it will
  update `txSenders`. 
- `CListMempool` stores the callback function as part of its state. When a
  transaction is removed from the mempool, the callback is invoked.
- Add a method `SetTxRemovedCallback(cb func(types.TxKey))` to the `Mempool`
  interface.

The channel and goroutine mechanism is the same used by the mempool to notify
the consensus reactor when there are transactions available to be included in a
new block. The advantage of the callback is that it is immediately called when a
transaction is removed, reducing the chances of data races. 

In any case, adding and removing the same transaction from the mempool is
unlikely to happen in parallel. A transaction is removed from the mempool
either:
1. on updating the mempool, when the transaction is included in a block, or 
2. when handling a `Recheck` CheckTx response, when the transaction was deemed
   invalid by the application.

In both cases, the transaction will still be in the cache. So, if the same
transaction is received again, it will be discarded by the cache, and thus not
added to the mempool and `txSenders`.

### Decision

We have chosen the second option of using a callback because it reduces the
chances of concurrent accesses to the list of senders and it removes the
transaction immediately, keeping the mempool and the list of senders better
synchoronized.

Update: We have decided to revert this change as it may cause in some cases that the transaction is
sent back to the sender. Storing the sender in the reactor after storing the transaction in `CList`
introduces a race condition resulting in the transaction having no sender during a very small window of
time.

## Consequences

The refactoring proposed here does not affect how users and other peers
interact with the mempool. It will only change how transaction metadata is
stored internally.

### Positive

- Get rid of the complex design pattern of callbacks for handling
  `CheckTxType_New` responses.
- Clear separation of propagation protocol in the reactor and the mempool
  implementation.
- Allow for future improvements to both the propagation protocol and the mempool
  implementation.

### Negative

- If chosen, adding a channel and a goroutine for communicating that a
  transaction was removed may increase the concurrency complexity. 
- Storing the sender separated from the transaction results in the transaction having no sender
  during a brief period of time, allowing the transaction to be sent back to the sender.

### Neutral

- We would need to extend the existing tests to cover new scenarios related to
  the new data structures and some potential concurrent issues.

## References

The trigger for this refactoring was [this comment][comment], where we discussed
improvements to the concurrency in the mempool.


[CheckTxType]: https://github.com/cometbft/cometbft/blob/406f8175e352faee381f100ff17fd5c82888646a/proto/tendermint/abci/types.proto#L94-L97
[CheckTxAsync]: https://github.com/cometbft/cometbft/blob/406f8175e352faee381f100ff17fd5c82888646a/mempool/clist_mempool.go#L269-L273
[comment]: https://github.com/cometbft/cometbft/pull/895#issuecomment-1584948704
