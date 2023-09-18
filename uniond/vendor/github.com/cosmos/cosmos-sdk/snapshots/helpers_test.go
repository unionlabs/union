package snapshots_test

import (
	"bufio"
	"bytes"
	"compress/zlib"
	"crypto/sha256"
	"errors"
	"io"
	"testing"
	"time"

	db "github.com/cometbft/cometbft-db"
	"github.com/cometbft/cometbft/libs/log"
	protoio "github.com/cosmos/gogoproto/io"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/snapshots"
	snapshottypes "github.com/cosmos/cosmos-sdk/snapshots/types"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

func checksums(slice [][]byte) [][]byte {
	hasher := sha256.New()
	checksums := make([][]byte, len(slice))
	for i, chunk := range slice {
		hasher.Write(chunk)
		checksums[i] = hasher.Sum(nil)
		hasher.Reset()
	}
	return checksums
}

func hash(chunks [][]byte) []byte {
	hasher := sha256.New()
	for _, chunk := range chunks {
		hasher.Write(chunk)
	}
	return hasher.Sum(nil)
}

func makeChunks(chunks [][]byte) <-chan io.ReadCloser {
	ch := make(chan io.ReadCloser, len(chunks))
	for _, chunk := range chunks {
		ch <- io.NopCloser(bytes.NewReader(chunk))
	}
	close(ch)
	return ch
}

func readChunks(chunks <-chan io.ReadCloser) [][]byte {
	bodies := [][]byte{}
	for chunk := range chunks {
		body, err := io.ReadAll(chunk)
		if err != nil {
			panic(err)
		}
		bodies = append(bodies, body)
	}
	return bodies
}

// snapshotItems serialize a array of bytes as SnapshotItem_ExtensionPayload, and return the chunks.
func snapshotItems(items [][]byte, ext snapshottypes.ExtensionSnapshotter) [][]byte {
	// copy the same parameters from the code
	snapshotChunkSize := uint64(10e6)
	snapshotBufferSize := int(snapshotChunkSize)

	ch := make(chan io.ReadCloser)
	go func() {
		chunkWriter := snapshots.NewChunkWriter(ch, snapshotChunkSize)
		bufWriter := bufio.NewWriterSize(chunkWriter, snapshotBufferSize)
		zWriter, _ := zlib.NewWriterLevel(bufWriter, 7)
		protoWriter := protoio.NewDelimitedWriter(zWriter)
		for _, item := range items {
			_ = snapshottypes.WriteExtensionPayload(protoWriter, item)
		}
		// write extension metadata
		_ = protoWriter.WriteMsg(&snapshottypes.SnapshotItem{
			Item: &snapshottypes.SnapshotItem_Extension{
				Extension: &snapshottypes.SnapshotExtensionMeta{
					Name:   ext.SnapshotName(),
					Format: ext.SnapshotFormat(),
				},
			},
		})
		_ = ext.SnapshotExtension(0, func(payload []byte) error {
			return snapshottypes.WriteExtensionPayload(protoWriter, payload)
		})
		_ = protoWriter.Close()
		_ = bufWriter.Flush()
		_ = chunkWriter.Close()
	}()

	var chunks [][]byte
	for chunkBody := range ch {
		chunk, err := io.ReadAll(chunkBody)
		if err != nil {
			panic(err)
		}
		chunks = append(chunks, chunk)
	}

	return chunks
}

type mockSnapshotter struct {
	items            [][]byte
	prunedHeights    map[int64]struct{}
	snapshotInterval uint64
}

func (m *mockSnapshotter) Restore(
	height uint64, format uint32, protoReader protoio.Reader,
) (snapshottypes.SnapshotItem, error) {
	if format == 0 {
		return snapshottypes.SnapshotItem{}, snapshottypes.ErrUnknownFormat
	}
	if m.items != nil {
		return snapshottypes.SnapshotItem{}, errors.New("already has contents")
	}

	var item snapshottypes.SnapshotItem
	m.items = [][]byte{}
	for {
		item.Reset()
		err := protoReader.ReadMsg(&item)
		if err == io.EOF {
			break
		} else if err != nil {
			return snapshottypes.SnapshotItem{}, sdkerrors.Wrap(err, "invalid protobuf message")
		}
		payload := item.GetExtensionPayload()
		if payload == nil {
			break
		}
		m.items = append(m.items, payload.Payload)
	}

	return item, nil
}

func (m *mockSnapshotter) Snapshot(height uint64, protoWriter protoio.Writer) error {
	for _, item := range m.items {
		if err := snapshottypes.WriteExtensionPayload(protoWriter, item); err != nil {
			return err
		}
	}
	return nil
}

func (m *mockSnapshotter) SnapshotFormat() uint32 {
	return snapshottypes.CurrentFormat
}

func (m *mockSnapshotter) SupportedFormats() []uint32 {
	return []uint32{snapshottypes.CurrentFormat}
}

func (m *mockSnapshotter) PruneSnapshotHeight(height int64) {
	m.prunedHeights[height] = struct{}{}
}

func (m *mockSnapshotter) GetSnapshotInterval() uint64 {
	return m.snapshotInterval
}

func (m *mockSnapshotter) SetSnapshotInterval(snapshotInterval uint64) {
	m.snapshotInterval = snapshotInterval
}

type mockErrorSnapshotter struct{}

var _ snapshottypes.Snapshotter = (*mockErrorSnapshotter)(nil)

func (m *mockErrorSnapshotter) Snapshot(height uint64, protoWriter protoio.Writer) error {
	return errors.New("mock snapshot error")
}

func (m *mockErrorSnapshotter) Restore(
	height uint64, format uint32, protoReader protoio.Reader,
) (snapshottypes.SnapshotItem, error) {
	return snapshottypes.SnapshotItem{}, errors.New("mock restore error")
}

func (m *mockErrorSnapshotter) SnapshotFormat() uint32 {
	return snapshottypes.CurrentFormat
}

func (m *mockErrorSnapshotter) SupportedFormats() []uint32 {
	return []uint32{snapshottypes.CurrentFormat}
}

func (m *mockErrorSnapshotter) PruneSnapshotHeight(height int64) {
}

func (m *mockErrorSnapshotter) GetSnapshotInterval() uint64 {
	return 0
}

func (m *mockErrorSnapshotter) SetSnapshotInterval(snapshotInterval uint64) {
}

// setupBusyManager creates a manager with an empty store that is busy creating a snapshot at height 1.
// The snapshot will complete when the returned closer is called.
func setupBusyManager(t *testing.T) *snapshots.Manager {
	store, err := snapshots.NewStore(db.NewMemDB(), testutil.GetTempDir(t))
	require.NoError(t, err)
	hung := newHungSnapshotter()
	hung.SetSnapshotInterval(opts.Interval)
	mgr := snapshots.NewManager(store, opts, hung, nil, log.NewNopLogger())
	require.Equal(t, opts.Interval, hung.snapshotInterval)

	go func() {
		_, err := mgr.Create(1)
		require.NoError(t, err)
		_, didPruneHeight := hung.prunedHeights[1]
		require.True(t, didPruneHeight)
	}()
	time.Sleep(10 * time.Millisecond)
	t.Cleanup(hung.Close)

	return mgr
}

// hungSnapshotter can be used to test operations in progress. Call close to end the snapshot.
type hungSnapshotter struct {
	ch               chan struct{}
	prunedHeights    map[int64]struct{}
	snapshotInterval uint64
}

func newHungSnapshotter() *hungSnapshotter {
	return &hungSnapshotter{
		ch:            make(chan struct{}),
		prunedHeights: make(map[int64]struct{}),
	}
}

func (m *hungSnapshotter) Close() {
	close(m.ch)
}

func (m *hungSnapshotter) Snapshot(height uint64, protoWriter protoio.Writer) error {
	<-m.ch
	return nil
}

func (m *hungSnapshotter) PruneSnapshotHeight(height int64) {
	m.prunedHeights[height] = struct{}{}
}

func (m *hungSnapshotter) SetSnapshotInterval(snapshotInterval uint64) {
	m.snapshotInterval = snapshotInterval
}

func (m *hungSnapshotter) Restore(
	height uint64, format uint32, protoReader protoio.Reader,
) (snapshottypes.SnapshotItem, error) {
	panic("not implemented")
}

type extSnapshotter struct {
	state []uint64
}

func newExtSnapshotter(count int) *extSnapshotter {
	state := make([]uint64, 0, count)
	for i := 0; i < count; i++ {
		state = append(state, uint64(i))
	}
	return &extSnapshotter{
		state,
	}
}

func (s *extSnapshotter) SnapshotName() string {
	return "mock"
}

func (s *extSnapshotter) SnapshotFormat() uint32 {
	return 1
}

func (s *extSnapshotter) SupportedFormats() []uint32 {
	return []uint32{1}
}

func (s *extSnapshotter) SnapshotExtension(height uint64, payloadWriter snapshottypes.ExtensionPayloadWriter) error {
	for _, i := range s.state {
		if err := payloadWriter(sdk.Uint64ToBigEndian(uint64(i))); err != nil {
			return err
		}
	}
	return nil
}

func (s *extSnapshotter) RestoreExtension(height uint64, format uint32, payloadReader snapshottypes.ExtensionPayloadReader) error {
	for {
		payload, err := payloadReader()
		if err == io.EOF {
			break
		} else if err != nil {
			return err
		}
		s.state = append(s.state, sdk.BigEndianToUint64(payload))
	}
	// finalize restoration
	return nil
}
