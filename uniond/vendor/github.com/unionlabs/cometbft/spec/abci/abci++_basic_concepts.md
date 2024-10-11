---
order: 1
title: Overview and basic concepts
---

## Outline

- [Overview and basic concepts](#overview-and-basic-concepts)
  - [ABCI 2.0 vs. legacy ABCI](#abci-20-vs-legacy-abci)
  - [Method overview](#method-overview)
    - [Consensus/block execution methods](#consensusblock-execution-methods)
    - [Mempool methods](#mempool-methods)
    - [Info methods](#info-methods)
    - [State-sync methods](#state-sync-methods)
    - [Other methods](#other-methods)
  - [Proposal timeout](#proposal-timeout)
  - [Deterministic State-Machine Replication](#deterministic-state-machine-replication)
  - [Events](#events)
  - [Evidence of Misbehavior](#evidence-of-misbehavior)
  - [Errors](#errors)
    - [`CheckTx`](#checktx)
    - [`ExecTxResult` (as part of `FinalizeBlock`)](#exectxresult-as-part-of-finalizeblock)
    - [`Query`](#query)
# Overview and basic concepts

## ABCI 2.0 vs. legacy ABCI

[&#8593; Back to Outline](#outline)

The Application's main role is to execute blocks decided (a.k.a. finalized) by consensus. The
decided blocks are the consensus's main output to the (replicated) Application. With ABCI, the
application only interacts with consensus at *decision* time. This restricted mode of interaction
prevents numerous features for the Application, including many scalability improvements that are
now better understood than when ABCI was first written. For example, many ideas proposed to improve
scalability can be boiled down to "make the block proposers do work, so the network does not have
to". This includes optimizations such as transaction level signature aggregation, state transition
proofs, etc. Furthermore, many new security properties cannot be achieved in the current paradigm,
as the Application cannot require validators to do more than executing the transactions contained in
finalized blocks. This includes features such as threshold cryptography, and guaranteed IBC
connection attempts.

ABCI 2.0 addresses these limitations by allowing the application to intervene at three key places of
consensus execution: (a) at the moment a new proposal is to be created, (b) at the moment a
proposal is to be validated, and (c) at the moment a (precommit) vote is sent/received.
The new interface allows block proposers to perform application-dependent
work in a block through the `PrepareProposal` method (a); and validators to perform application-dependent work
and checks in a proposed block through the `ProcessProposal` method (b); and applications to require their validators
to do more than just validate blocks through the `ExtendVote` and `VerifyVoteExtension` methods (c).

Furthermore, ABCI 2.0 coalesces {`BeginBlock`, [`DeliverTx`], `EndBlock`} into `FinalizeBlock`, as a
simplified, efficient way to deliver a decided block to the Application.

## Method overview


[&#8593; Back to Outline](#outline)

Methods can be classified into four categories: *consensus*, *mempool*, *info*, and *state-sync*.

### Consensus/block execution methods

The first time a new blockchain is started, CometBFT calls `InitChain`. From then on, method
`FinalizeBlock` is executed upon the decision of each block, resulting in an updated Application
state. During the execution of an instance of consensus, which decides the block for a given
height, and before method `FinalizeBlock` is called, methods `PrepareProposal`, `ProcessProposal`,
`ExtendVote`, and `VerifyVoteExtension` may be called several times. See
[CometBFT's expected behavior](abci++_comet_expected_behavior.md) for details on the possible
call sequences of these methods.

- [**InitChain:**](./abci++_methods.md#initchain) This method initializes the blockchain.
  CometBFT calls it once upon genesis.

- [**PrepareProposal:**](./abci++_methods.md#prepareproposal) It allows the block
  proposer to perform application-dependent work in a block before proposing it.
  This enables, for instance, batch optimizations to a block, which has been empirically
  demonstrated to be a key component for improved performance. Method `PrepareProposal` is called
  every time CometBFT is about to broadcast a Proposal message and *validValue* is `nil`.
  CometBFT gathers outstanding transactions from the
  mempool, generates a block header, and uses them to create a block to propose. Then, it calls
  `PrepareProposal` with the newly created proposal, called *raw proposal*. The Application
  can make changes to the raw proposal, such as reordering, adding and removing transactions, before returning the
  (potentially) modified proposal, called *prepared proposal* in the `PrepareProposalResponse`.
  The logic modifying the raw proposal MAY be non-deterministic.

- [**ProcessProposal:**](./abci++_methods.md#processproposal) It allows a validator to
  perform application-dependent work in a proposed block. This enables features such as immediate
  block execution, and allows the Application to reject invalid blocks.

  CometBFT calls it when it receives a proposal and *validValue* is `nil`.
  The Application cannot modify the proposal at this point but can reject it if
  invalid. If that is the case, the consensus algorithm will prevote `nil` on the proposal, which has
  strong liveness implications for CometBFT. As a general rule, the Application
  SHOULD accept a prepared proposal passed via `ProcessProposal`, even if a part of
  the proposal is invalid (e.g., an invalid transaction); the Application can
  ignore the invalid part of the prepared proposal at block execution time.
  The logic in `ProcessProposal` MUST be deterministic.

- [**ExtendVote:**](./abci++_methods.md#extendvote) It allows applications to let their
  validators do more than just validate within consensus. `ExtendVote` allows applications to
  include non-deterministic data, opaque to the consensus algorithm, to precommit messages (the final round of
  voting). The data, called *vote extension*, will be broadcast and received together with the
  vote it is extending, and will be made available to the Application in the next height,
  in the rounds where the local process is the proposer.
  CometBFT calls `ExtendVote` when the consensus algorithm is about to send a non-`nil` precommit message.
  If the Application does not have vote extension information to provide at that time, it returns
  a 0-length byte array as its vote extension.
  The logic in `ExtendVote` MAY be non-deterministic.

- [**VerifyVoteExtension:**](./abci++_methods.md#verifyvoteextension) It allows
  validators to validate the vote extension data attached to a precommit message. If the validation
  fails, the whole precommit message will be deemed invalid and ignored by consensus algorithm.
  This has a negative impact on liveness, i.e., if vote extensions repeatedly cannot be
  verified by correct validators, the consensus algorithm may not be able to finalize a block even if sufficiently
  many (+2/3) validators send precommit votes for that block. Thus, `VerifyVoteExtension`
  should be implemented with special care.
  As a general rule, an Application that detects an invalid vote extension SHOULD
  accept it in `VerifyVoteExtensionResponse` and ignore it in its own logic. CometBFT calls it when
  a process receives a precommit message with a (possibly empty) vote extension, for the current height. It is not called for precommit votes received after the height is concluded but while waiting to accumulate more precommit votes.
  The logic in `VerifyVoteExtension` MUST be deterministic.

- [**FinalizeBlock:**](./abci++_methods.md#finalizeblock) It delivers a decided block to the
  Application. The Application must execute the transactions in the block deterministically and
  update its state accordingly. Cryptographic commitments to the block and transaction results,
  returned via the corresponding parameters in `FinalizeBlockResponse`, are included in the header
  of the next block. CometBFT calls it when a new block is decided.
  When calling `FinalizeBlock` with a block, the consensus algorithm run by CometBFT guarantees
  that at least one non-byzantine validator has run `ProcessProposal` on that block.

- [**Commit:**](./abci++_methods.md#commit) Instructs the Application to persist its
  state. It is a fundamental part of CometBFT's crash-recovery mechanism that ensures the
  synchronization between CometBFT and the Application upon recovery. CometBFT calls it just after
  having persisted the data returned by calls to `FinalizeBlockResponse`. The Application can now discard
  any state or data except the one resulting from executing the transactions in the decided block.

### Mempool methods

- [**CheckTx:**](./abci++_methods.md#checktx) This method allows the Application to validate
  transactions. Validation can be stateless (e.g., checking signatures ) or stateful
  (e.g., account balances). The type of validation performed is up to the application. If a
  transaction passes the validation, then CometBFT adds it to the mempool; otherwise the
  transaction is discarded.
  CometBFT calls it when it receives a new transaction either coming from an external
  user (e.g., a client) or another node. Furthermore, CometBFT can be configured to call
  re-`CheckTx` on all outstanding transactions in the mempool after calling `Commit` for a block.

### Info methods

- [**Info:**](./abci++_methods.md#info) Used to sync CometBFT with the Application during a
  handshake that happens upon recovery, or on startup when state-sync is used.

- [**Query:**](./abci++_methods.md#query) This method can be used to query the Application for
  information about the application state.

### State-sync methods

State sync allows new nodes to rapidly bootstrap by discovering, fetching, and applying
state machine (application) snapshots instead of replaying historical blocks. For more details, see the
[state sync documentation](../p2p/legacy-docs/messages/state-sync.md).

New nodes discover and request snapshots from other nodes in the P2P network.
A CometBFT node that receives a request for snapshots from a peer will call
`ListSnapshots` on its Application. The Application returns the list of locally available
snapshots.
Note that the list does not contain the actual snapshots but metadata about them: height at which
the snapshot was taken, application-specific verification data and more (see
[snapshot data type](./abci++_methods.md#snapshot) for more details). After receiving a
list of available snapshots from a peer, the new node can offer any of the snapshots in the list to
its local Application via the `OfferSnapshot` method. The Application can check at this point the
validity of the snapshot metadata.

Snapshots may be quite large and are thus broken into smaller "chunks" that can be
assembled into the whole snapshot. Once the Application accepts a snapshot and
begins restoring it, CometBFT will fetch snapshot "chunks" from existing nodes.
The node providing "chunks" will fetch them from its local Application using
the `LoadSnapshotChunk` method.

As the new node receives "chunks" it will apply them sequentially to the local
application with `ApplySnapshotChunk`. When all chunks have been applied, the
Application's `AppHash` is retrieved via an `Info` query.
To ensure that the sync proceeded correctly, CometBFT compares the local Application's `AppHash`
to the `AppHash` stored on the blockchain (verified via
[light client verification](../light-client/verification/README.md)).

In summary:

- [**ListSnapshots:**](./abci++_methods.md#listsnapshots) Used by nodes to discover available
  snapshots on peers.

- [**OfferSnapshot:**](./abci++_methods.md#offersnapshot) When a node receives a snapshot from a
  peer, CometBFT uses this method to offer the snapshot to the Application.

- [**LoadSnapshotChunk:**](./abci++_methods.md#loadsnapshotchunk) Used by CometBFT to retrieve
  snapshot chunks from the Application to send to peers.

- [**ApplySnapshotChunk:**](./abci++_methods.md#applysnapshotchunk) Used by CometBFT to hand
  snapshot chunks to the Application.

### Other methods

Additionally, there is a [**Flush**](./abci++_methods.md#flush) method that is called on every connection,
and an [**Echo**](./abci++_methods.md#echo) method that is used for debugging.

More details on managing state across connections can be found in the section on
[Managing Application State](./abci%2B%2B_app_requirements.md#managing-the-application-state-and-related-topics).

## Proposal timeout

`PrepareProposal` stands on the consensus algorithm critical path,
i.e., CometBFT cannot make progress while this method is being executed.
Hence, if the Application takes a long time preparing a proposal,
the default value of *TimeoutPropose* might not be sufficient
to accommodate the method's execution and validator nodes might time out and prevote `nil`.
The proposal, in this case, will probably be rejected and a new round will be necessary.

Timeouts are automatically increased for each new round of a height and, if the execution of `PrepareProposal` is bound, eventually *TimeoutPropose*  will be long enough to accommodate the execution of `PrepareProposal`.
However, relying on this self adaptation could lead to performance degradation and, therefore,
operators are suggested to adjust the initial value of *TimeoutPropose* in CometBFT's configuration file,
in order to suit the needs of the particular application being deployed.

This is particularly important if applications implement *immediate execution*.
To implement this technique, proposers need to execute the block being proposed within `PrepareProposal`, which could take longer than *TimeoutPropose*.

## Deterministic State-Machine Replication

[&#8593; Back to Outline](#outline)

ABCI applications must implement deterministic finite-state machines to be
securely replicated by the CometBFT consensus engine. This means block execution
must be strictly deterministic: given the same
ordered set of transactions, all nodes will compute identical responses, for all
successive `FinalizeBlock` calls. This is critical because the
responses are included in the header of the next block, either via a Merkle root
or directly, so all nodes must agree on exactly what they are.

For this reason, it is recommended that application state is not exposed to any
external user or process except via the ABCI connections to a consensus engine
like CometBFT. The Application must only change its state based on input
from block execution (`FinalizeBlock` calls), and not through
any other kind of request. This is the only way to ensure all nodes see the same
transactions and compute the same results.

Applications that implement immediate execution (execute the blocks
that are about to be proposed, in `PrepareProposal`, or that require validation, in `ProcessProposal`) produce a new candidate state before a block is decided.
The state changes caused by processing those
proposed blocks must never replace the previous state until `FinalizeBlock` confirms
that the proposed block was decided and `Commit` is invoked for it.

The same is true to Applications that quickly accept blocks and execute the
blocks optimistically in parallel with the remaining consensus steps to save
time during `FinalizeBlock`; they must only apply state changes in `Commit`.

Additionally, vote extensions or the validation thereof (via `ExtendVote` or
`VerifyVoteExtension`) must *never* have side effects on the current state.
Their data can only be used when provided in a `PrepareProposal` call but, again,
without side effects to the app state.

If there is some non-determinism in the state machine, consensus will eventually
fail as nodes disagree over the correct values for the block header. The
non-determinism must be fixed and the nodes restarted.

Sources of non-determinism in applications may include:

- Hardware failures
    - Cosmic rays, overheating, etc.
- Node-dependent state
    - Random numbers
    - Time
- Underspecification
    - Library version changes
    - Race conditions
    - Floating point numbers
    - JSON or protobuf serialization
    - Iterating through hash-tables/maps/dictionaries
- External Sources
    - Filesystem
    - Network calls (eg. some external REST API service)

See [#56](https://github.com/tendermint/abci/issues/56) for the original discussion.

Note that some methods (e.g., `Query` and `FinalizeBlock`) may return
non-deterministic data in the form of `Info`, `Log` and/or `Events` fields. The
`Log` is intended for the literal output from the Application's logger, while
the `Info` is any additional info that should be returned. These fields are not
included in block header computations, so we don't need agreement on them. See
each field's description on whether it must be deterministic or not.

## Events

[&#8593; Back to Outline](#outline)

Method `FinalizeBlock` includes an `events` field at the top level in its
`FinalizeBlockResponse`, and one `events` field per transaction included in the block.
Applications may respond to this ABCI 2.0 method with an event list for each executed
transaction, and a general event list for the block itself.
Events allow applications to associate metadata with transactions and blocks.
Events returned via `FinalizeBlock` do not impact the consensus algorithm in any way
and instead exist to power subscriptions and queries of CometBFT state.

An `Event` contains a `type` and a list of `EventAttributes`, which are key-value
string pairs denoting metadata about what happened during the method's (or transaction's)
execution. `Event` values can be used to index transactions and blocks according to what
happened during their execution.

Each event has a `type` which is meant to categorize the event for a particular
`FinalizeBlockResponse` or `Tx`. A `FinalizeBlockResponse` or `Tx` may contain
multiple events with duplicate
`type` values, where each distinct entry is meant to categorize attributes for a
particular event. Every key and value in an event's attributes must be UTF-8
encoded strings along with the event type itself.

```protobuf
message Event {
  string                  type       = 1;
  repeated EventAttribute attributes = 2;
}
```

The attributes of an `Event` consist of a `key`, a `value`, and an `index`
flag. The index flag notifies the CometBFT indexer to index the attribute.

The `type` and `attributes` fields are non-deterministic and may vary across
different nodes in the network.

```protobuf
message EventAttribute {
  string key   = 1;
  string value = 2;
  bool   index = 3;  // nondeterministic
}
```

Example:

```go
 abci.FinalizeBlockResponse{
  // ...
 Events: []abci.Event{
  {
   Type: "validator.provisions",
   Attributes: []abci.EventAttribute{
    abci.EventAttribute{Key: "address", Value: "...", Index: true},
    abci.EventAttribute{Key: "amount", Value: "...", Index: true},
    abci.EventAttribute{Key: "balance", Value: "...", Index: true},
   },
  },
  {
   Type: "validator.provisions",
   Attributes: []abci.EventAttribute{
    abci.EventAttribute{Key: "address", Value: "...", Index: true},
    abci.EventAttribute{Key: "amount", Value: "...", Index: false},
    abci.EventAttribute{Key: "balance", Value: "...", Index: false},
   },
  },
  {
   Type: "validator.slashed",
   Attributes: []abci.EventAttribute{
    abci.EventAttribute{Key: "address", Value: "...", Index: false},
    abci.EventAttribute{Key: "amount", Value: "...", Index: true},
    abci.EventAttribute{Key: "reason", Value: "...", Index: true},
   },
  },
  // ...
 },
}
```

## Evidence of Misbehavior

[&#8593; Back to Outline](#outline)

CometBFT's security model relies on the use of evidence of misbehavior. An evidence is an
irrefutable proof of malicious behavior by a network participant. It is the responsibility of
CometBFT to detect such malicious behavior. When malicious behavior is detected, CometBFT
will gossip evidence of misbehavior to other nodes and commit the evidence to
the chain once they are verified by a subset of validators. These evidence of misbehavior will then be
passed on to the Application through ABCI. It is the responsibility of the
Application to handle the evidence of misbehavior and exercise punishment.

There are two forms of misbehavior: `Duplicate Vote` and `Light Client Attack`. More
information can be found in the consensus [evidence](../consensus/evidence.md) document.

`MisbehaviorType` has the following protobuf format:

```protobuf
// The type of misbehavior committed by a validator.
enum MisbehaviorType {
  // Unknown
  MISBEHAVIOR_TYPE_UNKNOWN = 0;
  // Duplicate vote
  MISBEHAVIOR_TYPE_DUPLICATE_VOTE = 1;
  // Light client attack
  MISBEHAVIOR_TYPE_LIGHT_CLIENT_ATTACK = 2;
}
```

## Returning Errors

[&#8593; Back to Outline](#outline)

Please note that the method signature for the ABCI methods includes a response and an error return, such as
`(*abcitypes.[Method_Name]Response, error)`.

### ABCI response error codes (e.g. `Code` and `Codespace`)

Some of the ABCI methods' responses feature a field (e.g., the `Code` field) that can be used to return an error
in the `[Method_Name]Response`. These fields play a significant role as they can return an error in the response, indicating
to CometBFT that a problem has occurred during data processing, such as transaction validation or a query.

The `Query` and `CheckTx` methods include a `Code` field in their `*Response`.
Field `Code` is meant to contain an application-specific response code.
A response code of `0` indicates no error.  Any other response where the `Code` field is
different from `0` indicates to CometBFT that an error occurred.

These methods also return a `Codespace` string to CometBFT. This field is
used to disambiguate `Code` values returned by different domains of the
Application. The `Codespace` is a namespace for the `Code`.

Method `FinalizeBlock` is a special case. It contains a number of
`Code` and `Codespace` fields as part of type `ExecTxResult`. Each of
these codes reports errors related to the transaction it is attached to.
However, `FinalizeBlock` does not return errors at the top level, so the
same considerations on critical issues made for `Echo`, `Info`, and
`InitChain` also apply here.

The handling of non-zero response codes by CometBFT is described below.

### `CheckTx`

When CometBFT receives a `CheckTxResponse` with a non-zero `Code`, the associated
transaction will not be added to CometBFT's mempool, or it will be removed if
it is already included.

### `ExecTxResult` (as part of `FinalizeBlock`)

The `ExecTxResult` type delivers transaction results from the Application to CometBFT. When
CometBFT receives a `FinalizeBlockResponse` containing an `ExecTxResult` with a non-zero `Code`,
the response code is logged. Past `Code` values can be queried by clients. As the transaction was
part of a decided block, the `Code` does not influence consensus.

### `Query`

When CometBFT receives a `QueryResponse` with a non-zero `Code`, this code is
returned directly to the client that initiated the query.

### ABCI methods' `error` return

The `error` return, the second object returned in an ABCI method e.g., `(*abcitypes.[Method_Name]Response, error)`, is utilized in situations
involving unrecoverable errors.

All ABCI methods return errors. An error returned in any of these methods represents a critical issue that CometBFT
has no reasonable way to handle. Therefore, if there is an error in one of these methods, CometBFT will crash
to ensure that an operator safely handles the error: the application must be terminated to avoid any further unintended consequences.

As a result, upon detecting a non-recoverable error condition, the application has the choice of either
(a) crashing itself (e.g., a`panic` in the code detecting the unrecoverable condition), or
(b) returning an error as the second return value in the ABCI method that detects the error condition,
knowing that CometBFT will panic upon receiving the error.
The choice between (a) and (b) is up to the application -- both are equivalent -- and depends on
whether an application (e.g. running in a different process that CometBFT)
prefers CometBFT to crash first.

