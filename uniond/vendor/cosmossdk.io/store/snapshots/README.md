# State Sync Snapshotting

The `snapshots` package implements automatic support for CometBFT state sync
in Cosmos SDK-based applications. State sync allows a new node joining a network
to simply fetch a recent snapshot of the application state instead of fetching
and applying all historical blocks. This can reduce the time needed to join the
network by several orders of magnitude (e.g. weeks to minutes), but the node
will not contain historical data from previous heights.

This document describes the Cosmos SDK implementation of the ABCI state sync
interface, for more information on CometBFT state sync in general see:

* [CometBFT State Sync for Developers](https://medium.com/cometbft/cometbft-core-state-sync-for-developers-70a96ba3ee35)
* [CometBFT State Sync](https://docs.cometbft.com/v0.38/core/state-sync)
* [ABCI State Sync Spec](https://docs.cometbft.com/v0.38/spec/p2p/legacy-docs/messages/state-sync)
* [ABCI State Sync Method/Type Reference](https://docs.cometbft.com/v0.38/spec/p2p/legacy-docs/messages/state-sync)

## Overview

For an overview of how Cosmos SDK state sync is set up and configured by
developers and end-users, see the
[Cosmos SDK State Sync Guide](https://blog.cosmos.network/cosmos-sdk-state-sync-guide-99e4cf43be2f).

Briefly, the Cosmos SDK takes state snapshots at regular height intervals given
by `state-sync.snapshot-interval` and stores them as binary files in the
filesystem under `<node_home>/data/snapshots/`, with metadata in a LevelDB database
`<node_home>/data/snapshots/metadata.db`. The number of recent snapshots to keep are given by
`state-sync.snapshot-keep-recent`.

Snapshots are taken asynchronously, i.e. new blocks will be applied concurrently
with snapshots being taken. This is possible because IAVL supports querying
immutable historical heights. However, this requires heights that are multiples of `state-sync.snapshot-interval`
to be kept until after the snapshot is complete. It is done to prevent a height from being removed
while it is being snapshotted.

When a remote node is state syncing, CometBFT calls the ABCI method
`ListSnapshots` to list available local snapshots and `LoadSnapshotChunk` to
load a binary snapshot chunk. When the local node is being state synced,
CometBFT calls `OfferSnapshot` to offer a discovered remote snapshot to the
local application and `ApplySnapshotChunk` to apply a binary snapshot chunk to
the local application. See the resources linked above for more details on these
methods and how CometBFT performs state sync.

The Cosmos SDK does not currently do any incremental verification of snapshots
during restoration, i.e. only after the entire snapshot has been restored will
CometBFT compare the app hash against the trusted hash from the chain. Cosmos
SDK snapshots and chunks do contain hashes as checksums to guard against IO
corruption and non-determinism, but these are not tied to the chain state and
can be trivially forged by an adversary. This was considered out of scope for
the initial implementation, but can be added later without changes to the
ABCI state sync protocol.

## Relationship to Pruning

Snapshot settings are optional. However, if set, they have an effect on how pruning is done by
persisting the heights that are multiples of `state-sync.snapshot-interval` until after the snapshot is complete.

If pruning is enabled (not `pruning = "nothing"`), we avoid pruning heights that are multiples of
`state-sync.snapshot-interval` in the regular logic determined by the
pruning settings and applied after every `Commit()`. This is done to prevent a
height from being removed before a snapshot is complete. Therefore, we keep
such heights until after a snapshot is done. At this point, the height is sent to
the `pruning.Manager` to be pruned according to the pruning settings after the next `Commit()`.

To illustrate, assume that we are currently at height 960 with `pruning-keep-recent = 50`,
`pruning-interval = 10`, and `state-sync.snapshot-interval = 100`. Let's assume that
the snapshot that was triggered at height `900` **just finishes**. Then, we can prune height
`900` right away (that is, when we call `Commit()` at height 960 because 900 is less than `960 - 50 = 910`).

Let's now assume that all conditions stay the same but the snapshot at height 900 is **not complete yet**.
Then, we cannot prune it to avoid deleting a height that is still being snapshotted. Therefore, we keep track
of this height until the snapshot is complete. The height 900 will be pruned at the first height h that satisfied the following conditions:

* the snapshot is complete
* h is a multiple of `pruning-interval`
* snapshot height is less than h - `pruning-keep-recent`

Note that in both examples, if we let current height = C, and previous height P = C - 1, then for every height h that is:

P - `pruning-keep-recent` - `pruning-interval` <= h <= P - `pruning-keep-recent`

we can prune height h. In our first example, all heights 899 - 909 fall in this range and are pruned at height 960 as long as
h is not a snapshot height (E.g. 900).

That is, we always use current height to determine at which height to prune (960) while we use previous
to determine which heights are to be pruned (959 - 50 - 10 = 899-909 = 959 - 50).

## Configuration

* `state-sync.snapshot-interval`
  * the interval at which to take snapshots.
  * the value of 0 disables snapshots.
  * if pruning is enabled, it is done after a snapshot is complete for the heights that are multiples of this interval.

* `state-sync.snapshot-keep-recent`:
  * the number of recent snapshots to keep.
  * 0 means keep all.

## Snapshot Metadata

The ABCI Protobuf type for a snapshot is listed below (refer to the ABCI spec
for field details):

```protobuf
message Snapshot {
  uint64 height   = 1;  // The height at which the snapshot was taken
  uint32 format   = 2;  // The application-specific snapshot format
  uint32 chunks   = 3;  // Number of chunks in the snapshot
  bytes  hash     = 4;  // Arbitrary snapshot hash, equal only if identical
  bytes  metadata = 5;  // Arbitrary application metadata
}
```

Because the `metadata` field is application-specific, the Cosmos SDK uses a
similar type `cosmos.base.snapshots.v1beta1.Snapshot` with its own metadata
representation:

```protobuf
// Snapshot contains CometBFT state sync snapshot info.
message Snapshot {
  uint64   height   = 1;
  uint32   format   = 2;
  uint32   chunks   = 3;
  bytes    hash     = 4;
  Metadata metadata = 5 [(gogoproto.nullable) = false];
}

// Metadata contains SDK-specific snapshot metadata.
message Metadata {
  repeated bytes chunk_hashes = 1; // SHA-256 chunk hashes
}
```

The `format` is currently `1`, defined in `snapshots.types.CurrentFormat`. This
must be increased whenever the binary snapshot format changes, and it may be
useful to support past formats in newer versions.

The `hash` is a SHA-256 hash of the entire binary snapshot, used to guard
against IO corruption and non-determinism across nodes. Note that this is not
tied to the chain state, and can be trivially forged (but CometBFT will always
compare the final app hash against the chain app hash). Similarly, the
`chunk_hashes` are SHA-256 checksums of each binary chunk.

The `metadata` field is Protobuf-serialized before it is placed into the ABCI
snapshot.

## Snapshot Format

The current version `1` snapshot format is a zlib-compressed, length-prefixed
Protobuf stream of `cosmos.base.store.v1beta1.SnapshotItem` messages, split into
chunks at exact 10 MB byte boundaries.

```protobuf
// SnapshotItem is an item contained in a rootmulti.Store snapshot.
message SnapshotItem {
  // item is the specific type of snapshot item.
  oneof item {
    SnapshotStoreItem store = 1;
    SnapshotIAVLItem  iavl  = 2 [(gogoproto.customname) = "IAVL"];
  }
}

// SnapshotStoreItem contains metadata about a snapshotted store.
message SnapshotStoreItem {
  string name = 1;
}

// SnapshotIAVLItem is an exported IAVL node.
message SnapshotIAVLItem {
  bytes key     = 1;
  bytes value   = 2;
  int64 version = 3;
  int32 height  = 4;
}
```

Snapshots are generated by `rootmulti.Store.Snapshot()` as follows:

1. Set up a `protoio.NewDelimitedWriter` that writes length-prefixed serialized
   `SnapshotItem` Protobuf messages.
    1. Iterate over each IAVL store in lexicographical order by store name.
    2. Emit a `SnapshotStoreItem` containing the store name.
    3. Start an IAVL export for the store using
       [`iavl.ImmutableTree.Export()`](https://pkg.go.dev/github.com/cosmos/iavl#ImmutableTree.Export).
    4. Iterate over each IAVL node.
    5. Emit a `SnapshotIAVLItem` for the IAVL node.
2. Pass the serialized Protobuf output stream to a zlib compression writer.
3. Split the zlib output stream into chunks at exactly every 10th megabyte.

Snapshots are restored via `rootmulti.Store.Restore()` as the inverse of the above, using
[`iavl.MutableTree.Import()`](https://pkg.go.dev/github.com/cosmos/iavl#MutableTree.Import)
to reconstruct each IAVL tree.

## Snapshot Storage

Snapshot storage is managed by `snapshots.Store`, with metadata in a `db.DB`
database and binary chunks in the filesystem. Note that this is only used to
store locally taken snapshots that are being offered to other nodes. When the
local node is being state synced, CometBFT will take care of buffering and
storing incoming snapshot chunks before they are applied to the application.

Metadata is generally stored in a LevelDB database at
`<node_home>/data/snapshots/metadata.db`. It contains serialized
`cosmos.base.snapshots.v1beta1.Snapshot` Protobuf messages with a key given by
the concatenation of a key prefix, the big-endian height, and the big-endian
format. Chunk data is stored as regular files under
`<node_home>/data/snapshots/<height>/<format>/<chunk>`.

The `snapshots.Store` API is based on streaming IO, and integrates easily with
the `snapshots.types.Snapshotter` snapshot/restore interface implemented by
`rootmulti.Store`. The `Store.Save()` method stores a snapshot given as a
`<- chan io.ReadCloser` channel of binary chunk streams, and `Store.Load()` loads
the snapshot as a channel of binary chunk streams -- the same stream types used
by `Snapshotter.Snapshot()` and `Snapshotter.Restore()` to take and restore
snapshots using streaming IO.

The store also provides many other methods such as `List()` to list stored
snapshots, `LoadChunk()` to load a single snapshot chunk, and `Prune()` to prune
old snapshots.

## Taking Snapshots

`snapshots.Manager` is a high-level snapshot manager that integrates a
`snapshots.types.Snapshotter` (i.e. the `rootmulti.Store` snapshot
functionality) and a `snapshots.Store`, providing an API that maps easily onto
the ABCI state sync API. The `Manager` will also make sure only one operation
is in progress at a time, e.g. to prevent multiple snapshots being taken
concurrently.

During `BaseApp.Commit`, once a state transition has been committed, the height
is checked against the `state-sync.snapshot-interval` setting. If the committed
height should be snapshotted, a goroutine `BaseApp.snapshot()` is spawned that
calls `snapshots.Manager.Create()` to create the snapshot. Once a snapshot is
complete and if pruning is enabled, the snapshot height is pruned away by the manager
with the call `PruneSnapshotHeight(...)` to the `snapshots.types.Snapshotter`.

`Manager.Create()` will do some basic pre-flight checks, and then start
generating a snapshot by calling `rootmulti.Store.Snapshot()`. The chunk stream
is passed into `snapshots.Store.Save()`, which stores the chunks in the
filesystem and records the snapshot metadata in the snapshot database.

Once the snapshot has been generated, `BaseApp.snapshot()` then removes any
old snapshots based on the `state-sync.snapshot-keep-recent` setting.

## Serving Snapshots

When a remote node is discovering snapshots for state sync, CometBFT will
call the `ListSnapshots` ABCI method to list the snapshots present on the
local node. This is dispatched to `snapshots.Manager.List()`, which in turn
dispatches to `snapshots.Store.List()`.

When a remote node is fetching snapshot chunks during state sync, CometBFT
will call the `LoadSnapshotChunk` ABCI method to fetch a chunk from the local
node. This dispatches to `snapshots.Manager.LoadChunk()`, which in turn
dispatches to `snapshots.Store.LoadChunk()`.

## Restoring Snapshots

When the operator has configured the local CometBFT node to run state sync
(see the resources listed in the introduction for details on CometBFT state
sync), it will discover snapshots across the P2P network and offer their
metadata in turn to the local application via the `OfferSnapshot` ABCI call.

`BaseApp.OfferSnapshot()` attempts to start a restore operation by calling
`snapshots.Manager.Restore()`. This may fail, e.g. if the snapshot format is
unknown (it may have been generated by a different version of the Cosmos SDK),
in which case CometBFT will offer other discovered snapshots.

If the snapshot is accepted, `Manager.Restore()` will record that a restore
operation is in progress, and spawn a separate goroutine that runs a synchronous
`rootmulti.Store.Restore()` snapshot restoration which will be fed snapshot
chunks until it is complete.

CometBFT will then start fetching and buffering chunks, providing them in
order via ABCI `ApplySnapshotChunk` calls. These dispatch to
`Manager.RestoreChunk()`, which passes the chunks to the ongoing restore
process, checking if errors have been encountered yet (e.g. due to checksum
mismatches or invalid IAVL data). Once the final chunk is passed,
`Manager.RestoreChunk()` will wait for the restore process to complete before
returning.

Once the restore is completed, CometBFT will go on to call the `Info` ABCI
call to fetch the app hash, and compare this against the trusted chain app
hash at the snapshot height to verify the restored state. If it matches,
CometBFT goes on to process blocks.
