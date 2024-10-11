# ADR 053: State Sync Prototype

State sync is now [merged](https://github.com/tendermint/tendermint/pull/4705). Up-to-date ABCI documentation is [available](https://github.com/tendermint/spec/pull/90), refer to it rather than this ADR for details.

This ADR outlines the plan for an initial state sync prototype, and is subject to change as we gain feedback and experience. It builds on discussions and findings in [ADR-042](adr-042-state-sync.md), see that for background information.

## Changelog

* 2020-01-28: Initial draft (Erik Grinaker)

* 2020-02-18: Updates after initial prototype (Erik Grinaker)
    * ABCI: added missing `reason` fields.
    * ABCI: used 32-bit 1-based chunk indexes (was 64-bit 0-based).
    * ABCI: moved `RequestApplySnapshotChunk.chain_hash` to `RequestOfferSnapshot.app_hash`.
    * Gaia: snapshots must include node versions as well, both for inner and leaf nodes.
    * Added experimental prototype info.
    * Added open questions and implementation plan.

* 2020-03-29: Strengthened and simplified ABCI interface (Erik Grinaker)
    * ABCI: replaced `chunks` with `chunk_hashes` in `Snapshot`.
    * ABCI: removed `SnapshotChunk` message.
    * ABCI: renamed `GetSnapshotChunk` to `LoadSnapshotChunk`.
    * ABCI: chunks are now exchanged simply as `bytes`.
    * ABCI: chunks are now 0-indexed, for parity with `chunk_hashes` array.
    * Reduced maximum chunk size to 16 MB, and increased snapshot message size to 4 MB.

* 2020-04-29: Update with final released ABCI interface (Erik Grinaker)

## Context

State sync will allow a new node to receive a snapshot of the application state without downloading blocks or going through consensus. This bootstraps the node significantly faster than the current fast sync system, which replays all historical blocks.

Background discussions and justifications are detailed in [ADR-042](adr-042-state-sync.md). Its recommendations can be summarized as:

* The application periodically takes full state snapshots (i.e. eager snapshots).

* The application splits snapshots into smaller chunks that can be individually verified against a chain app hash.

* Tendermint uses the light client to obtain a trusted chain app hash for verification.

* Tendermint discovers and downloads snapshot chunks in parallel from multiple peers, and passes them to the application via ABCI to be applied and verified against the chain app hash.

* Historical blocks are not backfilled, so state synced nodes will have a truncated block history.

## Tendermint Proposal

This describes the snapshot/restore process seen from Tendermint. The interface is kept as small and general as possible to give applications maximum flexibility.

### Snapshot Data Structure

A node can have multiple snapshots taken at various heights. Snapshots can be taken in different application-specified formats (e.g. MessagePack as format `1` and Protobuf as format `2`, or similarly for schema versioning). Each snapshot consists of multiple chunks containing the actual state data, for parallel downloads and reduced memory usage.

```proto
message Snapshot {
  uint64 height   = 1;  // The height at which the snapshot was taken
  uint32 format   = 2;  // The application-specific snapshot format
  uint32 chunks   = 3;  // Number of chunks in the snapshot
  bytes  hash     = 4;  // Arbitrary snapshot hash - should be equal only for identical snapshots
  bytes  metadata = 5;  // Arbitrary application metadata
}
```

Chunks are exchanged simply as `bytes`, and cannot be larger than 16 MB. `Snapshot` messages should be less than 4 MB.

### ABCI Interface

```proto
// Lists available snapshots
message RequestListSnapshots {}

message ResponseListSnapshots {
  repeated Snapshot snapshots = 1;
}

// Offers a snapshot to the application
message RequestOfferSnapshot {
  Snapshot snapshot = 1;  // snapshot offered by peers
  bytes    app_hash = 2;  // light client-verified app hash for snapshot height
 }

message ResponseOfferSnapshot {
  Result result = 1;

  enum Result {
    accept        = 0;  // Snapshot accepted, apply chunks
    abort         = 1;  // Abort all snapshot restoration
    reject        = 2;  // Reject this specific snapshot, and try a different one
    reject_format = 3;  // Reject all snapshots of this format, and try a different one
    reject_sender = 4;  // Reject all snapshots from the sender(s), and try a different one
  }
}

// Loads a snapshot chunk
message RequestLoadSnapshotChunk {
  uint64 height = 1;
  uint32 format = 2;
  uint32 chunk  = 3; // Zero-indexed
}

message ResponseLoadSnapshotChunk {
  bytes chunk = 1;
}

// Applies a snapshot chunk
message RequestApplySnapshotChunk {
  uint32 index  = 1;
  bytes  chunk  = 2;
  string sender = 3;
 }

message ResponseApplySnapshotChunk {
  Result          result         = 1;
  repeated uint32 refetch_chunks = 2;  // Chunks to refetch and reapply (regardless of result)
  repeated string reject_senders = 3;  // Chunk senders to reject and ban (regardless of result)

  enum Result {
    accept          = 0;  // Chunk successfully accepted
    abort           = 1;  // Abort all snapshot restoration
    retry           = 2;  // Retry chunk, combine with refetch and reject as appropriate
    retry_snapshot  = 3;  // Retry snapshot, combine with refetch and reject as appropriate
    reject_snapshot = 4;  // Reject this snapshot, try a different one but keep sender rejections
  }
}
```

### Taking Snapshots

Tendermint is not aware of the snapshotting process at all, it is entirely an application concern. The following guarantees must be provided:

* **Periodic:** snapshots must be taken periodically, not on-demand, for faster restores, lower load, and less DoS risk.

* **Deterministic:** snapshots must be deterministic, and identical across all nodes - typically by taking a snapshot at given height intervals.

* **Consistent:** snapshots must be consistent, i.e. not affected by concurrent writes - typically by using a data store that supports versioning and/or snapshot isolation.

* **Asynchronous:** snapshots must be asynchronous, i.e. not halt block processing and state transitions.

* **Chunked:** snapshots must be split into chunks of reasonable size (on the order of megabytes), and each chunk must be verifiable against the chain app hash.

* **Garbage collected:** snapshots must be garbage collected periodically.

### Restoring Snapshots

Nodes should have options for enabling state sync and/or fast sync, and be provided a trusted header hash for the light client.

When starting an empty node with state sync and fast sync enabled, snapshots are restored as follows:

1. The node checks that it is empty, i.e. that it has no state nor blocks.

2. The node contacts the given seeds to discover peers.

3. The node contacts a set of full nodes, and verifies the trusted block header using the given hash via the light client.

4. The node requests available snapshots via P2P from peers, via `RequestListSnapshots`. Peers will return the 10 most recent snapshots, one message per snapshot.

5. The node aggregates snapshots from multiple peers, ordered by height and format (in reverse). If there are mismatches between different snapshots, the one hosted by the largest amount of peers is chosen. The node iterates over all snapshots in reverse order by height and format until it finds one that satisfies all of the following conditions:

    * The snapshot height's block is considered trustworthy by the light client (i.e. snapshot height is greater than trusted header and within unbonding period of the latest trustworthy block).

    * The snapshot's height or format hasn't been explicitly rejected by an earlier `RequestOfferSnapshot`.

    * The application accepts the `RequestOfferSnapshot` call.

6. The node downloads chunks in parallel from multiple peers, via `RequestLoadSnapshotChunk`. Chunk messages cannot exceed 16 MB.

7. The node passes chunks sequentially to the app via `RequestApplySnapshotChunk`.

8. Once all chunks have been applied, the node compares the app hash to the chain app hash, and if they do not match it either errors or discards the state and starts over.

9. The node switches to fast sync to catch up blocks that were committed while restoring the snapshot.

10. The node switches to normal consensus mode.

## Gaia Proposal

This describes the snapshot process seen from Gaia, using format version `1`. The serialization format is unspecified, but likely to be compressed Amino or Protobuf.

### Snapshot Metadata

In the initial version there is no snapshot metadata, so it is set to an empty byte buffer.

Once all chunks have been successfully built, snapshot metadata should be stored in a database and served via `RequestListSnapshots`.

### Snapshot Chunk Format

The Gaia data structure consists of a set of named IAVL trees. A root hash is constructed by taking the root hashes of each of the IAVL trees, then constructing a Merkle tree of the sorted name/hash map.

IAVL trees are versioned, but a snapshot only contains the version relevant for the snapshot height. All historical versions are ignored.

IAVL trees are insertion-order dependent, so key/value pairs must be set in an appropriate insertion order to produce the same tree branching structure. This insertion order can be found by doing a breadth-first scan of all nodes (including inner nodes) and collecting unique keys in order. However, the node hash also depends on the node's version, so snapshots must contain the inner nodes' version numbers as well.

For the initial prototype, each chunk consists of a complete dump of all node data for all nodes in an entire IAVL tree. Thus the number of chunks equals the number of persistent stores in Gaia. No incremental verification of chunks is done, only a final app hash comparison at the end of the snapshot restoration.

For a production version, it should be sufficient to store key/value/version for all nodes (leaf and inner) in insertion order, chunked in some appropriate way. If per-chunk verification is required, the chunk must also contain enough information to reconstruct the Merkle proofs all the way up to the root of the multistore, e.g. by storing a complete subtree's key/value/version data plus Merkle hashes of all other branches up to the multistore root. The exact approach will depend on tradeoffs between size, time, and verification. IAVL RangeProofs are not recommended, since these include redundant data such as proofs for intermediate and leaf nodes that can be derived from the above data.

Chunks should be built greedily by collecting node data up to some size limit (e.g. 10 MB) and serializing it. Chunk data is stored in the file system as `snapshots/<height>/<format>/<chunk>`, and a SHA-256 checksum is stored along with the snapshot metadata.

### Snapshot Scheduling

Snapshots should be taken at some configurable height interval, e.g. every 1000 blocks. All nodes should preferably have the same snapshot schedule, such that all nodes can serve chunks for a given snapshot.

Taking consistent snapshots of IAVL trees is greatly simplified by them being versioned: simply snapshot the version that corresponds to the snapshot height, while concurrent writes create new versions. IAVL pruning must not prune a version that is being snapshotted.

Snapshots must also be garbage collected after some configurable time, e.g. by keeping the latest `n` snapshots.

## Resolved Questions

* Is it OK for state-synced nodes to not have historical blocks nor historical IAVL versions?

    > Yes, this is as intended. Maybe backfill blocks later.

* Do we need incremental chunk verification for first version?

    > No, we'll start simple. Can add chunk verification via a new snapshot format without any breaking changes in Tendermint. For adversarial conditions, maybe consider support for whitelisting peers to download chunks from.

* Should the snapshot ABCI interface be a separate optional ABCI service, or mandatory?

    > Mandatory, to keep things simple for now. It will therefore be a breaking change and push the release. For apps using the Cosmos SDK, we can provide a default implementation that does not serve snapshots and errors when trying to apply them.

* How can we make sure `ListSnapshots` data is valid? An adversary can provide fake/invalid snapshots to DoS peers.

    > For now, just pick snapshots that are available on a large number of peers. Maybe support whitelisting. We may consider e.g. placing snapshot manifests on the blockchain later.

* Should we punish nodes that provide invalid snapshots? How?

    > No, these are full nodes not validators, so we can't punish them. Just disconnect from them and ignore them.

* Should we call these snapshots? The SDK already uses the term "snapshot" for `PruningOptions.SnapshotEvery`, and state sync will introduce additional SDK options for snapshot scheduling and pruning that are not related to IAVL snapshotting or pruning.

    > Yes. Hopefully these concepts are distinct enough that we can refer to state sync snapshots and IAVL snapshots without too much confusion.

* Should we store snapshot and chunk metadata in a database? Can we use the database for chunks?

    > As a first approach, store metadata in a database and chunks in the filesystem.

* Should a snapshot at height H be taken before or after the block at H is processed? E.g. RPC `/commit` returns app_hash after _previous_ height, i.e. _before_  current height.

    > After commit.

* Do we need to support all versions of blockchain reactor (i.e. fast sync)?

    > We should remove the v1 reactor completely once v2 has stabilized.

* Should `ListSnapshots` be a streaming API instead of a request/response API?

    > No, just use a max message size.

## Status

Implemented

## References

* [ADR-042](adr-042-state-sync.md) and its references
