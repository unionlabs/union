package unorderedtx

import (
	"encoding/binary"
	"errors"
	"io"
	"time"

	snapshot "cosmossdk.io/store/snapshots/types"
)

const (
	txHashSize  = 32
	timeoutSize = 8
	chunkSize   = txHashSize + timeoutSize
)

var _ snapshot.ExtensionSnapshotter = &Snapshotter{}

const (
	// SnapshotFormat defines the snapshot format of exported unordered transactions.
	// No protobuf envelope, no metadata.
	SnapshotFormat = 1

	// SnapshotName defines the snapshot name of exported unordered transactions.
	SnapshotName = "unordered_txs"
)

type Snapshotter struct {
	m *Manager
}

func NewSnapshotter(m *Manager) *Snapshotter {
	return &Snapshotter{m: m}
}

func (s *Snapshotter) SnapshotName() string {
	return SnapshotName
}

func (s *Snapshotter) SnapshotFormat() uint32 {
	return SnapshotFormat
}

func (s *Snapshotter) SupportedFormats() []uint32 {
	return []uint32{SnapshotFormat}
}

func (s *Snapshotter) SnapshotExtension(height uint64, payloadWriter snapshot.ExtensionPayloadWriter) error {
	// export all unordered transactions as a single blob
	return s.m.exportSnapshot(height, payloadWriter)
}

func (s *Snapshotter) RestoreExtension(height uint64, format uint32, payloadReader snapshot.ExtensionPayloadReader) error {
	if format == SnapshotFormat {
		return s.restore(height, payloadReader)
	}

	return snapshot.ErrUnknownFormat
}

func (s *Snapshotter) restore(height uint64, payloadReader snapshot.ExtensionPayloadReader) error {
	// the payload should be the entire set of unordered transactions
	payload, err := payloadReader()
	if err != nil {
		if errors.Is(err, io.EOF) {
			return io.ErrUnexpectedEOF
		}

		return err
	}

	if len(payload)%chunkSize != 0 {
		return errors.New("invalid unordered txs payload length")
	}

	var i int
	for i < len(payload) {
		var txHash TxHash
		copy(txHash[:], payload[i:i+txHashSize])

		timestamp := binary.BigEndian.Uint64(payload[i+txHashSize : i+chunkSize])

		// add all txs, we don't care at this point if they are expired,
		// we'll let the purge loop handle that
		if timestamp != 0 {
			s.m.Add(txHash, time.Unix(int64(timestamp), 0))
		}

		i += chunkSize
	}

	return nil
}
