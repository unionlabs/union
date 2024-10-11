---
order: 2
title: Methods
---

# Methods

## Methods existing in ABCI

### Echo

* **Request**:
    * `Message (string)`: A string to echo back
* **Response**:
    * `Message (string)`: The input string
* **Usage**:
    * Echo a string to test an ABCI client/server implementation

### Flush

* **Usage**:
    * Signals that messages queued on the client should be flushed to
    the server. It is called periodically by the client
    implementation to ensure asynchronous requests are actually
    sent, and is called immediately to make a synchronous request,
    which returns when the Flush response comes back.

### Info

* **Request**:

    | Name          | Type   | Description                            | Field Number |
    |---------------|--------|----------------------------------------|--------------|
    | version       | string | The CometBFT software semantic version | 1            |
    | block_version | uint64 | The CometBFT Block version             | 2            |
    | p2p_version   | uint64 | The CometBFT P2P version               | 3            |
    | abci_version  | string | The CometBFT ABCI semantic version     | 4            |

* **Response**:

    | Name                | Type   | Description                                         | Field Number | Deterministic |
    |---------------------|--------|-----------------------------------------------------|--------------|---------------|
    | data                | string | Some arbitrary information                          | 1            | N/A           |
    | version             | string | The application software semantic version           | 2            | N/A           |
    | app_version         | uint64 | The application version                             | 3            | N/A           |
    | last_block_height   | int64  | Latest height for which the app persisted its state | 4            | N/A           |
    | last_block_app_hash | bytes  | Latest AppHash returned by `FinalizeBlock`          | 5            | N/A           |

* **Usage**:
    * Return information about the application state.
    * Used to sync CometBFT with the application during a handshake
      that happens on startup or on recovery.
    * The returned `app_version` will be included in the Header of every block.
    * CometBFT expects `last_block_app_hash` and `last_block_height` to
      be updated and persisted during `Commit`.

> Note: Semantic version is a reference to [semantic versioning](https://semver.org/). Semantic versions in info will be displayed as X.X.x.

### InitChain

* **Request**:

    | Name             | Type                                            | Description                                         | Field Number |
    |------------------|-------------------------------------------------|-----------------------------------------------------|--------------|
    | time             | [google.protobuf.Timestamp][protobuf-timestamp] | Genesis time                                        | 1            |
    | chain_id         | string                                          | ID of the blockchain.                               | 2            |
    | consensus_params | [ConsensusParams](#consensusparams)             | Initial consensus-critical parameters.              | 3            |
    | validators       | repeated [ValidatorUpdate](#validatorupdate)    | Initial genesis validators, sorted by voting power. | 4            |
    | app_state_bytes  | bytes                                           | Serialized initial application state. JSON bytes.   | 5            |
    | initial_height   | int64                                           | Height of the initial block (typically `1`).        | 6            |

* **Response**:

    | Name             | Type                                         | Description                                      | Field Number | Deterministic |
    |------------------|----------------------------------------------|--------------------------------------------------|--------------|---------------|
    | consensus_params | [ConsensusParams](#consensusparams)          | Initial consensus-critical parameters (optional) | 1            | Yes           |
    | validators       | repeated [ValidatorUpdate](#validatorupdate) | Initial validator set (optional).                | 2            | Yes           |
    | app_hash         | bytes                                        | Initial application hash.                        | 3            | Yes           |

* **Usage**:
    * Called once upon genesis.
    * If `InitChainResponse.Validators` is empty, the initial validator set will be the `InitChainRequest.Validators`
    * If `InitChainResponse.Validators` is not empty, it will be the initial
      validator set (regardless of what is in `InitChainRequest.Validators`).
    * This allows the app to decide if it wants to accept the initial validator
      set proposed by CometBFT (ie. in the genesis file), or if it wants to use
      a different one (perhaps computed based on some application specific
      information in the genesis file).
    * Both `InitChainRequest.Validators` and `InitChainResponse.Validators` are [ValidatorUpdate](#validatorupdate) structs.
      So, technically, they both are _updating_ the set of validators from the empty set.

### Query

* **Request**:

    | Name   | Type   | Description                                                                                                                                                                                                                                                                                                                                                                                                                                                          | Field Number |
    |--------|--------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------|
    | data   | bytes  | Request parameters for the application to interpret analogously to a [URI query component](https://www.rfc-editor.org/rfc/rfc3986#section-3.4). Can be used with or in lieu of `path`.                                                                                                                                                                                                                                                                               | 1            |
    | path   | string | A request path for the application to interpret analogously to a [URI path component](https://www.rfc-editor.org/rfc/rfc3986#section-3.3) in e.g. routing. Can be used with or in lieu of `data`. Applications MUST interpret "/store" or any path starting with "/store/" as a query by key on the underlying store, in which case a key SHOULD be specified in `data`. Applications SHOULD allow queries over specific types like `/accounts/...` or `/votes/...`. | 2            |
    | height | int64  | The block height against which to query (default=0 returns data for the latest committed block). Note that this is the height of the block containing the application's Merkle root hash, which represents the state as it was after committing the block at Height-1.                                                                                                                                                                                               | 3            |
    | prove  | bool   | Return Merkle proof with response if possible.                                                                                                                                                                                                                                                                                                                                                                                                                       | 4            |

* **Response**:

    | Name      | Type                  | Description                                                                                                                                                                                                        | Field Number | Deterministic |
    |-----------|-----------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------|---------------|
    | code      | uint32                | Response code.                                                                                                                                                                                                     | 1            | N/A           |
    | log       | string                | The output of the application's logger.                                                                                                                                                                            | 3            | N/A           |
    | info      | string                | Additional information.                                                                                                                                                                                            | 4            | N/A           |
    | index     | int64                 | The index of the key in the tree.                                                                                                                                                                                  | 5            | N/A           |
    | key       | bytes                 | The key of the matching data.                                                                                                                                                                                      | 6            | N/A           |
    | value     | bytes                 | The value of the matching data.                                                                                                                                                                                    | 7            | N/A           |
    | proof_ops | [ProofOps](#proofops) | Serialized proof for the value data, if requested, to be verified against the `app_hash` for the given Height.                                                                                                     | 8            | N/A           |
    | height    | int64                 | The block height from which data was derived. Note that this is the height of the block containing the application's Merkle root hash, which represents the state as it was after committing the block at Height-1 | 9            | N/A           |
    | codespace | string                | Namespace for the `code`.                                                                                                                                                                                          | 10           | N/A           |

* **Usage**:
    * Query for data from the application at current or past height.
    * Optionally return Merkle proof.
    * Merkle proof includes self-describing `type` field to support many types
    of Merkle trees and encoding formats.

### CheckTx

* **Request**:

    | Name | Type        | Description                                                                                                                                                                                                                             | Field Number |
    |------|-------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------|
    | tx   | bytes       | The request transaction bytes                                                                                                                                                                                                           | 1            |
    | type | CheckTxType | One of `CheckTx_New` or `CheckTx_Recheck`. `CheckTx_New` is the default and means that a full check of the tranasaction is required. `CheckTx_Recheck` types are used when the mempool is initiating a normal recheck of a transaction. | 2            |

* **Response**:

    | Name       | Type                                              | Description                                                          | Field Number | Deterministic |
    |------------|---------------------------------------------------|----------------------------------------------------------------------|--------------|---------------|
    | code       | uint32                                            | Response code.                                                       | 1            | N/A           |
    | data       | bytes                                             | Result bytes, if any.                                                | 2            | N/A           |
    | log        | string                                            | The output of the application's logger.                              | 3            | N/A           |
    | info       | string                                            | Additional information.                                              | 4            | N/A           |
    | gas_wanted | int64                                             | Amount of gas requested for transaction.                             | 5            | N/A           |
    | gas_used   | int64                                             | Amount of gas consumed by transaction.                               | 6            | N/A           |
    | events     | repeated [Event](abci++_basic_concepts.md#events) | Type & Key-Value events for indexing transactions (e.g. by account). | 7            | N/A           |
    | codespace  | string                                            | Namespace for the `code`.                                            | 8            | N/A           |

* **Usage**:

    * Technically optional - not involved in processing blocks.
    * Guardian of the mempool: every node runs `CheckTx` before letting a
      transaction into its local mempool.
    * The transaction may come from an external user or another node
    * `CheckTx` validates the transaction against the current state of the application,
      for example, checking signatures and account balances, but does not apply any
      of the state changes described in the transaction.
    * Transactions where `CheckTxResponse.Code != 0` will be rejected - they will not be broadcast
      to other nodes or included in a proposal block.
      CometBFT attributes no other value to the response code.

### Commit

#### Parameters and Types

* **Request**:

    Commit signals the application to persist application state. It takes no parameters.

* **Response**:

    | Name          | Type  | Description                                                            | Field Number | Deterministic |
    |---------------|-------|------------------------------------------------------------------------|--------------|---------------|
    | retain_height | int64 | Blocks below this height may be removed. Defaults to `0` (retain all). | 3            | No            |

* **Usage**:

    * Signal the Application to persist the application state.
      Application is expected to persist its state at the end of this call, before calling `Commit`.
    * Use `CommitResponse.retain_height` with caution! If all nodes in the network remove historical
      blocks then this data is permanently lost, and no new nodes will be able to join the network and
      bootstrap, unless state sync is enabled on the chain. Historical blocks may also be required for other purposes, e.g. auditing, replay of
      non-persisted heights, light client verification, and so on.

### ListSnapshots

* **Request**:

    Empty request asking the application for a list of snapshots.

* **Response**:

    | Name      | Type                           | Description                    | Field Number | Deterministic |
    |-----------|--------------------------------|--------------------------------|--------------|---------------|
    | snapshots | repeated [Snapshot](#snapshot) | List of local state snapshots. | 1            | N/A           |

* **Usage**:
    * Used during state sync to discover available snapshots on peers.
    * See `Snapshot` data type for details.

### LoadSnapshotChunk

* **Request**:

    | Name   | Type   | Description                                                           | Field Number |
    |--------|--------|-----------------------------------------------------------------------|--------------|
    | height | uint64 | The height of the snapshot the chunk belongs to.                      | 1            |
    | format | uint32 | The application-specific format of the snapshot the chunk belongs to. | 2            |
    | chunk  | uint32 | The chunk index, starting from `0` for the initial chunk.             | 3            |

* **Response**:

    | Name  | Type  | Description                                                                                                                                            | Field Number | Deterministic |
    |-------|-------|--------------------------------------------------------------------------------------------------------------------------------------------------------|--------------|---------------|
    | chunk | bytes | The binary chunk contents, in an arbitrary format. Chunk messages cannot be larger than 16 MB _including metadata_, so 10 MB is a good starting point. | 1            | N/A           |

* **Usage**:
    * Used during state sync to retrieve snapshot chunks from peers.

### OfferSnapshot

* **Request**:

    | Name     | Type                  | Description                                                              | Field Number |
    |----------|-----------------------|--------------------------------------------------------------------------|--------------|
    | snapshot | [Snapshot](#snapshot) | The snapshot offered for restoration.                                    | 1            |
    | app_hash | bytes                 | The light client-verified app hash for this height, from the blockchain. | 2            |

* **Response**:

    | Name   | Type              | Description                       | Field Number | Deterministic |
    |--------|-------------------|-----------------------------------|--------------|---------------|
    | result | [Result](#result) | The result of the snapshot offer. | 1            | N/A           |

#### Result

```protobuf
  enum Result {
    UNKNOWN       = 0;  // Unknown result, abort all snapshot restoration
    ACCEPT        = 1;  // Snapshot is accepted, start applying chunks.
    ABORT         = 2;  // Abort snapshot restoration, and don't try any other snapshots.
    REJECT        = 3;  // Reject this specific snapshot, try others.
    REJECT_FORMAT = 4;  // Reject all snapshots with this `format`, try others.
    REJECT_SENDER = 5;  // Reject all snapshots from all senders of this snapshot, try others.
  }
```

* **Usage**:
    * `OfferSnapshot` is called when bootstrapping a node using state sync. The application may
    accept or reject snapshots as appropriate. Upon accepting, CometBFT will retrieve and
    apply snapshot chunks via `ApplySnapshotChunk`. The application may also choose to reject a
    snapshot in the chunk response, in which case it should be prepared to accept further
    `OfferSnapshot` calls.
    * Only `AppHash` can be trusted, as it has been verified by the light client. Any other data
    can be spoofed by adversaries, so applications should employ additional verification schemes
    to avoid denial-of-service attacks. The verified `AppHash` is automatically checked against
    the restored application at the end of snapshot restoration.
    * For more information, see the `Snapshot` data type or the [state sync section](../p2p/legacy-docs/messages/state-sync.md).

### ApplySnapshotChunk

* **Request**:

    | Name   | Type   | Description                                                               | Field Number |
    |--------|--------|---------------------------------------------------------------------------|--------------|
    | index  | uint32 | The chunk index, starting from `0`. CometBFT applies chunks sequentially. | 1            |
    | chunk  | bytes  | The binary chunk contents, as returned by `LoadSnapshotChunk`.            | 2            |
    | sender | string | The P2P ID of the node who sent this chunk.                               | 3            |

* **Response**:

    | Name           | Type                | Description                                                                                                                                                                                                                             | Field Number | Deterministic |
    |----------------|---------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------|---------------|
    | result         | Result  (see below) | The result of applying this chunk.                                                                                                                                                                                                      | 1            | N/A           |
    | refetch_chunks | repeated uint32     | Refetch and reapply the given chunks, regardless of `result`. Only the listed chunks will be refetched, and reapplied in sequential order.                                                                                              | 2            | N/A           |
    | reject_senders | repeated string     | Reject the given P2P senders, regardless of `Result`. Any chunks already applied will not be refetched unless explicitly requested, but queued chunks from these senders will be discarded, and new chunks or other snapshots rejected. | 3            | N/A           |

```proto
  enum Result {
    UNKNOWN         = 0;  // Unknown result, abort all snapshot restoration
    ACCEPT          = 1;  // The chunk was accepted.
    ABORT           = 2;  // Abort snapshot restoration, and don't try any other snapshots.
    RETRY           = 3;  // Reapply this chunk, combine with `RefetchChunks` and `RejectSenders` as appropriate.
    RETRY_SNAPSHOT  = 4;  // Restart this snapshot from `OfferSnapshot`, reusing chunks unless instructed otherwise.
    REJECT_SNAPSHOT = 5;  // Reject this snapshot, try a different one.
  }
```

* **Usage**:
    * The application can choose to refetch chunks and/or ban P2P peers as appropriate. CometBFT
    will not do this unless instructed by the application.
    * The application may want to verify each chunk, e.g. by attaching chunk hashes in
    `Snapshot.Metadata` and/or incrementally verifying contents against `AppHash`.
    * When all chunks have been accepted, CometBFT will make an ABCI `Info` call to verify that
    `LastBlockAppHash` and `LastBlockHeight` matches the expected values, and record the
    `AppVersion` in the node state. It then switches to block sync or consensus and joins the
    network.
    * If CometBFT is unable to retrieve the next chunk after some time (e.g. because no suitable
    peers are available), it will reject the snapshot and try a different one via `OfferSnapshot`.
    The application should be prepared to reset and accept it or abort as appropriate.

## New methods introduced in ABCI 2.0

### PrepareProposal

#### Parameters and Types

* **Request**:

    | Name                 | Type                                            | Description                                                                                   | Field Number |
    |----------------------|-------------------------------------------------|-----------------------------------------------------------------------------------------------|--------------|
    | max_tx_bytes         | int64                                           | Currently configured maximum size in bytes taken by the modified transactions.                | 1            |
    | txs                  | repeated bytes                                  | Preliminary list of transactions that have been picked as part of the block to propose.       | 2            |
    | local_last_commit    | [ExtendedCommitInfo](#extendedcommitinfo)       | Info about the last commit, obtained locally from CometBFT's data structures.                 | 3            |
    | misbehavior          | repeated [Misbehavior](#misbehavior)            | List of information about validators that misbehaved.                                         | 4            |
    | height               | int64                                           | The height of the block that will be proposed.                                                | 5            |
    | time                 | [google.protobuf.Timestamp][protobuf-timestamp] | Timestamp of the block that that will be proposed.                                            | 6            |
    | next_validators_hash | bytes                                           | Merkle root of the next validator set.                                                        | 7            |
    | proposer_address     | bytes                                           | [Address](../core/data_structures.md#address) of the validator that is creating the proposal. | 8            |

* **Response**:

    | Name | Type           | Description                                                                                 | Field Number | Deterministic |
    |------|----------------|---------------------------------------------------------------------------------------------|--------------|---------------|
    | txs  | repeated bytes | Possibly modified list of transactions that have been picked as part of the proposed block. | 2            | No            |

* **Usage**:
    * `PrepareProposalRequest`'s fields `txs`, `misbehavior`, `height`, `time`,
      `next_validators_hash`, and `proposer_address` are the same as in `ProcessProposalRequest`
      and `FinalizeBlockRequest`.
    * `PrepareProposalRequest.local_last_commit` is a set of the precommit votes for the previous
      height, including the ones that led to the decision of the previous block,
      together with their corresponding vote extensions.
    * The `height`, `time`, and `proposer_address` values match the values from the header of the
      proposed block.
    * `PrepareProposalRequest` contains a preliminary set of transactions `txs` that CometBFT
      retrieved from the mempool, called _raw proposal_. The Application can modify this
      set and return a modified set of transactions via `PrepareProposalResponse.txs` .
        * The Application _can_ modify the raw proposal: it can reorder, remove or add transactions.
          Let `tx` be a transaction in `txs` (set of transactions within `PrepareProposalRequest`):
            * If the Application considers that `tx` should not be proposed in this block, e.g.,
              there are other transactions with higher priority, then it should not include it in
              `PrepareProposalResponse.txs`. However, this will not remove `tx` from the mempool.
            * If the Application wants to add a new transaction to the proposed block, then the
              Application includes it in `PrepareProposalResponse.txs`. CometBFT will not add
              the transaction to the mempool.
        * The Application should be aware that removing and adding transactions may compromise
          _traceability_.
          > Consider the following example: the Application transforms a client-submitted
            transaction `t1` into a second transaction `t2`, i.e., the Application asks CometBFT
            to remove `t1` from the block and add `t2` to the block. If a client wants to eventually check what
            happened to `t1`, it will discover that `t1` is not in a
            committed block (assuming a _re-CheckTx_ evicted it from the mempool), getting the wrong idea that `t1` did not make it into a block. Note
            that `t2` _will be_ in a committed block, but unless the Application tracks this
            information, no component will be aware of it. Thus, if the Application wants
            traceability, it is its responsibility's to support it. For instance, the Application
            could attach to a transformed transaction a list with the hashes of the transactions it
            derives from.
    * The Application MAY configure CometBFT to include a list of transactions in `PrepareProposalRequest.txs`
      whose total size in bytes exceeds `PrepareProposalRequest.max_tx_bytes`.
      If the Application sets `ConsensusParams.Block.MaxBytes` to -1, CometBFT
      will include _all_ transactions currently in the mempool in `PrepareProposalRequest.txs`,
      which may not fit in `PrepareProposalRequest.max_tx_bytes`.
      Therefore, if the size of `PrepareProposalRequest.txs` is greater than
      `PrepareProposalRequest.max_tx_bytes`, the Application MUST remove transactions to ensure
      that the `PrepareProposalRequest.max_tx_bytes` limit is respected by those transactions
      returned in `PrepareProposalResponse.txs`.
      This is specified in [Requirement 2](./abci%2B%2B_app_requirements.md).
    * As a result of executing the prepared proposal, the Application may produce block events or transaction events.
      The Application must keep those events until a block is decided and then pass them on to CometBFT via
      `FinalizeBlockResponse`.
    * CometBFT does NOT provide any additional validity checks (such as checking for duplicate
      transactions).
    * If CometBFT fails to validate the `PrepareProposalResponse`, CometBFT will assume the
      Application is faulty and crash.
    * The implementation of `PrepareProposal` MAY be non-deterministic.


#### When does CometBFT call "PrepareProposal" ?

When a validator _p_ enters consensus round _r_, height _h_, in which _p_ is the proposer,
and _p_'s _validValue_ is `nil`:

1. CometBFT collects outstanding transactions from _p_'s mempool
    * the transactions will be collected in order of priority
    * _p_'s CometBFT creates a block header.
2. _p_'s CometBFT calls `PrepareProposal` with the newly generated block, the local
   commit of the previous height (with vote extensions), and any outstanding evidence of
   misbehavior. The call is synchronous: CometBFT's execution will block until the Application
   returns from the call.
3. The Application uses the information received (transactions, commit info, misbehavior, time) to
    (potentially) modify the proposal.
    * the Application MAY fully execute the block and produce a candidate state (immediate execution)
    * the Application can manipulate transactions:
        * leave transactions untouched
        * add new transactions (not present initially) to the proposal
        * remove transactions from the proposal (but not from the mempool thus effectively _delaying_ them) - the
          Application does not include the transaction in `PrepareProposalResponse.txs`.
        * modify transactions (e.g. aggregate them). As explained above, this compromises client traceability, unless
          it is implemented at the Application level.
        * reorder transactions - the Application reorders transactions in the list
    * the Application MAY use the vote extensions in the commit info to modify the proposal, in which case it is suggested
     that extensions be validated in the same maner as done in `VerifyVoteExtension`, since extensions of votes included
     in the commit info after the minimum of +2/3 had been reached are not verified.
4. The Application includes the transaction list (whether modified or not) in the return parameters
   (see the rules in section _Usage_), and returns from the call.
5. _p_ uses the (possibly) modified block as _p_'s proposal in round _r_, height _h_.

Note that, if _p_ has a non-`nil` _validValue_ in round _r_, height _h_,
the consensus algorithm will use it as proposal and will not call `PrepareProposal`.

### ProcessProposal

#### Parameters and Types

* **Request**:

    | Name                 | Type                                            | Description                                                                               | Field Number |
    |----------------------|-------------------------------------------------|-------------------------------------------------------------------------------------------|--------------|
    | txs                  | repeated bytes                                  | List of transactions of the proposed block.                                               | 1            |
    | proposed_last_commit | [CommitInfo](#commitinfo)                       | Info about the last commit, obtained from the information in the proposed block.          | 2            |
    | misbehavior          | repeated [Misbehavior](#misbehavior)            | List of information about validators that misbehaved.                                     | 3            |
    | hash                 | bytes                                           | The hash of the proposed block.                                                           | 4            |
    | height               | int64                                           | The height of the proposed block.                                                         | 5            |
    | time                 | [google.protobuf.Timestamp][protobuf-timestamp] | Timestamp of the proposed block.                                                          | 6            |
    | next_validators_hash | bytes                                           | Merkle root of the next validator set.                                                    | 7            |
    | proposer_address     | bytes                                           | [Address](../core/data_structures.md#address) of the validator that created the proposal. | 8            |

* **Response**:

    | Name   | Type                              | Description                                                      | Field Number | Deterministic |
    |--------|-----------------------------------|------------------------------------------------------------------|--------------|---------------|
    | status | [ProposalStatus](#proposalstatus) | `enum` that signals if the application finds the proposal valid. | 1            | Yes           |

* **Usage**:
    * Contains all information on the proposed block needed to fully execute it.
        * The Application may fully execute the block as though it was handling
         `FinalizeBlock`.
        * However, any resulting state changes must be kept as _candidate state_,
          and the Application should be ready to discard it in case another block is decided.
    * `ProcessProposal` is also called at the proposer of a round.
      Normally the call to `ProcessProposal` occurs right after the call to `PrepareProposal` and
      `ProcessProposalRequest` matches the block produced based on `PrepareProposalResponse` (i.e.,
      `PrepareProposalRequest.txs` equals `ProcessProposalRequest.txs`).
      However, no such guarantee is made since, in the presence of failures, `ProcessProposalRequest` may match
      `PrepareProposalResponse` from an earlier invocation or `ProcessProposal` may not be invoked at all.
    * The height and time values match the values from the header of the proposed block.
    * If `ProcessProposalResponse.status` is `REJECT`, consensus assumes the proposal received
      is not valid.
    * The Application MAY fully execute the block (immediate execution)
    * The implementation of `ProcessProposal` MUST be deterministic. Moreover, the value of
      `ProcessProposalResponse.status` MUST **exclusively** depend on the parameters passed in
      the `ProcessProposalRequest`, and the last committed Application state
      (see [Requirements](./abci++_app_requirements.md) section).
    * Moreover, application implementers SHOULD always set `ProcessProposalResponse.status` to `ACCEPT`,
      unless they _really_ know what the potential liveness implications of returning `REJECT` are.

#### When does CometBFT call "ProcessProposal" ?

When a node _p_ enters consensus round _r_, height _h_, in which _q_ is the proposer (possibly _p_ = _q_):

1. _p_ sets up timer `ProposeTimeout`.
2. If _p_ is the proposer, _p_ executes steps 1-5 in [PrepareProposal](#prepareproposal).
3. Upon reception of Proposal message (which contains the header) for round _r_, height _h_ from
   _q_, _p_ verifies the block header.
4. Upon reception of Proposal message, along with all the block parts, for round _r_, height _h_
   from _q_, _p_ follows the validators' algorithm to check whether it should prevote for the
   proposed block, or `nil`.
5. If the validators' consensus algorithm indicates _p_ should prevote non-nil:
    1. CometBFT calls `ProcessProposal` with the block. The call is synchronous.
    2. The Application checks/processes the proposed block, which is read-only, and returns
       `ACCEPT` or `REJECT` in the `ProcessProposalResponse.status` field.
       * The Application, depending on its needs, may call `ProcessProposal`
         * either after it has completely processed the block (immediate execution),
         * or after doing some basic checks, and process the block asynchronously. In this case the
           Application will not be able to reject the block, or force prevote/precommit `nil`
           afterwards.
         * or immediately, returning `ACCEPT`, if _p_ is not a validator
           and the Application does not want non-validating nodes to handle `ProcessProposal`
    3. If _p_ is a validator and the returned value is
         * `ACCEPT`: _p_ prevotes on this proposal for round _r_, height _h_.
         * `REJECT`: _p_ prevotes `nil`.

### ExtendVote

#### Parameters and Types

* **Request**:

    | Name                 | Type                                            | Description                                                                               | Field Number |
    |----------------------|-------------------------------------------------|-------------------------------------------------------------------------------------------|--------------|
    | hash                 | bytes                                           | The header hash of the proposed block that the vote extension is to refer to.             | 1            |
    | height               | int64                                           | Height of the proposed block (for sanity check).                                          | 2            |
    | time                 | [google.protobuf.Timestamp][protobuf-timestamp] | Timestamp of the proposed block (that the extension is to refer to).                      | 3            |
    | txs                  | repeated bytes                                  | List of transactions of the block that the extension is to refer to.                      | 4            |
    | proposed_last_commit | [CommitInfo](#commitinfo)                       | Info about the last proposed block's last commit.                                         | 5            |
    | misbehavior          | repeated [Misbehavior](#misbehavior)            | List of information about validators that misbehaved contained in the proposed block.     | 6            |
    | next_validators_hash | bytes                                           | Merkle root of the next validator set contained in the proposed block.                    | 7            |
    | proposer_address     | bytes                                           | [Address](../core/data_structures.md#address) of the validator that created the proposal. | 8            |

* **Response**:

    | Name           | Type  | Description                                           | Field Number | Deterministic |
    |----------------|-------|-------------------------------------------------------|--------------|---------------|
    | vote_extension | bytes | Information signed by by CometBFT. Can have 0 length. | 1            | No            |

* **Usage**:
    * `ExtendVoteResponse.vote_extension` is application-generated information that will be signed
      by CometBFT and attached to the Precommit message.
    * The Application may choose to use an empty vote extension (0 length).
    * The contents of `ExtendVoteRequest` correspond to the proposed block on which the consensus algorithm
      will send the Precommit message.
    * `ExtendVoteResponse.vote_extension` will only be attached to a non-`nil` Precommit message. If the consensus algorithm is to
      precommit `nil`, it will not call `ExtendVote`.
    * The Application logic that creates the extension can be non-deterministic.

#### When does CometBFT call `ExtendVote`?

When a validator _p_ is in consensus state _prevote_ of round _r_, height _h_, in which _q_ is the proposer; and _p_ has received

* the Proposal message _v_ for round _r_, height _h_, along with all the block parts, from _q_,
* `Prevote` messages from _2f + 1_ validators' voting power for round _r_, height _h_, prevoting for the same block _id(v)_,

then _p_ locks _v_  and sends a Precommit message in the following way

1. _p_ sets _lockedValue_ and _validValue_ to _v_, and sets _lockedRound_ and _validRound_ to _r_
2. _p_'s CometBFT calls `ExtendVote` with _v_ (in `ExtendVoteRequest`). The call is synchronous.
3. The Application returns an array of bytes, `ExtendVoteResponse.extension`, which is not interpreted by the consensus algorithm.
4. _p_ sets `ExtendVoteResponse.extension` as the value of the `extension` field of type
   [CanonicalVoteExtension](../core/data_structures.md#canonicalvoteextension),
   populates the other fields in [CanonicalVoteExtension](../core/data_structures.md#canonicalvoteextension),
   and signs the populated data structure.
5. _p_ constructs and signs the [CanonicalVote](../core/data_structures.md#canonicalvote) structure.
6. _p_ constructs the Precommit message (i.e. [Vote](../core/data_structures.md#vote) structure)
   using [CanonicalVoteExtension](../core/data_structures.md#canonicalvoteextension)
   and [CanonicalVote](../core/data_structures.md#canonicalvote).
7. _p_ broadcasts the Precommit message.

In the cases when _p_ is to broadcast `precommit nil` messages (either _2f+1_ `prevote nil` messages received,
or _timeoutPrevote_ triggered), _p_'s CometBFT does **not** call `ExtendVote` and will not include
a [CanonicalVoteExtension](../core/data_structures.md#canonicalvoteextension) field in the `precommit nil` message.

### VerifyVoteExtension

#### Parameters and Types

* **Request**:

    | Name              | Type  | Description                                                                               | Field Number |
    |-------------------|-------|-------------------------------------------------------------------------------------------|--------------|
    | hash              | bytes | The hash of the proposed block that the vote extension refers to.                         | 1            |
    | validator_address | bytes | [Address](../core/data_structures.md#address) of the validator that signed the extension. | 2            |
    | height            | int64 | Height of the block (for sanity check).                                                   | 3            |
    | vote_extension    | bytes | Application-specific information signed by CometBFT. Can have 0 length.                   | 4            |

* **Response**:

    | Name   | Type                          | Description                                                    | Field Number | Deterministic |
    |--------|-------------------------------|----------------------------------------------------------------|--------------|---------------|
    | status | [VerifyStatus](#verifystatus) | `enum` signaling if the application accepts the vote extension | 1            | Yes           |

* **Usage**:
    * `VerifyVoteExtensionRequest.vote_extension` can be an empty byte array. The Application's
      interpretation of it should be
      that the Application running at the process that sent the vote chose not to extend it.
      CometBFT will always call `VerifyVoteExtension`, even for 0 length vote extensions.
    * `VerifyVoteExtension` is not called for precommit votes sent by the local process.
    * `VerifyVoteExtensionRequest.hash` refers to a proposed block. There is no guarantee that
      this proposed block has previously been exposed to the Application via `ProcessProposal`.
    * If `VerifyVoteExtensionResponse.status` is `REJECT`, the consensus algorithm will reject the whole received vote.
      See the [Requirements](./abci++_app_requirements.md) section to understand the potential
      liveness implications of this.
    * The implementation of `VerifyVoteExtension` MUST be deterministic. Moreover, the value of
      `VerifyVoteExtensionResponse.status` MUST **exclusively** depend on the parameters passed in
      the `VerifyVoteExtensionRequest`, and the last committed Application state
      (see [Requirements](./abci++_app_requirements.md) section).
    * Moreover, application implementers SHOULD always set `VerifyVoteExtensionResponse.status` to `ACCEPT`,
      unless they _really_ know what the potential liveness implications of returning `REJECT` are.

#### When does CometBFT call `VerifyVoteExtension`?

When a node _p_ is in consensus round _r_, height _h_, and _p_ receives a Precommit
message for round _r_, height _h_ from validator _q_ (_q_ &ne; _p_):

1. If the Precommit message does not contain a vote extension with a valid signature, _p_
   discards the Precommit message as invalid.
   * a 0-length vote extension is valid as long as its accompanying signature is also valid.
2. Else, _p_'s CometBFT calls `VerifyVoteExtension`.
3. The Application returns `ACCEPT` or `REJECT` via `VerifyVoteExtensionResponse.status`.
4. If the Application returns
   * `ACCEPT`, _p_ will keep the received vote, together with its corresponding
     vote extension in its internal data structures. It will be used to populate the [ExtendedCommitInfo](#extendedcommitinfo)
     structure in calls to `PrepareProposal`, in rounds of height _h + 1_ where _p_ is the proposer.
   * `REJECT`, _p_ will deem the Precommit message invalid and discard it.

When a node _p_ is in consensus round _0_, height _h_, and _p_ receives a Precommit
message for CommitRound _r_, height _h-1_ from validator _q_ (_q_ &ne; _p_), _p_
MAY add the Precommit message and associated extension to [ExtendedCommitInfo](#extendedcommitinfo)
without calling `VerifyVoteExtension` to verify it.


### FinalizeBlock

#### Parameters and Types

* **Request**:

    | Name                 | Type                                            | Description                                                                               | Field Number |
    |----------------------|-------------------------------------------------|-------------------------------------------------------------------------------------------|--------------|
    | txs                  | repeated bytes                                  | List of transactions committed as part of the block.                                      | 1            |
    | decided_last_commit  | [CommitInfo](#commitinfo)                       | Info about the last commit, obtained from the block that was just decided.                | 2            |
    | misbehavior          | repeated [Misbehavior](#misbehavior)            | List of information about validators that misbehaved.                                     | 3            |
    | hash                 | bytes                                           | The block's hash.                                                                         | 4            |
    | height               | int64                                           | The height of the finalized block.                                                        | 5            |
    | time                 | [google.protobuf.Timestamp][protobuf-timestamp] | Timestamp of the finalized block.                                                         | 6            |
    | next_validators_hash | bytes                                           | Merkle root of the next validator set.                                                    | 7            |
    | proposer_address     | bytes                                           | [Address](../core/data_structures.md#address) of the validator that created the proposal. | 8            |
    | syncing_to_height    | int64                                           | If the node is syncing/replaying blocks then syncing_to_height == target height. If not, syncing_to_height == height.    | 9            |  

* **Response**:

    | Name                    | Type                                              | Description                                                                      | Field Number | Deterministic |
    |-------------------------|---------------------------------------------------|----------------------------------------------------------------------------------|--------------|---------------|
    | events                  | repeated [Event](abci++_basic_concepts.md#events) | Type & Key-Value events for indexing                                             | 1            | No            |
    | tx_results              | repeated [ExecTxResult](#exectxresult)            | List of structures containing the data resulting from executing the transactions | 2            | Yes           |
    | validator_updates       | repeated [ValidatorUpdate](#validatorupdate)      | Changes to validator set (set voting power to 0 to remove).                      | 3            | Yes           |
    | consensus_param_updates | [ConsensusParams](#consensusparams)               | Changes to gas, size, and other consensus-related parameters.                    | 4            | Yes           |
    | app_hash                | bytes                                             | The Merkle root hash of the application state.                                   | 5            | Yes           |

* **Usage**:
    * Contains the fields of the newly decided block.
    * This method is equivalent to the call sequence `BeginBlock`, [`DeliverTx`],
      and `EndBlock` in ABCI 1.0.
    * The height and time values match the values from the header of the proposed block.
    * The Application can use `FinalizeBlockRequest.decided_last_commit` and `FinalizeBlockRequest.misbehavior`
      to determine rewards and punishments for the validators.
    * The Application executes the transactions in `FinalizeBlockRequest.txs` deterministically,
      according to the rules set up by the Application, before returning control to CometBFT.
      Alternatively, it can apply the candidate state corresponding to the same block previously
      executed via `PrepareProposal` or `ProcessProposal`.
    * `FinalizeBlockResponse.tx_results[i].Code == 0` only if the _i_-th transaction is fully valid.
    * The Application must provide values for `FinalizeBlockResponse.app_hash`,
      `FinalizeBlockResponse.tx_results`, `FinalizeBlockResponse.validator_updates`, and
      `FinalizeBlockResponse.consensus_param_updates` as a result of executing the block.
        * The values for `FinalizeBlockResponse.validator_updates`, or
          `FinalizeBlockResponse.consensus_param_updates` may be empty. In this case, CometBFT will keep
          the current values.
        * `FinalizeBlockResponse.validator_updates`, triggered by block `H`, affect validation
          for blocks `H+1`, `H+2`, and `H+3`. Heights following a validator update are affected in the following way:
            * Height `H+1`: `NextValidatorsHash` includes the new `validator_updates` value.
            * Height `H+2`: The validator set change takes effect and `ValidatorsHash` is updated.
            * Height `H+3`: `*_last_commit` fields in `PrepareProposal`, `ProcessProposal`, and
              `FinalizeBlock` now include the altered validator set.
        * `FinalizeBlockResponse.consensus_param_updates` returned for block `H` apply to the consensus
          params for block `H+1`. For more information on the consensus parameters,
          see the [consensus parameters](./abci%2B%2B_app_requirements.md#consensus-parameters)
          section.
    * `FinalizeBlockResponse.app_hash` contains an (optional) Merkle root hash of the application state.
    * `FinalizeBlockResponse.app_hash` is included as the `Header.AppHash` in the next block.
        * `FinalizeBlockResponse.app_hash` may also be empty or hard-coded, but MUST be
          **deterministic** - it must not be a function of anything that did not come from the parameters
          of `FinalizeBlockRequest` and the previous committed state.
    * Later calls to `Query` can return proofs about the application state anchored
      in this Merkle root hash.
    * The implementation of `FinalizeBlock` MUST be deterministic, since it is
      making the Application's state evolve in the context of state machine replication.
    * Currently, CometBFT will fill up all fields in `FinalizeBlockRequest`, even if they were
      already passed on to the Application via `PrepareProposalRequest` or `ProcessProposalRequest`.
    * When calling `FinalizeBlock` with a block, the consensus algorithm run by CometBFT guarantees
      that at least one non-byzantine validator has run `ProcessProposal` on that block.

#### When does CometBFT call `FinalizeBlock`?

When a node _p_ is in consensus height _h_, and _p_ receives

* the Proposal message with block _v_ for a round _r_, along with all its block parts, from _q_,
  which is the proposer of round _r_, height _h_,
* `Precommit` messages from _2f + 1_ validators' voting power for round _r_, height _h_,
  precommitting the same block _id(v)_,

then _p_ decides block _v_ and finalizes consensus for height _h_ in the following way

1. _p_ persists _v_ as the decision for height _h_.
2. _p_'s CometBFT calls `FinalizeBlock` with _v_'s data. The call is synchronous.
3. _p_'s Application executes block _v_.
4. _p_'s Application calculates and returns the _AppHash_, along with a list containing
   the outputs of each of the transactions executed.
5. _p_'s CometBFT hashes all the transaction outputs and stores it in _ResultHash_.
6. _p_'s CometBFT persists the transaction outputs, _AppHash_, and _ResultsHash_.
7. _p_'s CometBFT locks the mempool &mdash; no calls to `CheckTx` on new transactions.
8. _p_'s CometBFT calls `Commit` to instruct the Application to persist its state.
9. _p_'s CometBFT, optionally, re-checks all outstanding transactions in the mempool
   against the newly persisted Application state.
10. _p_'s CometBFT unlocks the mempool &mdash; newly received transactions can now be checked.
11. _p_ starts consensus for height _h+1_, round 0

## Data Types (exist before ABCI 2.0)

Most of the data structures used in ABCI are shared [common data structures](../core/data_structures.md). In certain cases, ABCI uses different data structures which are documented here:

### Validator

* **Fields**:

    | Name    | Type  | Description                                                | Field Number |
    |---------|-------|------------------------------------------------------------|--------------|
    | address | bytes | [Address](../core/data_structures.md#address) of validator | 1            |
    | power   | int64 | Voting power of the validator                              | 3            |

* **Usage**:
    * Validator identified by address
    * Used as part of VoteInfo within `CommitInfo` (used in `ProcessProposal` and `FinalizeBlock`),
      and `ExtendedCommitInfo` (used in `PrepareProposal`).
    * Does not include PubKey to avoid sending potentially large quantum pubkeys
    over the ABCI

### ValidatorUpdate

* **Fields**:

    | Name          | Type                                             | Description                                         | Field Number | Deterministic |
    |---------------|--------------------------------------------------|-----------------------------------------------------|--------------|---------------|
    | power         | int64                                            | Voting power                                        | 2            | Yes           |
    | pub_key_type  | string                                           | Public key's type (e.g. "tendermint/PubKeyEd25519") | 3            | Yes           |
    | pub_key_bytes | bytes                                            | Public key's bytes                                  | 4            | Yes           |

* **Usage**:
    * Validator identified by PubKeyType and PubKeyBytes
    * Used to tell CometBFT to update the validator set

### Misbehavior

* **Fields**:

    | Name               | Type                                            | Description                                                  | Field Number |
    |--------------------|-------------------------------------------------|--------------------------------------------------------------|--------------|
    | type               | [MisbehaviorType](#misbehaviortype)             | Type of the misbehavior. An enum of possible misbehaviors.   | 1            |
    | validator          | [Validator](#validator)                         | The offending validator                                      | 2            |
    | height             | int64                                           | Height when the offense occurred                             | 3            |
    | time               | [google.protobuf.Timestamp][protobuf-timestamp] | Timestamp of the block that was committed at height `height` | 4            |
    | total_voting_power | int64                                           | Total voting power of the validator set at height `height`   | 5            |

#### MisbehaviorType

* **Fields**

    MisbehaviorType is an enum with the listed fields:

    | Name                | Field Number |
    |---------------------|--------------|
    | UNKNOWN             | 0            |
    | DUPLICATE_VOTE      | 1            |
    | LIGHT_CLIENT_ATTACK | 2            |

### ConsensusParams

* **Fields**:

    | Name      | Type                                                          | Description                                                                  | Field Number | Deterministic |
    | --------- | ------------------------------------------------------------- | ---------------------------------------------------------------------------- | ------------ | ------------- |
    | block     | [BlockParams](../core/data_structures.md#blockparams)         | Parameters limiting the size of a block and time between consecutive blocks. | 1            | Yes           |
    | evidence  | [EvidenceParams](../core/data_structures.md#evidenceparams)   | Parameters limiting the validity of evidence of byzantine behaviour.         | 2            | Yes           |
    | validator | [ValidatorParams](../core/data_structures.md#validatorparams) | Parameters limiting the types of public keys validators can use.             | 3            | Yes           |
    | version   | [VersionsParams](../core/data_structures.md#versionparams)    | The ABCI application version.                                                | 4            | Yes           |
    | abci      | [ABCIParams](../core/data_structures.md#abciparams)           | ABCI-related parameters.                                                     | 5            | Yes           |
    | synchrony | [SynchronyParams](../core/data_structures.md#synchronyparams) | Parameters determining the validity bounds of a proposal timestamp.          | 6            | Yes           |

### ProofOps

* **Fields**:

    | Name | Type                         | Description                                                                                                                                                                                                                  | Field Number | Deterministic |
    |------|------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------|---------------|
    | ops  | repeated [ProofOp](#proofop) | List of chained Merkle proofs, of possibly different types. The Merkle root of one op is the value being proven in the next op. The Merkle root of the final op should equal the ultimate root hash being verified against.. | 1            | N/A           |

### ProofOp

* **Fields**:

    | Name | Type   | Description                                    | Field Number | Deterministic |
    |------|--------|------------------------------------------------|--------------|---------------|
    | type | string | Type of Merkle proof and how it's encoded.     | 1            | N/A           |
    | key  | bytes  | Key in the Merkle tree that this proof is for. | 2            | N/A           |
    | data | bytes  | Encoded Merkle proof for the key.              | 3            | N/A           |

### Snapshot

* **Fields**:

    | Name     | Type   | Description                                                                                                                                                                     | Field Number | Deterministic |
    |----------|--------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------|---------------|
    | height   | uint64 | The height at which the snapshot was taken (after commit).                                                                                                                      | 1            | N/A           |
    | format   | uint32 | An application-specific snapshot format, allowing applications to version their snapshot data format and make backwards-incompatible changes. CometBFT does not interpret this. | 2            | N/A           |
    | chunks   | uint32 | The number of chunks in the snapshot. Must be at least 1 (even if empty).                                                                                                       | 3            | N/A           |
    | hash     | bytes  | An arbitrary snapshot hash. Must be equal only for identical snapshots across nodes. CometBFT does not interpret the hash, it only compares them.                               | 4            | N/A           |
    | metadata | bytes  | Arbitrary application metadata, for example chunk hashes or other verification data.                                                                                            | 5            | N/A           |

* **Usage**:
    * Used for state sync snapshots, see the [state sync section](../p2p/legacy-docs/messages/state-sync.md) for details.
    * A snapshot is considered identical across nodes only if _all_ fields are equal (including
    `Metadata`). Chunks may be retrieved from all nodes that have the same snapshot.
    * When sent across the network, a snapshot message can be at most 4 MB.

## Data types introduced or modified in ABCI 2.0

### VoteInfo

* **Fields**:

    | Name              | Type                    | Description                                                   | Field Number |
    |-------------------|-------------------------|---------------------------------------------------------------|--------------|
    | validator         | [Validator](#validator) | The validator that sent the vote.                             | 1            |
    | signed_last_block | bool                    | Indicates whether or not the validator signed the last block. | 2            |

* **Usage**:
    * Indicates whether a validator signed the last block, allowing for rewards based on validator availability.
    * This information is typically extracted from a proposed or decided block.

### ExtendedVoteInfo

* **Fields**:

    | Name              | Type                    | Description                                                                  | Field Number |
    |-------------------|-------------------------|------------------------------------------------------------------------------|--------------|
    | validator         | [Validator](#validator) | The validator that sent the vote.                                            | 1            |
    | signed_last_block | bool                    | Indicates whether or not the validator signed the last block.                | 2            |
    | vote_extension    | bytes                   | Non-deterministic extension provided by the sending validator's Application. | 3            |

* **Usage**:
    * Indicates whether a validator signed the last block, allowing for rewards based on validator availability.
    * This information is extracted from CometBFT's data structures in the local process.
    * `vote_extension` contains the sending validator's vote extension, which is signed by CometBFT. It can be empty

### CommitInfo

* **Fields**:

    | Name  | Type                           | Description                                                                                  | Field Number |
    |-------|--------------------------------|----------------------------------------------------------------------------------------------|--------------|
    | round | int32                          | Commit round. Reflects the round at which the block proposer decided in the previous height. | 1            |
    | votes | repeated [VoteInfo](#voteinfo) | List of validators' addresses in the last validator set with their voting information.       | 2            |

* **Notes**
  * The `VoteInfo` in `votes` are ordered by the voting power of the validators (descending order, highest to lowest voting power).
  * CometBFT guarantees the `votes` ordering through its logic to update the validator set in which, in the end, the  validators are sorted (descending) by their voting power.
  * The ordering is also persisted when a validator set is saved in the store.
  * The validator set is loaded from the store when building the `CommitInfo`, ensuring order is maintained from the persisted validator set.

### ExtendedCommitInfo

* **Fields**:

    | Name  | Type                                           | Description                                                                                                       | Field Number |
    |-------|------------------------------------------------|-------------------------------------------------------------------------------------------------------------------|--------------|
    | round | int32                                          | Commit round. Reflects the round at which the block proposer decided in the previous height.                      | 1            |
    | votes | repeated [ExtendedVoteInfo](#extendedvoteinfo) | List of validators' addresses in the last validator set with their voting information, including vote extensions. | 2            |

* **Notes**
    * The `ExtendedVoteInfo` in `votes` are ordered by the voting power of the validators (descending order, highest to lowest voting power).
    * CometBFT guarantees the `votes` ordering through its logic to update the validator set in which, in the end, the validators are sorted (descending) by their voting power.
    * The ordering is also persisted when a validator set is saved in the store.
    * The validator set is loaded from the store when building the `ExtendedCommitInfo`, ensuring order is maintained from the persisted validator set.

### ExecTxResult

* **Fields**:

    | Name       | Type                                              | Description                                                          | Field Number | Deterministic |
    |------------|---------------------------------------------------|----------------------------------------------------------------------|--------------|---------------|
    | code       | uint32                                            | Response code.                                                       | 1            | Yes           |
    | data       | bytes                                             | Result bytes, if any.                                                | 2            | Yes           |
    | log        | string                                            | The output of the application's logger.                              | 3            | No            |
    | info       | string                                            | Additional information.                                              | 4            | No            |
    | gas_wanted | int64                                             | Amount of gas requested for transaction.                             | 5            | Yes           |
    | gas_used   | int64                                             | Amount of gas consumed by transaction.                               | 6            | Yes           |
    | events     | repeated [Event](abci++_basic_concepts.md#events) | Type & Key-Value events for indexing transactions (e.g. by account). | 7            | No            |
    | codespace  | string                                            | Namespace for the `code`.                                            | 8            | Yes           |

### ProposalStatus

```proto
enum ProposalStatus {
  UNKNOWN = 0; // Unknown status. Returning this from the application is always an error.
  ACCEPT  = 1; // Status that signals that the application finds the proposal valid.
  REJECT  = 2; // Status that signals that the application finds the proposal invalid.
}
```

* **Usage**:
    * Used within the [ProcessProposal](#processproposal) response.
        * If `Status` is `UNKNOWN`, a problem happened in the Application. CometBFT will assume the application is faulty and crash.
        * If `Status` is `ACCEPT`, the consensus algorithm accepts the proposal and will issue a Prevote message for it.
        * If `Status` is `REJECT`, the consensus algorithm rejects the proposal and will issue a Prevote for `nil` instead.


### VerifyStatus

```proto
enum VerifyStatus {
  UNKNOWN = 0; // Unknown status. Returning this from the application is always an error.
  ACCEPT  = 1; // Status that signals that the application finds the vote extension valid.
  REJECT  = 2; // Status that signals that the application finds the vote extension invalid.
}
```

* **Usage**:
    * Used within the [VerifyVoteExtension](#verifyvoteextension) response.
        * If `Status` is `UNKNOWN`, a problem happened in the Application. CometBFT will assume the application is faulty and crash.
        * If `Status` is `ACCEPT`, the consensus algorithm will accept the vote as valid.
        * If `Status` is `REJECT`, the consensus algorithm will reject the vote as invalid.

[protobuf-timestamp]: https://developers.google.com/protocol-buffers/docs/reference/google.protobuf#google.protobuf.Timestamp
