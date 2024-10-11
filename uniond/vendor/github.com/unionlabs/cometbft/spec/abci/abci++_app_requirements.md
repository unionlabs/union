---
order: 3
title: Requirements for the Application
---

# Requirements for the Application

- [Requirements for the Application](#requirements-for-the-application)
    - [Formal Requirements](#formal-requirements)
        - [Consensus Connection Requirements](#consensus-connection-requirements)
        - [Mempool Connection Requirements](#mempool-connection-requirements)
    - [Managing the Application state and related topics](#managing-the-application-state-and-related-topics)
        - [Connection State](#connection-state)
            - [Concurrency](#concurrency)
            - [FinalizeBlock](#finalizeblock)
            - [Commit](#commit)
            - [Candidate States](#candidate-states)
        - [States and ABCI Connections](#states-and-abci-connections)
            - [Consensus Connection](#consensus-connection)
            - [Mempool Connection](#mempool-connection)
                - [Replay Protection](#replay-protection)
            - [Info/Query Connection](#infoquery-connection)
            - [Snapshot Connection](#snapshot-connection)
        - [Transaction Results](#transaction-results)
            - [Gas](#gas)
            - [Specifics of `CheckTxResponse`](#specifics-of-checktxresponse)
            - [Specifics of `ExecTxResult`](#specifics-of-exectxresult)
        - [Updating the Validator Set](#updating-the-validator-set)
        - [Consensus Parameters](#consensus-parameters)
            - [List of Parameters](#list-of-parameters)
                - [BlockParams.MaxBytes](#blockparamsmaxbytes)
                - [BlockParams.MaxGas](#blockparamsmaxgas)
                - [EvidenceParams.MaxAgeDuration](#evidenceparamsmaxageduration)
                - [EvidenceParams.MaxAgeNumBlocks](#evidenceparamsmaxagenumblocks)
                - [EvidenceParams.MaxBytes](#evidenceparamsmaxbytes)
                - [ValidatorParams.PubKeyTypes](#validatorparamspubkeytypes)
                - [VersionParams.App](#versionparamsapp)
                - [ABCIParams.VoteExtensionsEnableHeight](#abciparamsvoteextensionsenableheight)
            - [Updating Consensus Parameters](#updating-consensus-parameters)
                - [`InitChain`](#initchain)
                - [`FinalizeBlock`, `PrepareProposal`/`ProcessProposal`](#finalizeblock-prepareproposalprocessproposal)
        - [`Query`](#query)
            - [Query Proofs](#query-proofs)
            - [Peer Filtering](#peer-filtering)
            - [Paths](#paths)
        - [Crash Recovery](#crash-recovery)
        - [State Sync](#state-sync)
            - [Taking Snapshots](#taking-snapshots)
            - [Bootstrapping a Node](#bootstrapping-a-node)
                - [Snapshot Discovery](#snapshot-discovery)
                - [Snapshot Restoration](#snapshot-restoration)
                - [Snapshot Verification](#snapshot-verification)
                - [Transition to Consensus](#transition-to-consensus)
    - [Application configuration required to switch to ABCI 2.0](#application-configuration-required-to-switch-to-abci-20)

## Formal Requirements

### Consensus Connection Requirements

This section specifies what CometBFT expects from the Application. It is structured as a set
of formal requirements that can be used for testing and verification of the Application's logic.

Let *p* and *q* be two correct processes.
Let *r<sub>p</sub>* (resp. *r<sub>q</sub>*) be a round of height *h* where *p* (resp. *q*) is the
proposer.
Let *s<sub>p,h-1</sub>* be *p*'s Application's state committed for height *h-1*.
Let *v<sub>p</sub>* (resp. *v<sub>q</sub>*) be the block that *p*'s (resp. *q*'s) CometBFT passes
on to the Application
via `PrepareProposalRequest` as proposer of round *r<sub>p</sub>* (resp *r<sub>q</sub>*), height *h*,
also known as the raw proposal.
Let *u<sub>p</sub>* (resp. *u<sub>q</sub>*) the possibly modified block *p*'s (resp. *q*'s) Application
returns via `PrepareProposalResponse` to CometBFT, also known as the prepared proposal.

Process *p*'s prepared proposal can differ in two different rounds where *p* is the proposer.

- Requirement 1 [`PrepareProposal`, timeliness]: If *p*'s Application fully executes prepared blocks in
  `PrepareProposal` and the network is in a synchronous period while processes *p* and *q* are in *r<sub>p</sub>*,
  then the value of *TimeoutPropose* at *q* must be such that *q*'s propose timer does not time out
  (which would result in *q* prevoting `nil` in *r<sub>p</sub>*).

Full execution of blocks at `PrepareProposal` time stands on CometBFT's critical path. Thus,
Requirement 1 ensures the Application or operator will set a value for `TimeoutPropose` such that the time it takes
to fully execute blocks in `PrepareProposal` does not interfere with CometBFT's propose timer.
Note that the violation of Requirement 1 may lead to further rounds, but will not
compromise liveness because even though `TimeoutPropose` is used as the initial
value for proposal timeouts, CometBFT will be dynamically adjust these timeouts
such that they will eventually be enough for completing `PrepareProposal`.

- Requirement 2 [`PrepareProposal`, tx-size]: When *p*'s Application calls `PrepareProposal`, the
  total size in bytes of the transactions returned does not exceed `PrepareProposalRequest.max_tx_bytes`.

Busy blockchains might seek to gain full visibility into transactions in CometBFT's mempool,
rather than having visibility only on *a* subset of those transactions that fit in a block.
The application can do so by setting `ConsensusParams.Block.MaxBytes` to -1.
This instructs CometBFT (a) to enforce the maximum possible value for `MaxBytes` (100 MB) at CometBFT level,
and (b) to provide *all* transactions in the mempool when calling `PrepareProposal`.
Under these settings, the aggregated size of all transactions may exceed `PrepareProposalRequest.max_tx_bytes`.
Hence, Requirement 2 ensures that the size in bytes of the transaction list returned by the application will never
cause the resulting block to go beyond its byte size limit.

- Requirement 3 [`PrepareProposal`, `ProcessProposal`, coherence]: For any two correct processes *p* and *q*,
  if *q*'s CometBFT calls `ProcessProposal` on *u<sub>p</sub>*,
  *q*'s Application returns Accept in `ProcessProposalResponse`.

Requirement 3 makes sure that blocks proposed by correct processes *always* pass the correct receiving process's
`ProcessProposal` check.
On the other hand, if there is a deterministic bug in `PrepareProposal` or `ProcessProposal` (or in both),
strictly speaking, this makes all processes that hit the bug byzantine. This is a problem in practice,
as very often validators are running the Application from the same codebase, so potentially *all* would
likely hit the bug at the same time. This would result in most (or all) processes prevoting `nil`, with the
serious consequences on CometBFT's liveness that this entails. Due to its criticality, Requirement 3 is a
target for extensive testing and automated verification.

- Requirement 4 [`ProcessProposal`, determinism-1]: `ProcessProposal` is a (deterministic) function of the current
  state and the block that is about to be applied. In other words, for any correct process *p*, and any arbitrary block *u*,
  if *p*'s CometBFT calls `ProcessProposal` on *u* at height *h*,
  then *p*'s Application's acceptance or rejection **exclusively** depends on *u* and *s<sub>p,h-1</sub>*.

- Requirement 5 [`ProcessProposal`, determinism-2]: For any two correct processes *p* and *q*, and any arbitrary
  block *u*,
  if *p*'s (resp. *q*'s) CometBFT calls `ProcessProposal` on *u* at height *h*,
  then *p*'s Application accepts *u* if and only if *q*'s Application accepts *u*.
  Note that this requirement follows from Requirement 4 and the Agreement property of consensus.

Requirements 4 and 5 ensure that all correct processes will react in the same way to a proposed block, even
if the proposer is Byzantine. However, `ProcessProposal` may contain a bug that renders the
acceptance or rejection of the block non-deterministic, and therefore prevents processes hitting
the bug from fulfilling Requirements 4 or 5 (effectively making those processes Byzantine).
In such a scenario, CometBFT's liveness cannot be guaranteed.
Again, this is a problem in practice if most validators are running the same software, as they are likely
to hit the bug at the same point. There is currently no clear solution to help with this situation, so
the Application designers/implementers must proceed very carefully with the logic/implementation
of `ProcessProposal`. As a general rule `ProcessProposal` SHOULD always accept the block.

According to the Tendermint consensus algorithm, currently adopted in CometBFT,
a correct process can broadcast at most one precommit
message in round *r*, height *h*.
Since, as stated in the [Methods](./abci++_methods.md#extendvote) section, `ExtendVote`
is only called when the consensus algorithm
is about to broadcast a non-`nil` precommit message, a correct process can only produce one vote extension
in round *r*, height *h*.
Let *e<sup>r</sup><sub>p</sub>* be the vote extension that the Application of a correct process *p* returns via
`ExtendVoteResponse` in round *r*, height *h*.
Let *w<sup>r</sup><sub>p</sub>* be the proposed block that *p*'s CometBFT passes to the Application via `ExtendVoteRequest`
in round *r*, height *h*.

- Requirement 6 [`ExtendVote`, `VerifyVoteExtension`, coherence]: For any two different correct
  processes *p* and *q*, if *q* receives *e<sup>r</sup><sub>p</sub>* from *p* in height *h*, *q*'s
  Application returns Accept in `VerifyVoteExtensionResponse`.

Requirement 6 constrains the creation and handling of vote extensions in a similar way as Requirement 3
constrains the creation and handling of proposed blocks.
Requirement 6 ensures that extensions created by correct processes *always* pass the `VerifyVoteExtension`
checks performed by correct processes receiving those extensions.
However, if there is a (deterministic) bug in `ExtendVote` or `VerifyVoteExtension` (or in both),
we will face the same liveness issues as described for Requirement 5, as Precommit messages with invalid vote
extensions will be discarded.

- Requirement 7 [`VerifyVoteExtension`, determinism-1]: `VerifyVoteExtension` is a (deterministic) function of
  the current state, the vote extension received, and the prepared proposal that the extension refers to.
  In other words, for any correct process *p*, and any arbitrary vote extension *e*, and any arbitrary
  block *w*, if *p*'s (resp. *q*'s) CometBFT calls `VerifyVoteExtension` on *e* and *w* at height *h*,
  then *p*'s Application's acceptance or rejection **exclusively** depends on *e*, *w* and *s<sub>p,h-1</sub>*.

- Requirement 8 [`VerifyVoteExtension`, determinism-2]: For any two correct processes *p* and *q*,
  and any arbitrary vote extension *e*, and any arbitrary block *w*,
  if *p*'s (resp. *q*'s) CometBFT calls `VerifyVoteExtension` on *e* and *w* at height *h*,
  then *p*'s Application accepts *e* if and only if *q*'s Application accepts *e*.
  Note that this requirement follows from Requirement 7 and the Agreement property of consensus.

Requirements 7 and 8 ensure that the validation of vote extensions will be deterministic at all
correct processes.
Requirements 7 and 8 protect against arbitrary vote extension data from Byzantine processes,
in a similar way as Requirements 4 and 5 protect against arbitrary proposed blocks.
Requirements 7 and 8 can be violated by a bug inducing non-determinism in
`VerifyVoteExtension`. In this case liveness can be compromised.
Extra care should be put in the implementation of `ExtendVote` and `VerifyVoteExtension`.
As a general rule, `VerifyVoteExtension` SHOULD always accept the vote extension.

- Requirement 9 [*all*, no-side-effects]: *p*'s calls to `PrepareProposal`,
  `ProcessProposal`, `ExtendVote`, and `VerifyVoteExtension` at height *h* do
  not modify *s<sub>p,h-1</sub>*.

- Requirement 10 [`ExtendVote`, `FinalizeBlock`, non-dependency]: for any correct process *p*,
and any vote extension *e* that *p* received at height *h*, the computation of
*s<sub>p,h</sub>* does not depend on *e*.

The call to correct process *p*'s `FinalizeBlock` at height *h*, with block *v<sub>p,h</sub>*
passed as parameter, creates state *s<sub>p,h</sub>*.
Additionally, *p*'s `FinalizeBlock` creates a set of transaction results *T<sub>p,h</sub>*.

- Requirement 11 [`FinalizeBlock`, determinism-1]: For any correct process *p*,
  *s<sub>p,h</sub>* exclusively depends on *s<sub>p,h-1</sub>* and *v<sub>p,h</sub>*.

- Requirement 12 [`FinalizeBlock`, determinism-2]: For any correct process *p*,
  the contents of *T<sub>p,h</sub>* exclusively depend on *s<sub>p,h-1</sub>* and *v<sub>p,h</sub>*.

Note that Requirements 11 and 12, combined with the Agreement property of consensus ensure
state machine replication, i.e., the Application state evolves consistently at all correct processes.

Also, notice that neither `PrepareProposal` nor `ExtendVote` have determinism-related
requirements associated.
Indeed, `PrepareProposal` is not required to be deterministic:

- *u<sub>p</sub>* may depend on *v<sub>p</sub>* and *s<sub>p,h-1</sub>*, but may also depend on other values or operations.
- *v<sub>p</sub> = v<sub>q</sub> &#8655; u<sub>p</sub> = u<sub>q</sub>*.

Likewise, `ExtendVote` can also be non-deterministic:

- *e<sup>r</sup><sub>p</sub>* may depend on *w<sup>r</sup><sub>p</sub>* and *s<sub>p,h-1</sub>*,
  but may also depend on other values or operations.
- *w<sup>r</sup><sub>p</sub> = w<sup>r</sup><sub>q</sub> &#8655;
  e<sup>r</sup><sub>p</sub> = e<sup>r</sup><sub>q</sub>*

### Mempool Connection Requirements

Let *CheckTxCodes<sub>tx,p,h</sub>* denote the set of result codes returned by *p*'s Application,
via `CheckTxResponse`,
to successive calls to `CheckTx` occurring while the Application is at height *h*
and having transaction *tx* as parameter.
*CheckTxCodes<sub>tx,p,h</sub>* is a set since *p*'s Application may
return different result codes during height *h*.
If *CheckTxCodes<sub>tx,p,h</sub>* is a singleton set, i.e. the Application always returned
the same result code in `CheckTxResponse` while at height *h*,
we define *CheckTxCode<sub>tx,p,h</sub>* as the singleton value of *CheckTxCodes<sub>tx,p,h</sub>*.
If *CheckTxCodes<sub>tx,p,h</sub>* is not a singleton set, *CheckTxCode<sub>tx,p,h</sub>* is undefined.
Let predicate *OK(CheckTxCode<sub>tx,p,h</sub>)* denote whether *CheckTxCode<sub>tx,p,h</sub>* is `SUCCESS`.

- Requirement 13 [`CheckTx`, eventual non-oscillation]: For any transaction *tx*,
  there exists a boolean value *b*,
  and a height *h<sub>stable</sub>* such that,
  for any correct process *p*,
  *CheckTxCode<sub>tx,p,h</sub>* is defined, and
  *OK(CheckTxCode<sub>tx,p,h</sub>) = b*
  for any height *h &#8805; h<sub>stable</sub>*.

Requirement 13 ensures that
a transaction will eventually stop oscillating between `CheckTx` success and failure
if it stays in *p's* mempool for long enough.
This condition on the Application's behavior allows the mempool to ensure that
a transaction will leave the mempool of all full nodes,
either because it is expunged everywhere due to failing `CheckTx` calls,
or because it stays valid long enough to be gossipped, proposed and decided.
Although Requirement 13 defines a global *h<sub>stable</sub>*, application developers
can consider such stabilization height as local to process *p* (*h<sub>p,stable</sub>*),
without loss for generality.
In contrast, the value of *b* MUST be the same across all processes.

## Managing the Application state and related topics

### Connection State

CometBFT maintains four concurrent ABCI connections, namely
[Consensus Connection](#consensus-connection),
[Mempool Connection](#mempool-connection),
[Info/Query Connection](#infoquery-connection), and
[Snapshot Connection](#snapshot-connection).
It is common for an application to maintain a distinct copy of
the state for each connection, which are synchronized upon `Commit` calls.

#### Concurrency

In principle, each of the four ABCI connections operates concurrently with one
another. This means applications need to ensure access to state is
thread safe. Both the
[default in-process ABCI client](https://github.com/cometbft/cometbft/blob/main/abci/client/local_client.go#L13)
and the
[default Go ABCI server](https://github.com/cometbft/cometbft/blob/main/abci/server/socket_server.go#L20)
use a global lock to guard the handling of events across all connections, so they are not
concurrent at all. This means whether your app is compiled in-process with
CometBFT using the `NewLocalClient`, or run out-of-process using the `SocketServer`,
ABCI messages from all connections are received in sequence, one at a
time.

The existence of this global mutex means Go application developers can get thread safety for application state by routing all reads and writes through the ABCI system. Thus it may be unsafe to expose application state directly to an RPC interface, and unless explicit measures are taken, all queries should be routed through the ABCI Query method.

#### FinalizeBlock

When the consensus algorithm decides on a block, CometBFT uses `FinalizeBlock` to send the
decided block's data to the Application, which uses it to transition its state, but MUST NOT persist it;
persisting MUST be done during `Commit`.

The Application must remember the latest height from which it
has run a successful `Commit` so that it can tell CometBFT where to
pick up from when it recovers from a crash. See information on the Handshake
[here](#crash-recovery).

#### Commit

The Application should persist its state during `Commit`, before returning from it.

Before invoking `Commit`, CometBFT locks the mempool and flushes the mempool connection. This ensures that
no new messages
will be received on the mempool connection during this processing step, providing an opportunity to safely
update all four
connection states to the latest committed state at the same time.

CometBFT unlocks the mempool after it has finished updating for the new block,
which occurs asynchronously from `Commit`.
See [Mempool Update](../mempool/mempool.md) for more information on what the `update` task does.

WARNING: if the ABCI app logic processing the `Commit` message sends a
`/broadcast_tx_sync` or `/broadcast_tx` and waits for the response
before proceeding, it will deadlock. Executing `broadcast_tx` calls
involves acquiring the mempool lock that CometBFT holds during the `Commit` call.
Synchronous mempool-related calls must be avoided as part of the sequential logic of the
`Commit` function.

#### Candidate States

CometBFT calls `PrepareProposal` when it is about to send a proposed block to the network.
Likewise, CometBFT calls `ProcessProposal` upon reception of a proposed block from the
network. The proposed block's data
that is disclosed to the Application by these two methods is the following:

- the transaction list
- the `LastCommit` referring to the previous block
- the block header's hash (except in `PrepareProposal`, where it is not known yet)
- list of validators that misbehaved
- the block's timestamp
- `NextValidatorsHash`
- Proposer address

The Application may decide to *immediately* execute the given block (i.e., upon `PrepareProposal`
or `ProcessProposal`). There are two main reasons why the Application may want to do this:

- *Avoiding invalid transactions in blocks*.
  In order to be sure that the block does not contain *any* invalid transaction, there may be
  no way other than fully executing the transactions in the block as though it was the *decided*
  block.
- *Quick `FinalizeBlock` execution*.
  Upon reception of the decided block via `FinalizeBlock`, if that same block was executed
  upon `PrepareProposal` or `ProcessProposal` and the resulting state was kept in memory, the
  Application can simply apply that state (faster) to the main state, rather than reexecuting
  the decided block (slower).

`PrepareProposal`/`ProcessProposal` can be called many times for a given height. Moreover,
it is not possible to accurately predict which of the blocks proposed in a height will be decided,
being delivered to the Application in that height's `FinalizeBlock`.
Therefore, the state resulting from executing a proposed block, denoted a *candidate state*, should
be kept in memory as a possible final state for that height. When `FinalizeBlock` is called, the Application should
check if the decided block corresponds to one of its candidate states; if so, it will apply it as
its *ExecuteTxState* (see [Consensus Connection](#consensus-connection) below),
which will be persisted during the upcoming `Commit` call.

Under adverse conditions (e.g., network instability), the consensus algorithm might take many rounds.
In this case, potentially many proposed blocks will be disclosed to the Application for a given height.
By the nature of Tendermint consensus algorithm, currently adopted in CometBFT, the number of proposed blocks received by the Application
for a particular height cannot be bound, so Application developers must act with care and use mechanisms
to bound memory usage. As a general rule, the Application should be ready to discard candidate states
before `FinalizeBlock`, even if one of them might end up corresponding to the
decided block and thus have to be reexecuted upon `FinalizeBlock`.

### States and ABCI Connections

#### Consensus Connection

The Consensus Connection should maintain an *ExecuteTxState* &mdash; the working state
for block execution. It should be updated by the call to `FinalizeBlock`
during block execution and committed to disk as the "latest
committed state" during `Commit`. Execution of a proposed block (via `PrepareProposal`/`ProcessProposal`)
**must not** update the *ExecuteTxState*, but rather be kept as a separate candidate state until `FinalizeBlock`
confirms which of the candidate states (if any) can be used to update *ExecuteTxState*.

#### Mempool Connection

The mempool Connection maintains *CheckTxState*. CometBFT sequentially processes an incoming
transaction (via RPC from client or P2P from the gossip layer) against *CheckTxState*.
If the processing does not return any error, the transaction is accepted into the mempool
and CometBFT starts gossipping it.
*CheckTxState* should be reset to the latest committed state
at the end of every `Commit`.

During the execution of a consensus instance, the *CheckTxState* may be updated concurrently with the
*ExecuteTxState*, as messages may be sent concurrently on the Consensus and Mempool connections.
At the end of the consensus instance, as described above, CometBFT locks the mempool and flushes
the mempool connection before calling `Commit`. This ensures that all pending `CheckTx` calls are
responded to and no new ones can begin.

After the `Commit` call returns, while still holding the mempool lock, `CheckTx` is run again on all
transactions that remain in the node's local mempool after filtering those included in the block.
Parameter `Type` in `CheckTxRequest`
indicates whether an incoming transaction is new (`CHECK_TX_TYPE_NEW`), or a
recheck (`CHECK_TX_TYPE_RECHECK`).

Finally, after re-checking transactions in the mempool, CometBFT will unlock
the mempool connection. New transactions are once again able to be processed through `CheckTx`.

Note that `CheckTx` is just a weak filter to keep invalid transactions out of the mempool and,
ultimately, ouf of the blockchain.
Since the transaction cannot be guaranteed to be checked against the exact same state as it
will be executed as part of a (potential) decided block, `CheckTx` shouldn't check *everything*
that affects the transaction's validity, in particular those checks whose validity may depend on
transaction ordering. `CheckTx` is weak because a Byzantine node need not care about `CheckTx`;
it can propose a block full of invalid transactions if it wants. The mechanism ABCI, from version 1.0, has
in place for dealing with such behavior is `ProcessProposal`.

##### Replay Protection

It is possible for old transactions to be sent again to the Application. This is typically
undesirable for all transactions, except for a generally small subset of them which are idempotent.

The mempool has a mechanism to prevent duplicated transactions from being processed.
This mechanism is nevertheless best-effort (currently based on the indexer)
and does not provide any guarantee of non duplication.
It is thus up to the Application to implement an application-specific
replay protection mechanism with strong guarantees as part of the logic in `CheckTx`.

#### Info/Query Connection

The Info (or Query) Connection should maintain a `QueryState`. This connection has two
purposes: 1) having the application answer the queries CometBFT receives from users
(see section [Query](#query)),
and 2) synchronizing CometBFT and the Application at start up time (see
[Crash Recovery](#crash-recovery))
or after state sync (see [State Sync](#state-sync)).

`QueryState` is a read-only copy of *ExecuteTxState* as it was after the last
`Commit`, i.e.
after the full block has been processed and the state committed to disk.

#### Snapshot Connection

The Snapshot Connection is used to serve state sync snapshots for other nodes
and/or restore state sync snapshots to a local node being bootstrapped.
Snapshot management is optional: an Application may choose not to implement it.

For more information, see Section [State Sync](#state-sync).

### Transaction Results

The Application is expected to return a list of
[`ExecTxResult`](./abci%2B%2B_methods.md#exectxresult) in
[`FinalizeBlockResponse`](./abci%2B%2B_methods.md#finalizeblock). The list of transaction
results MUST respect the same order as the list of transactions delivered via
[`FinalizeBlockRequest`](./abci%2B%2B_methods.md#finalizeblock).
This section discusses the fields inside this structure, along with the fields in
[`CheckTxResponse`](./abci%2B%2B_methods.md#checktx),
whose semantics are similar.

The `Info` and `Log` fields are
non-deterministic values for debugging/convenience purposes. CometBFT logs them but they
are otherwise ignored.

#### Gas

Ethereum introduced the notion of *gas* as an abstract representation of the
cost of the resources consumed by nodes when processing a transaction. Every operation in the
Ethereum Virtual Machine uses some amount of gas.
Gas has a market-variable price based on which miners can accept or reject to execute a
particular operation.

Users propose a maximum amount of gas for their transaction; if the transaction uses less, they get
the difference credited back. CometBFT adopts a similar abstraction,
though uses it only optionally and weakly, allowing applications to define
their own sense of the cost of execution.

In CometBFT, the [ConsensusParams.Block.MaxGas](#consensus-parameters) limits the amount of
total gas that can be used by all transactions in a block.
The default value is `-1`, which means the block gas limit is not enforced, or that the concept of
gas is meaningless.

Responses contain a `GasWanted` and `GasUsed` field. The former is the maximum
amount of gas the sender of a transaction is willing to use, and the latter is how much it actually
used. Applications should enforce that `GasUsed <= GasWanted` &mdash; i.e. transaction execution
or validation should fail before it can use more resources than it requested.

When `MaxGas > -1`, CometBFT enforces the following rules:

- `GasWanted <= MaxGas` for every transaction in the mempool
- `(sum of GasWanted in a block) <= MaxGas` when proposing a block

If `MaxGas == -1`, no rules about gas are enforced.

In v0.34.x and earlier versions, CometBFT does not enforce anything about Gas in consensus,
only in the mempool.
This means it does not guarantee that committed blocks satisfy these rules.
It is the application's responsibility to return non-zero response codes when gas limits are exceeded
when executing the transactions of a block.
Since the introduction of `PrepareProposal` and `ProcessProposal` in v.0.37.x, it is now possible
for the Application to enforce that all blocks proposed (and voted for) in consensus &mdash; and thus all
blocks decided &mdash; respect the `MaxGas` limits described above.

Since the Application should enforce that `GasUsed <= GasWanted` when executing a transaction, and
it can use `PrepareProposal` and `ProcessProposal` to enforce that `(sum of GasWanted in a block) <= MaxGas`
in all proposed or prevoted blocks,
we have:

- `(sum of GasUsed in a block) <= MaxGas` for every block

The `GasUsed` field is ignored by CometBFT.

#### Specifics of `CheckTxResponse`

If `Code != 0`, it will be rejected from the mempool and hence
not broadcasted to other peers and not included in a proposal block.

`Data` contains the result of the `CheckTx` transaction execution, if any. It does not need to be
deterministic since, given a transaction, nodes' Applications
might have a different *CheckTxState* values when they receive it and check their validity
via `CheckTx`.
CometBFT ignores this value in `CheckTxResponse`.

From v0.34.x on, there is a `Priority` field in `CheckTxResponse` that can be
used to explicitly prioritize transactions in the mempool for inclusion in a block
proposal.

#### Specifics of `ExecTxResult`

`FinalizeBlock` is the workhorse of the blockchain. CometBFT delivers the decided block,
including the list of all its transactions synchronously to the Application.
The block delivered (and thus the transaction order) is the same at all correct nodes as guaranteed
by the Agreement property of consensus.

The `Data` field in `ExecTxResult` contains an array of bytes with the transaction result.
It must be deterministic (i.e., the same value must be returned at all nodes), but it can contain arbitrary
data. Likewise, the value of `Code` must be deterministic.
If `Code != 0`, the transaction will be marked invalid,
though it is still included in the block. Invalid transactions are not indexed, as they are
considered analogous to those that failed `CheckTx`.

Both the `Code` and `Data` are included in a structure that is hashed into the
`LastResultsHash` of the block header in the next height.

`Events` include any events for the execution, which CometBFT will use to index
the transaction by. This allows transactions to be queried according to what
events took place during their execution.

### Updating the Validator Set

The application may set the validator set during
[`InitChain`](./abci%2B%2B_methods.md#initchain), and may update it during
[`FinalizeBlock`](./abci%2B%2B_methods.md#finalizeblock). In both cases, a structure of type
[`ValidatorUpdate`](./abci%2B%2B_methods.md#validatorupdate) is returned.

The `InitChain` method, used to initialize the Application, can return a list of validators.
If the list is empty, CometBFT will use the validators loaded from the genesis
file.
If the list returned by `InitChain` is not empty, CometBFT will use its contents as the validator set.
This way the application can set the initial validator set for the
blockchain.

Applications must ensure that a single set of validator updates does not contain duplicates, i.e.
a given public key can only appear once within a given update. If an update includes
duplicates, the block execution will fail irrecoverably.

Structure `ValidatorUpdate` contains a public key, which is used to identify the validator:
The public key currently supports three types:

- `ed25519`
- `secp256k1`
- `bls12381`

Structure `ValidatorUpdate` also contains an `ìnt64` field denoting the validator's new power.
Applications must ensure that
`ValidatorUpdate` structures abide by the following rules:

- power must be non-negative
- if power is set to 0, the validator must be in the validator set; it will be removed from the set
- if power is greater than 0:
    - if the validator is not in the validator set, it will be added to the
      set with the given power
    - if the validator is in the validator set, its power will be adjusted to the given power
- the total power of the new validator set must not exceed `MaxTotalVotingPower`, where
  `MaxTotalVotingPower = MaxInt64 / 8`

Note the updates returned after processing the block at height `H` will only take effect
at block `H+2` (see Section [Methods](./abci%2B%2B_methods.md)).

### Consensus Parameters

`ConsensusParams` are global parameters that apply to all validators in a blockchain.
They enforce certain limits in the blockchain, like the maximum size
of blocks, amount of gas used in a block, and the maximum acceptable age of
evidence. They can be set in
[`InitChain`](./abci%2B%2B_methods.md#initchain), and updated in
[`FinalizeBlock`](./abci%2B%2B_methods.md#finalizeblock).
These parameters are deterministically set and/or updated by the Application, so
all full nodes have the same value at a given height.

#### List of Parameters

These are the current consensus parameters (as of v1.0.x):

1.  [BlockParams.MaxBytes](#blockparamsmaxbytes)
2.  [BlockParams.MaxGas](#blockparamsmaxgas)
3.  [EvidenceParams.MaxAgeDuration](#evidenceparamsmaxageduration)
4.  [EvidenceParams.MaxAgeNumBlocks](#evidenceparamsmaxagenumblocks)
5.  [EvidenceParams.MaxBytes](#evidenceparamsmaxbytes)
6.  [FeatureParams.PbtsEnableHeight](#featureparamspbtsenableheight)
7.  [FeatureParams.VoteExtensionsEnableHeight](#featureparamsvoteextensionsenableheight)
8.  [ValidatorParams.PubKeyTypes](#validatorparamspubkeytypes)
9.  [VersionParams.App](#versionparamsapp)
10. [SynchronyParams.Precision](#synchronyparamsprecision)
11. [SynchronyParams.MessageDelay](#synchronyparamsmessagedelay)

##### BlockParams.MaxBytes

The maximum size of a complete Protobuf encoded block.
This is enforced by the consensus algorithm.

This implies a maximum transaction size that is this `MaxBytes`, less the expected size of
the header, the validator set, and any included evidence in the block.

The Application should be aware that honest validators *may* produce and
broadcast blocks with up to the configured `MaxBytes` size.
As a result, the consensus
[timeout parameters](../../docs/explanation/core/configuration.md#consensus-timeouts-explained)
adopted by nodes should be configured so as to account for the worst-case
latency for the delivery of a full block with `MaxBytes` size to all validators.

If the Application wants full control over the size of blocks,
it can do so by enforcing a byte limit set up at the Application level.
This Application-internal limit is used by `PrepareProposal` to bound the total size
of transactions it returns, and by `ProcessProposal` to reject any received block
whose total transaction size is bigger than the enforced limit.
In such case, the Application MAY set `MaxBytes` to -1.

If the Application sets value -1, consensus will:

- consider that the actual value to enforce is 100 MB
- will provide *all* transactions in the mempool in calls to `PrepareProposal`

Must have `MaxBytes == -1` OR `0 < MaxBytes <= 100 MB`.

> Bear in mind that the default value for the `BlockParams.MaxBytes` consensus
> parameter accepts as valid blocks with size up to 21 MB.
> If the Application's use case does not need blocks of that size,
> or if the impact (specially on bandwidth consumption and block latency)
> of propagating blocks of that size was not evaluated,
> it is strongly recommended to wind down this default value.

##### BlockParams.MaxGas

The maximum of the sum of `GasWanted` that will be allowed in a proposed block.
This is *not* enforced by the consensus algorithm.
It is left to the Application to enforce (ie. if transactions are included past the
limit, they should return non-zero codes). It is used by CometBFT to limit the
transactions included in a proposed block.

Must have `MaxGas >= -1`.
If `MaxGas == -1`, no limit is enforced.

##### EvidenceParams.MaxAgeDuration

This is the maximum age of evidence in time units.
This is enforced by the consensus algorithm.

If a block includes evidence older than this (AND the evidence was created more
than `MaxAgeNumBlocks` ago), the block will be rejected (validators won't vote
for it).

Must have `MaxAgeDuration > 0`.

##### EvidenceParams.MaxAgeNumBlocks

This is the maximum age of evidence in blocks.
This is enforced by the consensus algorithm.

If a block includes evidence older than this (AND the evidence was created more
than `MaxAgeDuration` ago), the block will be rejected (validators won't vote
for it).

Must have `MaxAgeNumBlocks > 0`.

##### EvidenceParams.MaxBytes

This is the maximum size of total evidence in bytes that can be committed to a
single block. It should fall comfortably under the max block bytes.

Its value must not exceed the size of
a block minus its overhead ( ~ `BlockParams.MaxBytes`).

Must have `MaxBytes > 0`.

##### FeatureParams.PbtsEnableHeight

Height at which Proposer-Based Timestamps (PBTS) will be enabled.

A value of 0 means that PBTS is disabled. A value > 0 denotes the
height at which PBTS will be (or has been) enabled.

From the specified height, and for all subsequent heights, the PBTS
algorithm will be used to produce and validate block timestamps. Prior to
this height, or when this height is set to 0, the legacy BFT Time
algorithm is used to produce and validate timestamps.

PBTS cannot be disabled once it is enabled.

Cannot be set to heights lower or equal to the current blockchain height.

Must have `PbtsEnableHeight > [Current height]`

##### FeatureParams.VoteExtensionsEnableHeight

This parameter is either 0 or a positive height at which vote extensions
become mandatory. If the value is zero (which is the default), vote
extensions are disabled, and are therefore precommit messages received
from other nodes are not expected to contain vote extensions.
If the value is greater than zero, at all heights greater than the
configured height `H` vote extensions must be present (even if empty).
When the configured height `H` is reached, `PrepareProposal` will not
include vote extensions yet, but `ExtendVote` and `VerifyVoteExtension` will
be called. Then, when reaching height `H+1`, `PrepareProposal` will
include the vote extensions from height `H`. For all heights after `H`

- vote extensions cannot be disabled,
- they are mandatory: all precommit messages sent MUST have an extension
  attached. Nevertheless, the application MAY provide 0-length
  extensions.

Must always be set to a future height, 0, or the same height that was previously set.
Once the chain's height reaches the value set, it cannot be changed to a different value.

##### ValidatorParams.PubKeyTypes

The parameter restricts the type of keys validators can use. The parameter uses ABCI pubkey naming, not Amino names.

##### VersionParams.App

This is the version of the ABCI application.

##### SynchronyParams.Precision

This sets a bound on how skewed a proposer's clock may be from any validator
on the network while still producing valid proposals.

This parameter is used by the
[Proposer-Based Timestamps (PBTS)](../consensus/proposer-based-timestamp/README.md)
algorithm.

##### SynchronyParams.MessageDelay

This sets a bound on how long a proposal message may take to reach all
validators on a network and still be considered valid.

This parameter is used by the
[Proposer-Based Timestamps (PBTS)](../consensus/proposer-based-timestamp/README.md)
algorithm.

#### Updating Consensus Parameters

The application may set the `ConsensusParams` during
[`InitChain`](./abci%2B%2B_methods.md#initchain),
and update them during
[`FinalizeBlock`](./abci%2B%2B_methods.md#finalizeblock).
If the `ConsensusParams` is empty, it will be ignored. Each field
that is not empty will be applied in full. For instance, if updating the
`Block.MaxBytes`, applications must also set the other `Block` fields (like
`Block.MaxGas`), even if they are unchanged, as they will otherwise cause the
value to be updated to the default.

##### `InitChain`

`InitChainResponse` includes a `ConsensusParams` parameter.
If `ConsensusParams` is `nil`, CometBFT will use the params loaded in the genesis
file. If `ConsensusParams` is not `nil`, CometBFT will use it.
This way the application can determine the initial consensus parameters for the
blockchain.

##### `FinalizeBlock`, `PrepareProposal`/`ProcessProposal`

`FinalizeBlockResponse` accepts a `ConsensusParams` parameter.
If `ConsensusParams` is `nil`, CometBFT will do nothing.
If `ConsensusParams` is not `nil`, CometBFT will use it.
This way the application can update the consensus parameters over time.

The updates returned in block `H` will take effect right away for block
`H+1`.

### `Query`

`Query` is a generic method with lots of flexibility to enable diverse sets
of queries on application state. CometBFT makes use of `Query` to filter new peers
based on ID and IP, and exposes `Query` to the user over RPC.

Note that calls to `Query` are not replicated across nodes, but rather query the
local node's state - hence they may return stale reads. For reads that require
consensus, use a transaction.

The most important use of `Query` is to return Merkle proofs of the application state at some height
that can be used for efficient application-specific light-clients.

Note CometBFT has technically no requirements from the `Query`
message for normal operation - that is, the ABCI app developer need not implement
Query functionality if they do not wish to.

#### Query Proofs

The CometBFT block header includes a number of hashes, each providing an
anchor for some type of proof about the blockchain. The `ValidatorsHash` enables
quick verification of the validator set, the `DataHash` gives quick
verification of the transactions included in the block.

The `AppHash` is unique in that it is application specific, and allows for
application-specific Merkle proofs about the state of the application.
While some applications keep all relevant state in the transactions themselves
(like Bitcoin and its UTXOs), others maintain a separated state that is
computed deterministically *from* transactions, but is not contained directly in
the transactions themselves (like Ethereum contracts and accounts).
For such applications, the `AppHash` provides a much more efficient way to verify light-client proofs.

ABCI applications can take advantage of more efficient light-client proofs for
their state as follows:

- return the Merkle root of the deterministic application state in
  `FinalizeBlockResponse.Data`. This Merkle root will be included as the `AppHash` in the next block.
- return efficient Merkle proofs about that application state in `QueryResponse.Proof`
  that can be verified using the `AppHash` of the corresponding block.

For instance, this allows an application's light-client to verify proofs of
absence in the application state, something which is much less efficient to do using the block hash.

Some applications (eg. Ethereum, Cosmos-SDK) have multiple "levels" of Merkle trees,
where the leaves of one tree are the root hashes of others. To support this, and
the general variability in Merkle proofs, the `QueryResponse.Proof` has some minimal structure:

```protobuf
message ProofOps {
  repeated ProofOp ops = 1
}

message ProofOp {
  string type = 1;
  bytes key   = 2;
  bytes data  = 3;
}
```

Each `ProofOp` contains a proof for a single key in a single Merkle tree, of the specified `type`.
This allows ABCI to support many different kinds of Merkle trees, encoding
formats, and proofs (eg. of presence and absence) just by varying the `type`.
The `data` contains the actual encoded proof, encoded according to the `type`.
When verifying the full proof, the root hash for one ProofOp is the value being
verified for the next ProofOp in the list. The root hash of the final ProofOp in
the list should match the `AppHash` being verified against.

#### Peer Filtering

When CometBFT connects to a peer, it sends two queries to the ABCI application
using the following paths, with no additional data:

- `/p2p/filter/addr/<IP:PORT>`, where `<IP:PORT>` denote the IP address and
  the port of the connection
- `p2p/filter/id/<ID>`, where `<ID>` is the peer node ID (ie. the
  pubkey.Address() for the peer's PubKey)

If either of these queries return a non-zero ABCI code, CometBFT will refuse
to connect to the peer.

#### Paths

Queries are directed at paths, and may optionally include additional data.

The expectation is for there to be some number of high level paths
differentiating concerns, like `/p2p`, `/store`, and `/app`. Currently,
CometBFT only uses `/p2p`, for filtering peers. For more advanced use, see the
implementation of
[Query in the Cosmos-SDK](https://github.com/cosmos/cosmos-sdk/blob/e2037f7696fed4fdd4bc076f9e7053fe8178a881/baseapp/abci.go#L557-L565).

### Crash Recovery

CometBFT and the application are expected to crash together and there should not
exist a scenario where the application has persisted state of a height greater than the
latest height persisted by CometBFT.

In practice, persisting the state of a height consists of three steps, the last of which
is the call to the application's `Commit` method, the only place where the application is expected to
persist/commit its state.
On startup (upon recovery), CometBFT calls the `Info` method on the Info Connection to get the latest
committed state of the app. The app MUST return information consistent with the
last block for which it successfully completed `Commit`.

The three steps performed before the state of a height is considered persisted are:

- The block is stored by CometBFT in the blockstore
- CometBFT has stored the state returned by the application through `FinalizeBlockResponse`
- The application has committed its state within `Commit`.

The following diagram depicts the order in which these events happen, and the corresponding
ABCI functions that are called and executed by CometBFT and the application:


```
APP:                                              Execute block                         Persist application state
                                                 /     return ResultFinalizeBlock            /
                                                /                                           /
Event: ------------- block_stored ------------ / ------------ state_stored --------------- / ----- app_persisted_state
                          |                   /                   |                       /        |
CometBFT: Decide --- Persist block -- Call FinalizeBlock - Persist results ---------- Call Commit --
            on        in the                                (txResults, validator
           Block      block store                              updates...)

```

As these three steps are not atomic, we observe different cases based on which steps have been executed
before the crash occurred
(we assume that at least `block_stored` has been executed, otherwise, there is no state persisted,
and the operations for this height are repeated entirely):

- `block_stored`: we replay `FinalizeBlock` and the steps afterwards.
- `block_stored` and `state_stored`: As the app did not persist its state within `Commit`, we need to re-execute
  `FinalizeBlock` to retrieve the results and compare them to the state stored by CometBFT within `state_stored`.
  The expected case is that the states will match, otherwise CometBFT panics.
- `block_stored`, `state_stored`, `app_persisted_state`: we move on to the next height.

Based on the sequence of these events, CometBFT will panic if any of the steps in the sequence happen out of order,
that is if:

- The application has persisted a block at a height higher than the blocked saved during `state_stored`.
- The `block_stored` step persisted a block at a height smaller than the `state_stored`
- And the difference between the heights of the blocks persisted by `state_stored` and `block_stored` is more
than 1 (this corresponds to a scenario where we stored two blocks in the block store but never persisted the state of the first
block, which should never happen).

A special case is when a crash happens before the first block is committed - that is, after calling
`InitChain`. In that case, the application's state should still be at height 0 and thus `InitChain`
will be called again.


### State Sync

A new node joining the network can simply join consensus at the genesis height and replay all
historical blocks until it is caught up. However, for large chains this can take a significant
amount of time, often on the order of days or weeks.

State sync is an alternative mechanism for bootstrapping a new node, where it fetches a snapshot
of the state machine at a given height and restores it. Depending on the application, this can
be several orders of magnitude faster than replaying blocks.

Note that state sync does not currently backfill historical blocks, so the node will have a
truncated block history - users are advised to consider the broader network implications of this in
terms of block availability and auditability. This functionality may be added in the future.

For details on the specific ABCI calls and types, see the
[methods](abci%2B%2B_methods.md) section.

#### Taking Snapshots

Applications that want to support state syncing must take state snapshots at regular intervals. How
this is accomplished is entirely up to the application. A snapshot consists of some metadata and
a set of binary chunks in an arbitrary format:

- `Height (uint64)`: The height at which the snapshot is taken. It must be taken after the given
  height has been committed, and must not contain data from any later heights.

- `Format (uint32)`: An arbitrary snapshot format identifier. This can be used to version snapshot
  formats, e.g. to switch from Protobuf to MessagePack for serialization. The application can use
  this when restoring to choose whether to accept or reject a snapshot.

- `Chunks (uint32)`: The number of chunks in the snapshot. Each chunk contains arbitrary binary
  data, and should be less than 16 MB; 10 MB is a good starting point.

- `Hash ([]byte)`: An arbitrary hash of the snapshot. This is used to check whether a snapshot is
  the same across nodes when downloading chunks.

- `Metadata ([]byte)`: Arbitrary snapshot metadata, e.g. chunk hashes for verification or any other
  necessary info.

For a snapshot to be considered the same across nodes, all of these fields must be identical. When
sent across the network, snapshot metadata messages are limited to 4 MB.

When a new node is running state sync and discovering snapshots, CometBFT will query an existing
application via the ABCI `ListSnapshots` method to discover available snapshots, and load binary
snapshot chunks via `LoadSnapshotChunk`. The application is free to choose how to implement this
and which formats to use, but must provide the following guarantees:

- **Consistent:** A snapshot must be taken at a single isolated height, unaffected by
  concurrent writes. This can be accomplished by using a data store that supports ACID
  transactions with snapshot isolation.

- **Asynchronous:** Taking a snapshot can be time-consuming, so it must not halt chain progress,
  for example by running in a separate thread.

- **Deterministic:** A snapshot taken at the same height in the same format must be identical
  (at the byte level) across nodes, including all metadata. This ensures good availability of
  chunks, and that they fit together across nodes.

A very basic approach might be to use a datastore with MVCC transactions (such as RocksDB),
start a transaction immediately after block commit, and spawn a new thread which is passed the
transaction handle. This thread can then export all data items, serialize them using e.g.
Protobuf, hash the byte stream, split it into chunks, and store the chunks in the file system
along with some metadata - all while the blockchain is applying new blocks in parallel.

A more advanced approach might include incremental verification of individual chunks against the
chain app hash, parallel or batched exports, compression, and so on.

Old snapshots should be removed after some time - generally only the last two snapshots are needed
(to prevent the last one from being removed while a node is restoring it).

#### Bootstrapping a Node

An empty node can be state synced by setting the configuration option `statesync.enabled =
true`. The node also needs the chain genesis file for basic chain info, and configuration for
light client verification of the restored snapshot: a set of CometBFT RPC servers, and a
trusted header hash and corresponding height from a trusted source, via the `statesync`
configuration section.

Once started, the node will connect to the P2P network and begin discovering snapshots. These
will be offered to the local application via the `OfferSnapshot` ABCI method. Once a snapshot
is accepted CometBFT will fetch and apply the snapshot chunks. After all chunks have been
successfully applied, CometBFT verifies the app's `AppHash` against the chain using the light
client, then switches the node to normal consensus operation.

##### Snapshot Discovery

When the empty node joins the P2P network, it asks all peers to report snapshots via the
`ListSnapshots` ABCI call (limited to 10 per node). After some time, the node picks the most
suitable snapshot (generally prioritized by height, format, and number of peers), and offers it
to the application via `OfferSnapshot`. The application can choose a number of responses,
including accepting or rejecting it, rejecting the offered format, rejecting the peer who sent
it, and so on. CometBFT will keep discovering and offering snapshots until one is accepted or
the application aborts.

##### Snapshot Restoration

Once a snapshot has been accepted via `OfferSnapshot`, CometBFT begins downloading chunks from
any peers that have the same snapshot (i.e. that have identical metadata fields). Chunks are
spooled in a temporary directory, and then given to the application in sequential order via
`ApplySnapshotChunk` until all chunks have been accepted.

The method for restoring snapshot chunks is entirely up to the application.

During restoration, the application can respond to `ApplySnapshotChunk` with instructions for how
to continue. This will typically be to accept the chunk and await the next one, but it can also
ask for chunks to be refetched (either the current one or any number of previous ones), P2P peers
to be banned, snapshots to be rejected or retried, and a number of other responses - see the ABCI
reference for details.

If CometBFT fails to fetch a chunk after some time, it will reject the snapshot and try a
different one via `OfferSnapshot` - the application can choose whether it wants to support
restarting restoration, or simply abort with an error.

##### Snapshot Verification

Once all chunks have been accepted, CometBFT issues an `Info` ABCI call to retrieve the
`LastBlockAppHash`. This is compared with the trusted app hash from the chain, retrieved and
verified using the light client. CometBFT also checks that `LastBlockHeight` corresponds to the
height of the snapshot.

This verification ensures that an application is valid before joining the network. However, the
snapshot restoration may take a long time to complete, so applications may want to employ additional
verification during the restore to detect failures early. This might e.g. include incremental
verification of each chunk against the app hash (using bundled Merkle proofs), checksums to
protect against data corruption by the disk or network, and so on. However, it is important to
note that the only trusted information available is the app hash, and all other snapshot metadata
can be spoofed by adversaries.

Apps may also want to consider state sync denial-of-service vectors, where adversaries provide
invalid or harmful snapshots to prevent nodes from joining the network. The application can
counteract this by asking CometBFT to ban peers. As a last resort, node operators can use
P2P configuration options to whitelist a set of trusted peers that can provide valid snapshots.

##### Transition to Consensus

Once the snapshots have all been restored, CometBFT gathers additional information necessary for
bootstrapping the node (e.g. chain ID, consensus parameters, validator sets, and block headers)
from the genesis file and light client RPC servers. It also calls `Info` to verify the following:

- that the app hash from the snapshot it has delivered to the Application matches the apphash
  stored in the next height's block
- that the version that the Application returns in `InfoResponse` matches the version in the
  current height's block header

Once the state machine has been restored and CometBFT has gathered this additional
information, it transitions to consensus. As of ABCI 2.0, CometBFT ensures the necessary conditions
to switch are met [RFC-100](../../docs/references/rfc/rfc-100-abci-vote-extension-propag.md#base-implementation-persist-and-propagate-extended-commit-history).
From the application's point of view, these operations are transparent, unless the application has just upgraded to ABCI 2.0.
In that case, the application needs to be properly configured and aware of certain constraints in terms of when
to provide vote extensions. More details can be found in the section below.

Once a node switches to consensus, it operates like any other node, apart from having a truncated block history at the height of the restored snapshot.

## Application configuration required to switch to ABCI 2.0

Introducing vote extensions requires changes to the configuration of the application.

First of all, switching to a version of CometBFT with vote extensions, requires a coordinated upgrade.
For a detailed description on the upgrade path, please refer to the corresponding
[section](../../docs/references/rfc/rfc-100-abci-vote-extension-propag.md#upgrade-path) in RFC-100.

There is a newly introduced [**consensus parameter**](./abci%2B%2B_app_requirements.md#abciparamsvoteextensionsenableheight): `VoteExtensionsEnableHeight`.
This parameter represents the height at which vote extensions are
required for consensus to proceed, with 0 being the default value (vote extensions disabled).
A chain can enable vote extensions either:
- at genesis by setting `VoteExtensionsEnableHeight` to be equal, e.g., to the `InitialHeight`
- or via the application logic by changing the `ConsensusParam` to configure the
`VoteExtensionsEnableHeight`.

Once the (coordinated) upgrade to ABCI 2.0 has taken place, at height  *h<sub>u</sub>*,
the value of `VoteExtensionsEnableHeight` MAY be set to some height, *h<sub>e</sub>*,
which MUST be higher than the current height of the chain. Thus the earliest value for
 *h<sub>e</sub>* is  *h<sub>u</sub>* + 1.

Once a node reaches the configured height,
for all heights *h ≥ h<sub>e</sub>*, the consensus algorithm will
reject as invalid any precommit messages that do not have signed vote extension data.
If the application requires it, a 0-length vote extension is allowed, but it MUST be signed
and present in the precommit message.
Likewise, for all heights *h < h<sub>e</sub>*, any precommit messages that *do* have vote extensions
will also be rejected as malformed.
Height *h<sub>e</sub>* is somewhat special, as calls to `PrepareProposal` MUST NOT
have vote extension data, but all precommit votes in that height MUST carry a vote extension,
even if the extension is `nil`.
Height *h<sub>e</sub> + 1* is the first height for which `PrepareProposal` MUST have vote
extension data and all precommit votes in that height MUST have a vote extension.

Corollary, [CometBFT will decide](./abci%2B%2B_comet_expected_behavior.md#handling-upgrades-to-abci-20) which data to store, and require for successful operations, based on the current height
of the chain.
