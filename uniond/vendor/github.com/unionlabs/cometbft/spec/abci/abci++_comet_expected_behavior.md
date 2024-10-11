---
order: 4
title: CometBFT's expected behavior
---

# CometBFT's expected behavior

## Valid method call sequences

This section describes what the Application can expect from CometBFT.

The Tendermint consensus algorithm, currently adopted in CometBFT, is designed to protect safety under any network conditions, as long as
less than 1/3 of validators' voting power is byzantine. Most of the time, though, the network will behave
synchronously, no process will fall behind, and there will be no byzantine process. The following describes
what will happen during a block height _h_ in these frequent, benign conditions:

* Consensus will decide in round 0, for height _h_;
* `PrepareProposal` will be called exactly once at the proposer process of round 0, height _h_;
* `ProcessProposal` will be called exactly once at all processes, and
  will return _accept_ in its `ProcessProposalResponse`;
* `ExtendVote` will be called exactly once at all processes;
* `VerifyVoteExtension` will be called exactly _n-1_ times at each validator process, where _n_ is
  the number of validators, and will always return _accept_ in its `VerifyVoteExtensionResponse`;
* `FinalizeBlock` will be called exactly once at all processes, conveying the same prepared
  block that all calls to `PrepareProposal` and `ProcessProposal` had previously reported for
  height _h_; and
* `Commit` will finally be called exactly once at all processes at the end of height _h_.

However, the Application logic must be ready to cope with any possible run of the consensus algorithm for a given
height, including bad periods (byzantine proposers, network being asynchronous).
In these cases, the sequence of calls to ABCI methods may not be so straightforward, but
the Application should still be able to handle them, e.g., without crashing.
The purpose of this section is to define what these sequences look like in a precise way.

As mentioned in the [Basic Concepts](./abci%2B%2B_basic_concepts.md) section, CometBFT
acts as a client of ABCI and the Application acts as a server. Thus, it is up to CometBFT to
determine when and in which order the different ABCI methods will be called. A well-written
Application design should consider _any_ of these possible sequences.

The following grammar, written in case-sensitive Augmented Backus–Naur form (ABNF, specified
in [IETF rfc7405](https://datatracker.ietf.org/doc/html/rfc7405)), specifies all possible
sequences of calls to ABCI, taken by a **correct process**, across all heights from the genesis block,
including recovery runs, from the point of view of the Application.

```abnf
start               = clean-start / recovery

clean-start         = ( app-handshake / state-sync ) consensus-exec
app-handshake       = info init-chain
state-sync          = *state-sync-attempt success-sync info
state-sync-attempt  = offer-snapshot *apply-chunk
success-sync        = offer-snapshot 1*apply-chunk

recovery            = info [init-chain] consensus-exec

consensus-exec      = (inf)consensus-height
consensus-height    = *consensus-round finalize-block commit
consensus-round     = proposer / non-proposer

proposer            = *got-vote [prepare-proposal [process-proposal]] [extend]
extend              = *got-vote extend-vote *got-vote
non-proposer        = *got-vote [process-proposal] [extend]

init-chain          = %s"<InitChain>"
offer-snapshot      = %s"<OfferSnapshot>"
apply-chunk         = %s"<ApplySnapshotChunk>"
info                = %s"<Info>"
prepare-proposal    = %s"<PrepareProposal>"
process-proposal    = %s"<ProcessProposal>"
extend-vote         = %s"<ExtendVote>"
got-vote            = %s"<VerifyVoteExtension>"
finalize-block      = %s"<FinalizeBlock>"
commit              = %s"<Commit>"
```

We have kept some ABCI methods out of the grammar, in order to keep it as clear and concise as possible.
A common reason for keeping all these methods out is that they all can be called at any point in a sequence defined
by the grammar above. Other reasons depend on the method in question:

* `Echo` and `Flush` are only used for debugging purposes. Further, their handling by the Application should be trivial.
* `CheckTx` is detached from the main method call sequence that drives block execution.
* `Query` provides read-only access to the current Application state, so handling it should also be independent from
  block execution.
* Similarly, `ListSnapshots` and `LoadSnapshotChunk` provide read-only access to the Application's previously created
  snapshots (if any), and help populate the parameters of `OfferSnapshot` and `ApplySnapshotChunk` at a process performing
  state-sync while bootstrapping. Unlike `ListSnapshots` and `LoadSnapshotChunk`, both `OfferSnapshot`
  and `ApplySnapshotChunk` _are_ included in the grammar.

Finally, method `Info` is a special case. The method's purpose is three-fold, it can be used

1. as part of handling an RPC call from an external client,
2. as a handshake between CometBFT and the Application to check whether any blocks need
   to be replayed, and
3. at the end of _state-sync_ to verify that the correct state has been reached.

We have left `Info`'s first purpose out of the grammar for the same reasons as all the others: it can happen
at any time, and has nothing to do with the block execution sequence. The second and third purposes, on the other
hand, are present in the grammar.

Let us now examine the grammar line by line, providing further details.

* When a process starts, it may do so for the first time or after a crash (it is recovering).

>```abnf
>start               = clean-start / recovery
>```

* If the process is starting from scratch, depending on whether the _state-sync_ is enabled, it engages in the handshake
with the Application, or it starts the _state-sync_ mechanism to catch up with other processes. Finally, it enters
normal consensus execution.

>```abnf
>clean-start         = ( app-handshake / state-sync ) consensus-exec
>```

* If _state-sync_ is disabled, CometBFT calls `Info` method and then
since the process is starting from scratch and the Application has no state CometBFT calls `InitChain`.

>```abnf
>app-handshake         = info init_chain
>```

* In _state-sync_ mode, CometBFT makes one or more attempts at synchronizing the Application's state.
  At the beginning of each attempt, it offers the Application a snapshot found at another process.
  If the Application accepts the snapshot, a sequence of calls to `ApplySnapshotChunk` method follow
  to provide the Application with all the snapshots needed, in order to reconstruct the state locally.
  A successful attempt must provide at least one chunk via `ApplySnapshotChunk`.
  At the end of a successful attempt, CometBFT calls `Info` to make sure the reconstructed state's
  _AppHash_ matches the one in the block header at the corresponding height. Note that the state
  of  the application does not contain vote extensions itself. The application can rely on
  [CometBFT to ensure](../../docs/references/rfc/rfc-100-abci-vote-extension-propag.md#base-implementation-persist-and-propagate-extended-commit-history)
  the node has all the relevant data to proceed with the execution beyond this point.

>```abnf
>state-sync          = *state-sync-attempt success-sync info
>state-sync-attempt  = offer-snapshot *apply-chunk
>success-sync        = offer-snapshot 1*apply-chunk
>```

* In recovery mode, CometBFT first calls `Info` to know from which height it needs to replay decisions to the Application. If the Application
did not store any state CometBFT calls `InitChain`. After this, CometBFT enters consensus execution, first in replay mode, if there are blocks to replay, and then in normal mode.

>```abnf
>recovery            = info [init-chain] consensus-exec
>```

* The non-terminal `consensus-exec` is a key point in this grammar. It is an infinite sequence of
  consensus heights. The grammar is thus an
  [omega-grammar](https://dl.acm.org/doi/10.5555/2361476.2361481), since it produces infinite
  sequences of terminals (i.e., the API calls).

>```abnf
>consensus-exec      = (inf)consensus-height
>```

* A consensus height consists of zero or more rounds before deciding and executing via a call to
  `FinalizeBlock`, followed by a call to `Commit`. In each round, the sequence of method calls
  depends on whether the local process is the proposer or not. Note that, if a height contains zero
  rounds, this means the process is replaying an already decided value (catch-up mode).
  When calling `FinalizeBlock` with a block, the consensus algorithm run by CometBFT guarantees
  that at least one non-byzantine validator has run `ProcessProposal` on that block.


>```abnf
>consensus-height    = *consensus-round finalize-block commit
>consensus-round     = proposer / non-proposer
>```

* For every round, if the local process is the proposer of the current round, CometBFT calls `PrepareProposal`.
  A successful execution of `PrepareProposal` results in a proposal block being (i) signed and (ii) stored
  (e.g., in stable storage).

  A crash during this step will direct how the node proceeds the next time it is executed, for the same round, after restarted.
  If it crashed before (i), then, during the recovery, `PrepareProposal` will execute as if for the first time.
  Following a crash between (i) and (ii) and in (the likely) case `PrepareProposal` produces a different block,
  the signing of this block will fail, which means that the new block will not be stored or broadcast.
  If the crash happened after (ii), then signing fails but nothing happens to the stored block.

  If a block was stored, it is sent to all validators, including the proposer.
  Receiving a proposal block triggers `ProcessProposal` with such a block.

  Then, optionally, the Application is
  asked to extend its vote for that round. Calls to `VerifyVoteExtension` can come at any time: the
  local process may be slightly late in the current round, or votes may come from a future round
  of this height.

>```abnf
>proposer            = *got-vote [prepare-proposal [process-proposal]] [extend]
>extend              = *got-vote extend-vote *got-vote
>```

* Also for every round, if the local process is _not_ the proposer of the current round, CometBFT
  will call `ProcessProposal` at most once.
  Under certain conditions, CometBFT may not call `ProcessProposal` in a round;
  see [this section](./abci++_example_scenarios.md#scenario-3) for an example.
  At most one call to `ExtendVote` may occur only after
  `ProcessProposal` is called. A number of calls to `VerifyVoteExtension` can occur in any order
  with respect to `ProcessProposal` and `ExtendVote` throughout the round. The reasons are the same
  as above, namely, the process running slightly late in the current round, or votes from future
  rounds of this height received.

>```abnf
>non-proposer        = *got-vote [process-proposal] [extend]
>```

* Finally, the grammar describes all its terminal symbols, which denote the different ABCI method calls that
  may appear in a sequence.

>```abnf
>init-chain          = %s"<InitChain>"
>offer-snapshot      = %s"<OfferSnapshot>"
>apply-chunk         = %s"<ApplySnapshotChunk>"
>info                = %s"<Info>"
>prepare-proposal    = %s"<PrepareProposal>"
>process-proposal    = %s"<ProcessProposal>"
>extend-vote         = %s"<ExtendVote>"
>got-vote            = %s"<VerifyVoteExtension>"
>finalize-block      = %s"<FinalizeBlock>"
>commit              = %s"<Commit>"
>```

## Adapting existing Applications that use legacy ABCI

In some cases, an existing Application using the legacy ABCI may need to be adapted to work with new version of ABCI
with as minimal changes as possible. In this case, of course, new ABCI versions will not provide any advantage with respect
to the legacy ABCI implementation, but will keep the same guarantees.
Here is how ABCI methods should be implemented.

First of all, all the methods that did not change from ABCI 0.17.0 to ABCI 2.0, namely `Echo`, `Flush`, `Info`, `InitChain`,
`Query`, `CheckTx`, `ListSnapshots`, `LoadSnapshotChunk`, `OfferSnapshot`, and `ApplySnapshotChunk`, do not need
to undergo any changes in their implementation.

As for the new methods:

Introduced in ABCI 1.0:

* `PrepareProposal` must create a list of [transactions](./abci++_methods.md#prepareproposal)
  by copying over the transaction list passed in `PrepareProposalRequest.txs`, in the same order.
  The Application must check whether the size of all transactions exceeds the byte limit
  (`PrepareProposalRequest.max_tx_bytes`). If so, the Application must remove transactions at the
  end of the list until the total byte size is at or below the limit.
* `ProcessProposal` must set `ProcessProposalResponse.status` to _accept_ and return.

Introduced in ABCI 2.0:

* `ExtendVote` is to set `ExtendVoteResponse.extension` to an empty byte array and return.
* `VerifyVoteExtension` must set `VerifyVoteExtensionResponse.accept` to _true_ if the extension is
  an empty byte array and _false_ otherwise, then return.
* `FinalizeBlock` is to coalesce the implementation of methods `BeginBlock`, `DeliverTx`, and
  `EndBlock`. Legacy applications looking to reuse old code that implemented `DeliverTx` should
  wrap the legacy `DeliverTx` logic in a loop that executes one transaction iteration per
  transaction in `FinalizeBlockRequest.tx`.

Finally, `Commit`, which is kept in ABCI 2.0, no longer returns the `AppHash`. It is now up to
`FinalizeBlock` to do so. Thus, a slight refactoring of the old `Commit` implementation will be
needed to move the return of `AppHash` to `FinalizeBlock`.

## Accommodating for vote extensions

In a manner transparent to the application, CometBFT ensures the node is provided with all
the data it needs to participate in consensus.

In the case of recovering from a crash, or joining the network via state sync, CometBFT will make
sure the node acquires the necessary vote extensions before switching to consensus.

If a node is already in consensus but falls behind, during catch-up, CometBFT will provide the node with
vote extensions from past heights by retrieving the extensions within `ExtendedCommit` for old heights that it had previously stored.

We realize this is sub-optimal due to the increase in storage needed to store the extensions, we are
working on an optimization of this implementation which should alleviate this concern.
However, the application can use the existing `retain_height` parameter to decide how much
history it wants to keep, just as is done with the block history. The network-wide implications
of the usage of `retain_height` stay the same.
The decision to store
historical commits and potential optimizations, are discussed in detail in [RFC-100](../../docs/references/rfc/rfc-100-abci-vote-extension-propag.md#current-limitations-and-possible-implementations)

## Handling upgrades to ABCI 2.0

If applications upgrade to ABCI 2.0, CometBFT internally ensures that the [application setup](./abci%2B%2B_app_requirements.md#application-configuration-required-to-switch-to-abci-20) is reflected in its operation.
CometBFT retrieves from the application configuration the value of `VoteExtensionsEnableHeight`( _h<sub>e</sub>_,),
the height at which vote extensions are required for consensus to proceed, and uses it to determine the data it stores and data it sends to a peer that is catching up.

Namely, upon saving the block for a given height _h_ in the block store at decision time

* if _h ≥ h<sub>e</sub>_, the corresponding extended commit that was used to decide locally is saved as well
* if _h < h<sub>e</sub>_, there are no changes to the data saved

In the catch-up mechanism, when a node _f_ realizes that another peer is at height _h<sub>p</sub>_, which is more than 2 heights behind height _h<sub>f</sub>_,

* if _h<sub>p</sub> ≥ h<sub>e</sub>_, _f_ uses the extended commit to
      reconstruct the precommit votes with their corresponding extensions
* if _h<sub>p</sub> < h<sub>e</sub>_, _f_ uses the canonical commit to reconstruct the precommit votes,
      as done for ABCI 1.0 and earlier.
