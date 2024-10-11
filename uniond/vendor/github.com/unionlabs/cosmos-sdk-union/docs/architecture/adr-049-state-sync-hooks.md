# ADR 049: State Sync Hooks

## Changelog

* Jan 19, 2022: Initial Draft
* Apr 29, 2022: Safer extension snapshotter interface

## Status

Implemented

## Abstract

This ADR outlines a hooks-based mechanism for application modules to provide additional state (outside of the IAVL tree) to be used 
during state sync.

## Context

New clients use state-sync to download snapshots of module state from peers. Currently, the snapshot consists of a
stream of `SnapshotStoreItem` and `SnapshotIAVLItem`, which means that application modules that define their state outside of the IAVL 
tree cannot include their state as part of the state-sync process.

Note, Even though the module state data is outside of the tree, for determinism we require that the hash of the external data should 
be posted in the IAVL tree.

## Decision

A simple proposal based on our existing implementation is that, we can add two new message types: `SnapshotExtensionMeta` 
and `SnapshotExtensionPayload`, and they are appended to the existing multi-store stream with `SnapshotExtensionMeta` 
acting as a delimiter between extensions. As the chunk hashes should be able to ensure data integrity, we don't need 
a delimiter to mark the end of the snapshot stream.

Besides, we provide `Snapshotter` and `ExtensionSnapshotter` interface for modules to implement snapshotters, which will handle both taking 
snapshot and the restoration. Each module could have multiple snapshotters, and for modules with additional state, they should
implement `ExtensionSnapshotter` as extension snapshotters. When setting up the application, the snapshot `Manager` should call 
`RegisterExtensions([]ExtensionSnapshotter…)` to register all the extension snapshotters.

```protobuf
// SnapshotItem is an item contained in a rootmulti.Store snapshot.
// On top of the existing SnapshotStoreItem and SnapshotIAVLItem, we add two new options for the item.
message SnapshotItem {
  // item is the specific type of snapshot item.
  oneof item {
    SnapshotStoreItem        store             = 1;
    SnapshotIAVLItem         iavl              = 2 [(gogoproto.customname) = "IAVL"];
    SnapshotExtensionMeta    extension         = 3;
    SnapshotExtensionPayload extension_payload = 4;
  }
}

// SnapshotExtensionMeta contains metadata about an external snapshotter.
// One module may need multiple snapshotters, so each module may have multiple SnapshotExtensionMeta.
message SnapshotExtensionMeta {
  // the name of the ExtensionSnapshotter, and it is registered to snapshotter manager when setting up the application
  // name should be unique for each ExtensionSnapshotter as we need to alphabetically order their snapshots to get
  // deterministic snapshot stream.
  string name   = 1;
  // this is used by each ExtensionSnapshotter to decide the format of payloads included in SnapshotExtensionPayload message
  // it is used within the snapshotter/namespace, not global one for all modules
  uint32 format = 2;
}

// SnapshotExtensionPayload contains payloads of an external snapshotter.
message SnapshotExtensionPayload {
  bytes payload = 1;
}
```

When we create a snapshot stream, the `multistore` snapshot is always placed at the beginning of the binary stream, and other extension snapshots are alphabetically ordered by the name of the corresponding `ExtensionSnapshotter`. 

The snapshot stream would look like as follows:

```go
// multi-store snapshot
{SnapshotStoreItem | SnapshotIAVLItem, ...}
// extension1 snapshot
SnapshotExtensionMeta
{SnapshotExtensionPayload, ...}
// extension2 snapshot
SnapshotExtensionMeta
{SnapshotExtensionPayload, ...}
```

We add an `extensions` field to snapshot `Manager` for extension snapshotters. The `multistore` snapshotter is a special one and it doesn't need a name because it is always placed at the beginning of the binary stream.

```go
type Manager struct {
	store      *Store
	multistore types.Snapshotter
	extensions map[string]types.ExtensionSnapshotter
	mtx                sync.Mutex
	operation          operation
	chRestore          chan<- io.ReadCloser
	chRestoreDone      <-chan restoreDone
	restoreChunkHashes [][]byte
	restoreChunkIndex  uint32
}
```

For extension snapshotters that implement the `ExtensionSnapshotter` interface, their names should be registered to the snapshot `Manager` by 
calling `RegisterExtensions` when setting up the application. The snapshotters will handle both taking snapshot and restoration.

```go
// RegisterExtensions register extension snapshotters to manager
func (m *Manager) RegisterExtensions(extensions ...types.ExtensionSnapshotter) error 
```

On top of the existing `Snapshotter` interface for the `multistore`, we add `ExtensionSnapshotter` interface for the extension snapshotters. Three more function signatures: `SnapshotFormat()`, `SupportedFormats()` and `SnapshotName()` are added to `ExtensionSnapshotter`.

```go
// ExtensionPayloadReader read extension payloads,
// it returns io.EOF when reached either end of stream or the extension boundaries.
type ExtensionPayloadReader = func() ([]byte, error)

// ExtensionPayloadWriter is a helper to write extension payloads to underlying stream.
type ExtensionPayloadWriter = func([]byte) error

// ExtensionSnapshotter is an extension Snapshotter that is appended to the snapshot stream.
// ExtensionSnapshotter has an unique name and manages it's own internal formats.
type ExtensionSnapshotter interface {
	// SnapshotName returns the name of snapshotter, it should be unique in the manager.
	SnapshotName() string

	// SnapshotFormat returns the default format used to take a snapshot.
	SnapshotFormat() uint32

	// SupportedFormats returns a list of formats it can restore from.
	SupportedFormats() []uint32

	// SnapshotExtension writes extension payloads into the underlying protobuf stream.
	SnapshotExtension(height uint64, payloadWriter ExtensionPayloadWriter) error

	// RestoreExtension restores an extension state snapshot,
	// the payload reader returns `io.EOF` when reached the extension boundaries.
	RestoreExtension(height uint64, format uint32, payloadReader ExtensionPayloadReader) error

}
```

## Consequences

As a result of this implementation, we are able to create snapshots of binary chunk stream for the state that we maintain outside of the IAVL Tree, CosmWasm blobs for example. And new clients are able to fetch snapshots of state for all modules that have implemented the corresponding interface from peer nodes.


### Backwards Compatibility

This ADR introduces new proto message types, add an `extensions` field in snapshot `Manager`, and add new `ExtensionSnapshotter` interface, so this is not backwards compatible if we have extensions.

But for applications that does not have the state data outside of the IAVL tree for any module, the snapshot stream is backwards-compatible.

### Positive

* State maintained outside of IAVL tree like CosmWasm blobs can create snapshots by implementing extension snapshotters, and being fetched by new clients via state-sync.

### Negative

### Neutral

* All modules that maintain state outside of IAVL tree need to implement `ExtensionSnapshotter` and the snapshot `Manager` need to call `RegisterExtensions` when setting up the application.

## Further Discussions

While an ADR is in the DRAFT or PROPOSED stage, this section should contain a summary of issues to be solved in future iterations (usually referencing comments from a pull-request discussion).
Later, this section can optionally list ideas or improvements the author or reviewers found during the analysis of this ADR.

## Test Cases [optional]

Test cases for an implementation are mandatory for ADRs that are affecting consensus changes. Other ADRs can choose to include links to test cases if applicable.

## References

* https://github.com/cosmos/cosmos-sdk/pull/10961
* https://github.com/cosmos/cosmos-sdk/issues/7340
