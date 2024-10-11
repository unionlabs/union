package snapshots_test

import (
	"bytes"
	"errors"
	"io"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/store/snapshots"
	"cosmossdk.io/store/snapshots/types"
)

func setupStore(t *testing.T) *snapshots.Store {
	t.Helper()
	store, err := snapshots.NewStore(coretesting.NewMemDB(), GetTempDir(t))
	require.NoError(t, err)

	_, err = store.Save(1, 1, makeChunks([][]byte{
		{1, 1, 0}, {1, 1, 1},
	}))
	require.NoError(t, err)
	_, err = store.Save(2, 1, makeChunks([][]byte{
		{2, 1, 0}, {2, 1, 1},
	}))
	require.NoError(t, err)
	_, err = store.Save(2, 2, makeChunks([][]byte{
		{2, 2, 0}, {2, 2, 1}, {2, 2, 2},
	}))
	require.NoError(t, err)
	_, err = store.Save(3, 2, makeChunks([][]byte{
		{3, 2, 0}, {3, 2, 1}, {3, 2, 2},
	}))
	require.NoError(t, err)

	return store
}

func TestNewStore(t *testing.T) {
	tempdir := GetTempDir(t)
	_, err := snapshots.NewStore(coretesting.NewMemDB(), tempdir)

	require.NoError(t, err)
}

func TestNewStore_ErrNoDir(t *testing.T) {
	_, err := snapshots.NewStore(coretesting.NewMemDB(), "")
	require.Error(t, err)
}

func TestStore_Delete(t *testing.T) {
	store := setupStore(t)
	// Deleting a snapshot should remove it
	err := store.Delete(2, 2)
	require.NoError(t, err)

	snapshot, err := store.Get(2, 2)
	require.NoError(t, err)
	assert.Nil(t, snapshot)

	snapshots, err := store.List()
	require.NoError(t, err)
	assert.Len(t, snapshots, 3)

	// Deleting it again should not error
	err = store.Delete(2, 2)
	require.NoError(t, err)

	// Deleting a snapshot being saved should error
	ch := make(chan io.ReadCloser)
	go func() {
		_, err := store.Save(9, 1, ch)
		require.NoError(t, err)
	}()

	time.Sleep(10 * time.Millisecond)
	err = store.Delete(9, 1)
	require.Error(t, err)

	// But after it's saved it should work
	close(ch)
	time.Sleep(10 * time.Millisecond)
	err = store.Delete(9, 1)
	require.NoError(t, err)
}

func TestStore_Get(t *testing.T) {
	store := setupStore(t)

	// Loading a missing snapshot should return nil
	snapshot, err := store.Get(9, 9)
	require.NoError(t, err)
	assert.Nil(t, snapshot)

	// Loading a snapshot should returns its metadata
	snapshot, err = store.Get(2, 1)
	require.NoError(t, err)
	assert.Equal(t, &types.Snapshot{
		Height: 2,
		Format: 1,
		Chunks: 2,
		Hash:   hash([][]byte{{2, 1, 0}, {2, 1, 1}}),
		Metadata: types.Metadata{
			ChunkHashes: checksums([][]byte{
				{2, 1, 0}, {2, 1, 1},
			}),
		},
	}, snapshot)
}

func TestStore_GetLatest(t *testing.T) {
	store := setupStore(t)
	// Loading a missing snapshot should return nil
	snapshot, err := store.GetLatest()
	require.NoError(t, err)
	assert.Equal(t, &types.Snapshot{
		Height: 3,
		Format: 2,
		Chunks: 3,
		Hash: hash([][]byte{
			{3, 2, 0},
			{3, 2, 1},
			{3, 2, 2},
		}),
		Metadata: types.Metadata{
			ChunkHashes: checksums([][]byte{
				{3, 2, 0},
				{3, 2, 1},
				{3, 2, 2},
			}),
		},
	}, snapshot)
}

func TestStore_List(t *testing.T) {
	store := setupStore(t)
	snapshots, err := store.List()
	require.NoError(t, err)

	require.Equal(t, []*types.Snapshot{
		{
			Height: 3, Format: 2, Chunks: 3, Hash: hash([][]byte{{3, 2, 0}, {3, 2, 1}, {3, 2, 2}}),
			Metadata: types.Metadata{ChunkHashes: checksums([][]byte{{3, 2, 0}, {3, 2, 1}, {3, 2, 2}})},
		},
		{
			Height: 2, Format: 2, Chunks: 3, Hash: hash([][]byte{{2, 2, 0}, {2, 2, 1}, {2, 2, 2}}),
			Metadata: types.Metadata{ChunkHashes: checksums([][]byte{{2, 2, 0}, {2, 2, 1}, {2, 2, 2}})},
		},
		{
			Height: 2, Format: 1, Chunks: 2, Hash: hash([][]byte{{2, 1, 0}, {2, 1, 1}}),
			Metadata: types.Metadata{ChunkHashes: checksums([][]byte{{2, 1, 0}, {2, 1, 1}})},
		},
		{
			Height: 1, Format: 1, Chunks: 2, Hash: hash([][]byte{{1, 1, 0}, {1, 1, 1}}),
			Metadata: types.Metadata{ChunkHashes: checksums([][]byte{{1, 1, 0}, {1, 1, 1}})},
		},
	}, snapshots)
}

func TestStore_Load(t *testing.T) {
	store := setupStore(t)
	// Loading a missing snapshot should return nil
	snapshot, chunks, err := store.Load(9, 9)
	require.NoError(t, err)
	assert.Nil(t, snapshot)
	assert.Nil(t, chunks)

	// Loading a snapshot should returns its metadata and chunks
	snapshot, chunks, err = store.Load(2, 1)
	require.NoError(t, err)
	assert.Equal(t, &types.Snapshot{
		Height: 2,
		Format: 1,
		Chunks: 2,
		Hash:   hash([][]byte{{2, 1, 0}, {2, 1, 1}}),
		Metadata: types.Metadata{
			ChunkHashes: checksums([][]byte{
				{2, 1, 0}, {2, 1, 1},
			}),
		},
	}, snapshot)

	for i := uint32(0); i < snapshot.Chunks; i++ {
		reader, ok := <-chunks
		require.True(t, ok)
		chunk, err := io.ReadAll(reader)
		require.NoError(t, err)
		err = reader.Close()
		require.NoError(t, err)
		assert.Equal(t, []byte{2, 1, byte(i)}, chunk)
	}
	assert.Empty(t, chunks)
}

func TestStore_LoadChunk(t *testing.T) {
	store := setupStore(t)
	// Loading a missing snapshot should return nil
	chunk, err := store.LoadChunk(9, 9, 0)
	require.NoError(t, err)
	assert.Nil(t, chunk)

	// Loading a missing chunk index should return nil
	chunk, err = store.LoadChunk(2, 1, 2)
	require.NoError(t, err)
	require.Nil(t, chunk)

	// Loading a chunk should returns a content reader
	chunk, err = store.LoadChunk(2, 1, 0)
	require.NoError(t, err)
	require.NotNil(t, chunk)
	body, err := io.ReadAll(chunk)
	require.NoError(t, err)
	assert.Equal(t, []byte{2, 1, 0}, body)
	err = chunk.Close()
	require.NoError(t, err)
}

func TestStore_Prune(t *testing.T) {
	store := setupStore(t)
	// Pruning too many snapshots should be fine
	pruned, err := store.Prune(4)
	require.NoError(t, err)
	assert.EqualValues(t, 0, pruned)

	snapshots, err := store.List()
	require.NoError(t, err)
	assert.Len(t, snapshots, 4)

	// Pruning until the last two heights should leave three snapshots (for two heights)
	pruned, err = store.Prune(2)
	require.NoError(t, err)
	assert.EqualValues(t, 1, pruned)

	snapshots, err = store.List()
	require.NoError(t, err)
	require.Equal(t, []*types.Snapshot{
		{
			Height: 3, Format: 2, Chunks: 3, Hash: hash([][]byte{{3, 2, 0}, {3, 2, 1}, {3, 2, 2}}),
			Metadata: types.Metadata{ChunkHashes: checksums([][]byte{{3, 2, 0}, {3, 2, 1}, {3, 2, 2}})},
		},
		{
			Height: 2, Format: 2, Chunks: 3, Hash: hash([][]byte{{2, 2, 0}, {2, 2, 1}, {2, 2, 2}}),
			Metadata: types.Metadata{ChunkHashes: checksums([][]byte{{2, 2, 0}, {2, 2, 1}, {2, 2, 2}})},
		},
		{
			Height: 2, Format: 1, Chunks: 2, Hash: hash([][]byte{{2, 1, 0}, {2, 1, 1}}),
			Metadata: types.Metadata{ChunkHashes: checksums([][]byte{{2, 1, 0}, {2, 1, 1}})},
		},
	}, snapshots)

	// Pruning all heights should also be fine
	pruned, err = store.Prune(0)
	require.NoError(t, err)
	assert.EqualValues(t, 3, pruned)

	snapshots, err = store.List()
	require.NoError(t, err)
	assert.Empty(t, snapshots)
}

func TestStore_Save(t *testing.T) {
	store := setupStore(t)
	// Saving a snapshot should work
	snapshot, err := store.Save(4, 1, makeChunks([][]byte{{1}, {2}}))
	require.NoError(t, err)
	assert.Equal(t, &types.Snapshot{
		Height: 4,
		Format: 1,
		Chunks: 2,
		Hash:   hash([][]byte{{1}, {2}}),
		Metadata: types.Metadata{
			ChunkHashes: checksums([][]byte{{1}, {2}}),
		},
	}, snapshot)
	loaded, err := store.Get(snapshot.Height, snapshot.Format)
	require.NoError(t, err)
	assert.Equal(t, snapshot, loaded)

	// Saving an existing snapshot should error
	_, err = store.Save(4, 1, makeChunks([][]byte{{1}, {2}}))
	require.Error(t, err)

	// Saving at height 0 should error
	_, err = store.Save(0, 1, makeChunks([][]byte{{1}, {2}}))
	require.Error(t, err)

	// Saving at format 0 should be fine
	_, err = store.Save(1, 0, makeChunks([][]byte{{1}, {2}}))
	require.NoError(t, err)

	// Saving a snapshot with no chunks should be fine, as should loading it
	_, err = store.Save(5, 1, makeChunks([][]byte{}))
	require.NoError(t, err)
	snapshot, chunks, err := store.Load(5, 1)
	require.NoError(t, err)
	assert.Equal(t, &types.Snapshot{Height: 5, Format: 1, Hash: hash([][]byte{}), Metadata: types.Metadata{ChunkHashes: [][]byte{}}}, snapshot)
	assert.Empty(t, chunks)

	// Saving a snapshot should error if a chunk reader returns an error, and it should empty out
	// the channel
	someErr := errors.New("boom")
	pr, pw := io.Pipe()
	err = pw.CloseWithError(someErr)
	require.NoError(t, err)

	ch := make(chan io.ReadCloser, 2)
	ch <- pr
	ch <- io.NopCloser(bytes.NewBuffer([]byte{0xff}))
	close(ch)

	_, err = store.Save(6, 1, ch)
	require.Error(t, err)
	require.True(t, errors.Is(err, someErr))
	assert.Empty(t, ch)

	// Saving a snapshot should error if a snapshot is already in progress for the same height,
	// regardless of format. However, a different height should succeed.
	ch = make(chan io.ReadCloser)
	go func() {
		_, err := store.Save(7, 1, ch)
		require.NoError(t, err)
	}()
	time.Sleep(10 * time.Millisecond)
	_, err = store.Save(7, 2, makeChunks(nil))
	require.Error(t, err)
	_, err = store.Save(8, 1, makeChunks(nil))
	require.NoError(t, err)
	close(ch)
}
