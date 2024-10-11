# ADR 111: `nop` Mempool

## Changelog

- 2023-11-07: First version (@sergio-mena)
- 2023-11-15: Addressed PR comments (@sergio-mena)
- 2023-11-17: Renamed `nil` to `nop` (@melekes)
- 2023-11-20: Mentioned that the app could reuse p2p network in the future (@melekes)
- 2023-11-22: Adapt ADR to implementation (@melekes)

## Status

Accepted

[Tracking issue](https://github.com/cometbft/cometbft/issues/1666)

## Context

### Summary

The current mempool built into CometBFT implements a robust yet somewhat inefficient transaction gossip mechanism.
While the CometBFT team is currently working on more efficient general-purpose transaction gossiping mechanisms,
some users have expressed their desire to manage both the mempool and the transaction dissemination mechanism
outside CometBFT (typically at the application level).

This ADR proposes a fairly simple way for CometBFT to fulfill this use case without moving away from our current architecture.

### In the Beginning...

It is well understood that a dissemination mechanism
(sometimes using _Reliable Broadcast_ [\[HT94\]][HT94] but not necessarily),
is needed in a distributed system implementing State-Machine Replication (SMR).
This is also the case in blockchains.
Early designs such as Bitcoin or Ethereum include an _internal_ component,
responsible for dissemination, called mempool.
Tendermint Core chose to follow the same design given the success
of those early blockchains and, since inception, Tendermint Core and later CometBFT have featured a mempool as an internal piece of its architecture.


However, the design of ABCI clearly dividing the application logic (i.e., the appchain)
and the consensus logic that provides SMR semantics to the app is a unique innovation in Cosmos
that sets it apart from Bitcoin, Ethereum, and many others.
This clear separation of concerns entailed many consequences, mostly positive:
it allows CometBFT to be used underneath (currently) tens of different appchains in production
in the Cosmos ecosystem and elsewhere.
But there are other implications for having an internal mempool
in CometBFT: the interaction between the mempool, the application, and the network
becomes more indirect, and thus more complex and hard to understand and operate.

### ABCI++ Improvements and Remaining Shortcomings

Before the release of ABCI++, `CheckTx` was the main mechanism the app had at its disposal to influence
what transactions made it to the mempool, and very indirectly what transactions got ultimately proposed in a block.
Since ABCI 1.0 (the first part of ABCI++, shipped in `v0.37.x`), the application has
a more direct say in what is proposed through `PrepareProposal` and `ProcessProposal`.

This has greatly improved the ability for appchains to influence the contents of the proposed block.
Further, ABCI++ has enabled many new use cases for appchains. However some issues remain with
the current model:

* We are using the same P2P network for disseminating transactions and consensus-related messages.
* Many mempool parameters are configured on a per-node basis by node operators,
  allowing the possibility of inconsistent mempool configuration across the network
  with potentially serious scalability effects
  (even causing unacceptable performance degradation in some extreme cases).
* The current mempool implementation uses a basic (robust but sub-optimal) flood algorithm
  * the CometBFT team is working on improving it as one of our current priorities,
    but any improvement we come up with must address the needs of a vast spectrum of applications,
    as well as be heavily scaled-tested in various scenarios
    (in an attempt to cover the applications' wide spectrum)
  * a mempool designed specifically for one particular application
    would reduce the search space as its designers can devise it with just their application's
    needs in mind.
* The interaction with the application is still somewhat convoluted:
  * the application has to decide what logic to implement in `CheckTx`,
    what to do with the transaction list coming in `RequestPrepareProposal`,
    whether it wants to maintain an app-side mempool (more on this below), and whether or not
    to combine the transactions in the app-side mempool with those coming in `RequestPrepareProposal`
  * all those combinations are hard to fully understand, as the semantics and guarantees are
    often not clear
  * when using exclusively an app-mempool (the approach taken in the Cosmos SDK `v0.47.x`)
    for populating proposed blocks, with the aim of simplifying the app developers' life,
    we still have a suboptimal model where we need to continue using CometBFT's mempool
    in order to disseminate the transactions. So, we end up using twice as much memory,
    as in-transit transactions need to be kept in both mempools.

The approach presented in this ADR builds on the app-mempool design released in `v0.47.x`
of the Cosmos SDK,
and briefly discussed in the last bullet point above (see [SDK app-mempool][sdk-app-mempool] for further details of this model).

In the app-mempool design in Cosmos SDK `v0.47.x`
an unconfirmed transaction must be both in CometBFT's mempool for dissemination and
in the app's mempool so the application can decide how to manage the mempool.
There is no doubt that this approach has numerous advantages. However, it also has some implications that need to be considered:

* Having every transaction both in CometBFT and in the application is suboptimal in terms of memory.
  Additionally, the app developer has to be careful
  that the contents of both mempools do not diverge over time
  (hence the crucial role `re-CheckTx` plays post-ABCI++).
* The main reason for a transaction needing to be in CometBFT's mempool is
  because the design in Cosmos SDK `v0.47.x` does not consider an application
  that has its own means of disseminating transactions.
  It reuses the peer to peer network underneath CometBFT reactors.
* There is no point in having transactions in CometBFT's mempool if an application implements an ad-hoc design for disseminating transactions.

This proposal targets this kind of applications:
those that have an ad-hoc mechanism for transaction dissemination that better meets the application requirements.

The ABCI application could reuse the P2P network once this is exposed via ABCI.
But this will take some time as it needs to be implemented, and has a dependency
on bi-directional ABCI, which is also quite substantial. See
[1](https://github.com/cometbft/cometbft/discussions/1112) and
[2](https://github.com/cometbft/cometbft/discussions/494) discussions.

We propose to introduce a `nop` (short for no operation) mempool which will effectively act as a stubbed object
internally:

* it will reject any transaction being locally submitted or gossipped by a peer
* when a _reap_ (as it is currently called) is executed in the mempool, an empty answer will always be returned
* the application running on the proposer validator will add transactions it received
  using the appchains's own mechanism via `PrepareProposal`.

## Alternative Approaches

These are the alternatives known to date:

1. Keep the current model. Useful for basic apps, but clearly suboptimal for applications
   with their own mechanism to disseminate transactions and particular performance requirements.
2. Provide more efficient general-purpose mempool implementations.
   This is an ongoing effort (e.g., [CAT mempool][cat-mempool]), but will take some time, and R&D effort, to come up with
   advanced mechanisms -- likely highly configurable and thus complex -- which then will have to be thoroughly tested.
3. A similar approach to this one ([ADR110][adr-110]) whereby the application-specific
   mechanism directly interacts with CometBFT via a newly defined gRPC interface.
4. Partially adopting this ADR. There are several possibilities:
    * Use the current mempool, disable transaction broadcast in `config.toml`, and accept transactions from users via `BroadcastTX*` RPC methods.
      Positive: avoids transaction gossiping; app can reuse the mempool existing in ComeBFT.
      Negative: requires clients to know the validators' RPC endpoints (potential security issues).
    * Transaction broadcast is disabled in `config.toml`, and have the application always reject transactions in `CheckTx`.
      Positive: effectively disables the mempool; does not require modifications to Comet (may be used in `v0.37.x` and `v0.38.x`).
      Negative: requires apps to disseminate txs themselves; the setup for this is less straightforward than this ADR's proposal.

## Decision

TBD

## Detailed Design

What this ADR proposes can already be achieved with an unmodified CometBFT since
`v0.37.x`, albeit with a complex, poor UX (see the last alternative in section
[Alternative Approaches](#alternative-approaches)). The core of this proposal
is to make some internal changes so it is clear an simple for app developers,
thus improving the UX.

#### `nop` Mempool

We propose a new mempool implementation, called `nop` Mempool, that effectively disables all mempool functionality
within CometBFT.
The `nop` Mempool implements the `Mempool` interface in a very simple manner:

*	`CheckTx(tx types.Tx) (*abcicli.ReqRes, error)`: returns `nil, ErrNotAllowed`
*	`RemoveTxByKey(txKey types.TxKey) error`: returns `ErrNotAllowed`
* `ReapMaxBytesMaxGas(maxBytes, maxGas int64) types.Txs`: returns `nil`
* `ReapMaxTxs(max int) types.Txs`: returns `nil`
*	`Lock()`: does nothing
* `Unlock()`: does nothing
*	`Update(...) error`: returns `nil`
* `FlushAppConn() error`: returns `nil`
*	`Flush()`: does nothing
* `TxsAvailable() <-chan struct{}`: returns `nil`
* `EnableTxsAvailable()`: does nothing
* `SetTxRemovedCallback(cb func(types.TxKey))`: does nothing
* `Size() int` returns 0
* `SizeBytes() int64` returns 0

Upon startup, the `nop` mempool reactor will advertise no channels to the peer-to-peer layer.

### Configuration

We propose the following changes to the `config.toml` file:

```toml
[mempool]
# The type of mempool for this CometBFT node to use.
#
# Valid types of mempools supported by CometBFT:
# - "flood" : clist mempool with flooding gossip protocol (default)
# - "nop"   : nop-mempool (app has implemented an alternative tx dissemination mechanism)
type = "nop"
```

The config validation logic will be modified to add a new rule that rejects a configuration file
if all of these conditions are met:

* the mempool is set to `nop`
* `create_empty_blocks`, in `consensus` section, is set to `false`.

The reason for this extra validity rule is that the `nop`-mempool, as proposed here,
does not support the "do not create empty blocks" functionality.
Here are some considerations on this:

* The "do not create empty blocks" functionality
  * entangles the consensus and mempool reactors
  * is hardly used in production by real appchains (to the best of CometBFT team's knowledge)
  * its current implementation for the built-in mempool has undesired side-effects
    * app hashes currently refer to the previous block,
    * and thus it interferes with query provability.
* If needed in the future, this can be supported by extending ABCI,
  but we will first need to see a real need for this before committing to changing ABCI
  (which has other, higher-impact changes waiting to be prioritized).

### RPC Calls

There are no changes needed in the code dealing with RPC. Those RPC paths that call methods of the `Mempool` interface,
will simply be calling the new implementation.

### Impacted Workflows

* *Submitting a transaction*. Users are not to submit transactions via CometBFT's RPC.
  `BroadcastTx*` RPC methods will fail with a reasonable error and the 501 status code.
  The application running on a full node must offer an interface for users to submit new transactions.
  It could also be a distinct node (or set of nodes) in the network.
  These considerations are exclusively the application's concern in this approach.
* *Time to propose a block*. The consensus reactor will call `ReapMaxBytesMaxGas` which will return a `nil` slice.
  `RequestPrepareProposal` will thus contain no transactions.
* *Consensus waiting for transactions to become available*. `TxsAvailable()` returns `nil`. 
  `cs.handleTxsAvailable()` won't ever be executed.
  At any rate, a configuration with the `nop` mempool and `create_empty_blocks` set to `false`
  will be rejected in the first place.
* *A new block is decided*.
  * When `Update` is called, nothing is done (no decided transaction is removed).
  * Locking and unlocking the mempool has no effect.
* *ABCI mempool's connection*
  CometBFT will still open a "mempool" connection, even though it won't be used.
  This is to avoid doing lots of breaking changes.

### Impact on Current Release Plans

The changes needed for this approach, are fairly simple, and the logic is clear.
This might allow us to even deliver it as part of CometBFT `v1` (our next release)
even without a noticeable impact on `v1`'s delivery schedule.

The CometBFT team (learning from past dramatic events) usually takes a conservative approach
for backporting changes to release branches that have already undergone a full QA cycle
(and thus are in code-freeze mode).
For this reason, although the limited impact of these changes would limit the risks
of backporting to `v0.38.x` and `v0.37.x`, a careful risk/benefit evaluation will
have to be carried out.

Backporting to `v0.34.x` does not make sense as this version predates the release of `ABCI 1.0`,
so using the `nop` mempool renders CometBFT's operation useless.

### Config parameter _vs._ application-enforced parameter

In the current proposal, the parameter selecting the mempool is in `config.toml`.
However, it is not a clear-cut decision. These are the alternatives we see:

* *Mempool selected in `config.toml` (our current design)*.
  This is the way the mempool has always been selected in Tendermint Core and CometBFT,
  in those versions where there were more than one mempool to choose from.
  As the configuration is in `config.toml`, it is up to the node operators to configure their
  nodes consistently, via social consensus. However this cannot be guaranteed.
  A network with an inconsistent choice of mempool at different nodes might
  result in undesirable side effects, such as peers disconnecting from nodes
  that sent them messages via the mempool channel.
* *Mempool selected as a network-wide parameter*.
  A way to prevent any inconsistency when selecting the mempool is to move the configuration out of `config.toml`
  and have it as a network-wide application-enforced parameter, implemented in the same way as Consensus Params.
  The Cosmos community may not be ready for such a rigid, radical change,
  even if it eliminates the risk of operators shooting themselves in the foot.
  Hence we went currently favor the previous alternative.
* *Mempool selected as a network-wide parameter, but allowing override*.
  A third option, half way between the previous two, is to have the mempool selection
  as a network-wide parameter, but with a special value called _local-config_ that still
  allows an appchain to decide to leave it up to operators to configure it in `config.toml`.

Ultimately, the "config parameter _vs._ application-enforced parameter" discussion
is a more general one that is applicable to other parameters not related to mempool selection.
In that sense, it is out of the scope of this ADR.

## Consequences

### Positive

- Applications can now find mempool mechanisms that fit better their particular needs:
  - Ad-hoc ways to add, remove, merge, reorder, modify, prioritize transactions according
    to application needs.
  - A way to disseminate transactions (gossip-based or other) to get the submitted transactions
    to proposers. The application developers can devise simpler, efficient mechanisms tailored
    to their application.
  - Back-pressure mechanisms to prevent malicious users from abusing the transaction
    dissemination mechanism.
- In this approach, CometBFT's peer-to-peer layer is relieved from managing transaction gossip, freeing up its resources for other reactors such as consensus, evidence, block-sync, or state-sync.
- There is no risk for the operators of a network to provide inconsistent configurations
  for some mempool-related parameters. Some of those misconfigurations are known to have caused
  serious performance issues in CometBFT's peer to peer network.
  Unless, of course, the application-defined transaction dissemination mechanism ends up
  allowing similar configuration inconsistencies.
- The interaction between the application and CometBFT at `PrepareProposal` time
  is simplified. No transactions are ever provided by CometBFT,
  and no transactions can ever be left in the mempool when CometBFT calls `PrepareProposal`:
  the application trivially has all the information.
- UX is improved compared to how this can be done prior to this ADR.

### Negative

- With the `nop` mempool, it is up to the application to provide users with a way
  to submit transactions and deliver those transactions to validators.
  This is a considerable endeavor, and more basic appchains may consider it is not worth the hassle.
- There is a risk of wasting resources by those nodes that have a misconfigured
  mempool (bandwidth, CPU, memory, etc). If there are TXs submitted (incorrectly)
  via CometBFT's RPC, but those TXs are never submitted (correctly via an
  app-specific interface) to the App. As those TXs risk being there until the node
  is stopped. Moreover, those TXs will be replied & proposed every single block.
  App developers will need to keep this in mind and panic on `CheckTx` or
  `PrepareProposal` with non-empty list of transactions.
- Optimizing block proposals by only including transaction IDs (e.g. TX hashes) is more difficult.
  The ABCI app could do it by submitting TX hashes (rather than TXs themselves)
  in `PrepareProposal`, and then having a mechanism for pulling TXs from the
  network upon `FinalizeBlock`.
  
[sdk-app-mempool]: https://docs.cosmos.network/v0.47/build/building-apps/app-mempool
[adr-110]: https://github.com/cometbft/cometbft/pull/1565
[HT94]: https://dl.acm.org/doi/book/10.5555/866693
[cat-mempool]: https://github.com/cometbft/cometbft/pull/1472