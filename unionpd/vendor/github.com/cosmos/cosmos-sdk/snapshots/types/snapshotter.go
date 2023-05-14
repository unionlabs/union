package types

import (
	protoio "github.com/cosmos/gogoproto/io"
)

// Snapshotter is something that can create and restore snapshots, consisting of streamed binary
// chunks - all of which must be read from the channel and closed. If an unsupported format is
// given, it must return ErrUnknownFormat (possibly wrapped with fmt.Errorf).
type Snapshotter interface {
	// Snapshot writes snapshot items into the protobuf writer.
	Snapshot(height uint64, protoWriter protoio.Writer) error

	// PruneSnapshotHeight prunes the given height according to the prune strategy.
	// If PruneNothing, this is a no-op.
	// If other strategy, this height is persisted until it is
	// less than <current height> - KeepRecent and <current height> % Interval == 0
	PruneSnapshotHeight(height int64)

	// SetSnapshotInterval sets the interval at which the snapshots are taken.
	// It is used by the store that implements the Snapshotter interface
	// to determine which heights to retain until after the snapshot is complete.
	SetSnapshotInterval(snapshotInterval uint64)

	// Restore restores a state snapshot, taking the reader of protobuf message stream as input.
	Restore(height uint64, format uint32, protoReader protoio.Reader) (SnapshotItem, error)
}

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

	// SnapshotFormat returns the default format the extension snapshotter use to encode the
	// payloads when taking a snapshot.
	// It's defined within the extension, different from the global format for the whole state-sync snapshot.
	SnapshotFormat() uint32

	// SupportedFormats returns a list of formats it can restore from.
	SupportedFormats() []uint32

	// SnapshotExtension writes extension payloads into the underlying protobuf stream.
	SnapshotExtension(height uint64, payloadWriter ExtensionPayloadWriter) error

	// RestoreExtension restores an extension state snapshot,
	// the payload reader returns `io.EOF` when reached the extension boundaries.
	RestoreExtension(height uint64, format uint32, payloadReader ExtensionPayloadReader) error
}
