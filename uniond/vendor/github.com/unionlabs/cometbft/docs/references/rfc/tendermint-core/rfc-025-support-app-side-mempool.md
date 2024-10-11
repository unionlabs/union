# RFC 25: Support Application Defined Transaction Storage (app-side mempools)

## Changelog

- Aug 17, 2022: initial draft (@williambanfield)
- Aug 19, 2022: updated draft (@williambanfield)

## Abstract

With the release of ABCI++, specifically the `PrepareProposal` call, the utility
of the Tendermint mempool becomes much less clear. This RFC discusses possible
changes that should be considered to Tendermint to better support applications
that intend to use `PrepareProposal` to implement much more powerful transaction
ordering and filtering functionality than Tendermint can provide. It proposes
scoping down the responsibilities of Tendermint to suit this new use case.

## Background

Tendermint currently ships with a data structure it calls the
[mempool][mempool-link]. The mempool's primary function is to store pending
valid transactions. Tendermint uses the contents of the mempool in two main
ways: 1) to gossip these pending transactions to other nodes on a Tendermint
network and 2) to select transactions to be included in a proposed block. Before
ABCI++, when proposing a block Tendermint selects the next set of transactions
from the mempool that fit within block and proposes them.

There are a few issues with this data structure. These include issues of how
transaction validity is defined, how transactions should be ordered and selected
for inclusion in the next block, and when a transaction should start or stop
being gossiped. The creation of `PrepareProposal` in ABCI++ adds the additional
issue of unclear ownership over which entity, Tendermint or the ABCI
application, is responsible for selecting the transactions to be included in
a proposed block.

None of these issues of validity, ordering, and gossiping having simple,
one-size fits all solutions. Different applications will have different
preferences and needs for each of them. The current Tendermint mempool attempts
to strike a balance but is quite prescriptive about these questions. We can
better support a varied range of applications by simplifying the current mempool
and by reducing and clarifying its scope of responsibilities.

## Discussion

### The mempool is a leaky abstraction and handles too many concerns

The current mempool is a leaky abstraction. Presently, Tendermint's mempool keeps
track of a multitude of details that primarily service concerns of the application.

#### Gas

The mempool keeps track of Gas, a proxy for how computationally expensive it
will be to execute a transaction. As discussed in [RFC-011](https://github.com/tendermint/tendermint/blob/2313f358003d0c4d9d0e7705b4632d819dfb0d92/docs/rfc/rfc-011-delete-gas.md), this metadata is
not a concern of Tendermint's. Tendermint does not execute transactions. This
data is stored within Tendermint's mempool along with the maximum gas the application
will permit to be used in a block so that Tendermint's mempool can enforce
transaction validity using it: transactions that exceed the configured maximum
are rejected from the mempool. How much 'Gas' a transaction consumes and if that
precludes it from execution by the application is a validity condition imposed
by the application, not Tendermint. It is an application abstraction that leaks
into the Tendermint mempool.

#### Sender

The Tendermint mempool stores a `sender` string metadata for each transaction
it receives. The mempool only stores one transaction per sender at any time.
The `sender` metadata is populated by the application during `CheckTx`.
`Sender` uniqueness is enforced separately on each node's mempool. Nothing
prevents multiple transactions with the same `sender` from existing in separate
mempools on the network.

While multiple transactions from the same sender on a network is a shortcoming
of the `sender` abstraction, the issue posed by sender to the mempool is that
`sender` uniqueness is a condition of transaction validity that is otherwise
meaningless to Tendermint. The `sender` field allows the application to
influence which transactions Tendermint will include next in a block. However,
with the advent of `PrepareProposal`, the application can select directly and
this `sender` field is of marginal benefit. Additionally, applications require
much more expressive validity conditions than just `sender` uniqueness.

#### Adding additional semantics to the mempool

The Tendermint mempool is relied upon by every ABCI application. Changing its
code to incorporate new features or update its behavior affects all of
Tendermint's downstream consumers. New applications frequently need ways of
sorting pending transactions and imposing transaction validity conditions. This
data structure cannot change quickly to meet the shifting needs of new
consumers while also maintaining a stable API for the applications that are
already successfully running on top of Tendermint. New strategies for sorting
and validating pending transactions would be best implemented outside of
Tendermint, where creating new semantics does not risk disrupting the existing
users.

### Tendermint's scope of responsibility

#### What should Tendermint be responsible for?

Tendermint's responsibilities should be as narrowly scoped as possible to allow
the code base to be useful for many developers and maintainable by the core
team.

The Tendermint node maintains a P2P network over which pending transactions,
proposed blocks, votes and other messages are sent. Tendermint, using these
messages, comes to consensus on the proposed blocks and delivers their contents
to the application in an orderly fashion.

In this description of Tendermint, its only responsibility, in terms of pending
transactions, is to _gossip_ them over its P2P network. Any additional logic
surrounding validity, ordering etc. requires an understanding of the meaning of
the transaction that Tendermint does not and _should not_ have.

#### What should the application be responsible for?

Transaction contents have semantic meaning to the ABCI application. Pending
transactions are valid and have execution priority in relationship to the
current state of application. While Tendermint is clearly responsible for the
action of gossiping the transaction, it cannot decide when to start or stop
gossiping any given transaction. While only valid transactions should be
gossiped, as stated, it cannot appropriately make decisions about transaction
validity beyond simple heuristics. The application therefore should be
responsible for defining pending transaction validity, determining when to start
or stop gossiping a transaction, and for selecting which transaction should be
contained within a block.

### How can Tendermint best be designed for this responsibility?

With the understanding that Tendermint's responsibility is to gossip the set of
transactions that the application currently considers valid and high priority,
we can update its API and data structures accordingly. With the creation of
`PrepareProposal`, the mempool may be able to drop its responsibility to select
transactions for a block; It can be primarily responsible for gossiping and
nothing else.

#### Goodbye mempool, hello GossipList

The mempool contains many structures to retain, order, and select the set of
transactions to gossip and to propose. These mempool structures could be
completely replaced with a single list that allows Tendermint to fulfill the
previously stated responsibility. This proposed list, the `GossipList`, would
simply contain the set of transactions that Tendermint is responsible for
gossiping at the moment. This `GossipList` would be updated by the application
at a set of defined junctures and Tendermint would never add to it or remove
from it without input from the application. Tendermint would impose _no_
validity constraints on the contents of this list and would not attempt to
remove items unless instructed to.

### Mock API of the GossipList

Outlined below is a proposed API for this data structure. These calls would be
added to the ABCI API and would come to replace the current `CheckTx` call.

#### `OfferPendingTransaction`

`OfferPendingTransaction` replaces the `CheckTx` call that is invoked when
Tendermint receives a submitted or gossiped transaction. The `GossipList` will
invoke `OfferPendingTransaction` on _every_ transaction sent to Tendermint that
does not match one of the transactions already in the `GossipList`. The mempool
currently drops gossiped transactions before `CheckTx` is called if the
transaction is considered invalid for a Tendermint-defined reason such as
inclusion in the mempool 'cache' or it overflows the max transaction size.

The application can indicate if the transaction should be added to the
`GossipList` via `ResponseOfferPendingTransaction`'s `GossipStatus` field. If
the `GossipList` is full, the application must list a transaction to remove from
`GossipList`, otherwise the transaction will not be added. In this way,
a transaction will _never_ leave the list unless the application removes it from
the list explicitly.

```proto
message RequestOfferPendingTransaction {
  bytes  tx = 1;
  int64 gossip_list_max_size = 2;
  int64 gossip_list_current_size = 3;
}

message ResponseOfferPendingTransaction {
  GossipStatus  gossip_status = 1;
  enum GossipStatus {
    UNKNOWN  = 0;
    GOSSIP = 1;
    NO_GOSSIP = 2;
  }
  repeated bytes removals =2
}
```

#### `UpdateTransactionGossipList`

`UpdateTransactionGossipList` would be a simple method that allows the
application to exactly set the contents of the `GossipList`. Tendermint would
call `UpdateTransactionGossipList` on the application, which would respond with
the list of all transactions to gossip. The contents of the `GossipList` would
be completely replaced with the contents provided by the application in
`UpdateTransactionGossipList`.

```proto
message UpdateTransactionGossipListRequest {
  int64 max_size = 1; // application cannot provide more than `max_size` transactions.
}

message UpdateTransactionGossipListResponse {
  repeated bytes = 1;
}
```

This new `ABCI` method would serve multiple functions. First, it would replace
the re-`CheckTx` calls made by Tendermint after each block is committed. After
each block is committed, Tendermint currently passes the entire contents of the
mempool to the application one-by-one via `CheckTx` calls with `CheckTxType` set
to `RECHECK`. The application, in this way, can then inspect the entire mempool
and remove any transactions that became invalid as a result of the most recent
block being committed.

`UpdateTransactionGossipList` would completely replace this set of re-`CheckTx`
calls. After each block is committed, Tendermint would call
`UpdateTransactionGossipList` and the application would be responsible for
exactly providing the set of transactions for Tendermint to maintain. The IPC
overhead here would be roughly equivalent to the re-`CheckTx` overhead, as the
entire contents of the gossip structure is communicated, but, in the
`UpdateTransactionGossipList` call, the application sends transactions instead
of Tendermint.

This new method would _also_ replace the mempool's `Update` API. The `Update`
method on the mempool receives the list of transactions that were just executed
as part of the most recent height and removes them from the mempool. The
`GossipList` would have no such method and instead, the application would become
responsible for setting the contents after each block via
`UpdateTransactionGossipList`. This gives the application more control over when
to start and stop gossiping transactions than it has at the moment. In this
call, the application can completely replace the `GossipList`.

This also complements the `PrepareProposal` call nicely, because a transaction
introduced via `PrepareProposal` may be semantically equivalent to a transaction
present in Tendermint's mempool in a way that Tendermint cannot detect. The
mempool `Update` call only compares transaction hashes,
`UpdateTransactionGossipList` allows the application to easily compare on
transaction contents as well.

As a nice benefit, it also allows the application to easily continue gossiping
of a transaction that was just executed in the block. Applications may wish to
execute the same transaction multiple times, which the mempool `Update` call
makes very cumbersome by clearing transactions that have the same contents of
those that were just executed.

### Tendermint startup

On Tendermint startup, the `GossipList` would be completely empty. It does not
persist transactions and is an in-memory only data structure. To populate the
`GossipList` on startup, Tendermint will issue an `UpdateTransactionGossipList`
call to the application to request the application provide it with a list of
transactions to fill the gossip list.

### Additional benefits of this API

#### No more confusing mempool cache

The current Tendermint mempool stores a [cache][cache-when-clear] of transaction
hashes that should not be accepted into the mempool. When a transaction is sent
to the mempool but is present in the cache the transaction is dropped without
ever being sent to the application via `CheckTx`. This cache is intended to help
the application guard against receiving the same invalid transaction over and
over. However, this means that presence or absence from the mempool cache
becomes a condition of validity for pending transactions.

Being placed in this cache has serious consequences for a proposed transaction,
but the rules for when a transaction should be placed in this cache are unclear.
So unclear in fact, that conditions for when to include a transaction in this
cache have been completely reversed by different commits
([1][update-remove-from-cache],[2][update-keep-in-cache]) on the Tendermint
project. Additional github issues have noted that it's very ambiguous as to
[when the cache should be cleared][cache-when-clear] and whether or not the
cache should allow [previously invalid transactions][later-valid] to later
become valid. There is no one-size-fits all solution to these problems.
Different applications need very different behavior, so this should ultimately
not be the responsibility of Tendermint. Implementing the `GossipList` clears
Tendermint of this responsibility.

#### Improved guarantees about the set of transactions being gossiped

As discussed in the [Mock API](#mock-api-of-the-gossiplist) section, the
`GossipList` only adds and removes or replaces transactions in the `GossipList`
when the application says to. Under this design, the contents of this list are
never ambiguous. The list contains exactly what the application most recently
told Tendermint to gossip, nothing more nothing less.

### Additional considerations

This document leaves a few aspects unconsidered that should be understood before
future designs are made in this area:

1. Impact of duplicating transactions in both the `GossipList` and within the
   application.
2. Transition plan and feasibility of migrating applications to the new API.

## References

[cache-when-clear]:https://github.com/tendermint/tendermint/issues/7723
[update-remove-from-cache]:https://github.com/tendermint/tendermint/pull/233
[update-keep-in-cache]:https://github.com/tendermint/tendermint/issues/2855
[later-valid]:https://github.com/tendermint/tendermint/issues/458
[mempool-link]:https://github.com/tendermint/tendermint/blob/c8302c5fcb7f1ffafdefc5014a26047df1d27c99/mempool/mempool.go#L30
