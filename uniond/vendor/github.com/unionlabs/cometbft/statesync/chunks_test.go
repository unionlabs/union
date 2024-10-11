package statesync

import (
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/p2p"
)

func setupChunkQueue(t *testing.T) (*chunkQueue, func()) {
	t.Helper()
	snapshot := &snapshot{
		Height:   3,
		Format:   1,
		Chunks:   5,
		Hash:     []byte{7},
		Metadata: nil,
	}
	queue, err := newChunkQueue(snapshot, "")
	require.NoError(t, err)
	teardown := func() {
		err := queue.Close()
		require.NoError(t, err)
	}
	return queue, teardown
}

func TestNewChunkQueue_TempDir(t *testing.T) {
	snapshot := &snapshot{
		Height:   3,
		Format:   1,
		Chunks:   5,
		Hash:     []byte{7},
		Metadata: nil,
	}
	dir, err := os.MkdirTemp("", "newchunkqueue")
	require.NoError(t, err)
	defer os.RemoveAll(dir)
	queue, err := newChunkQueue(snapshot, dir)
	require.NoError(t, err)

	files, err := os.ReadDir(dir)
	require.NoError(t, err)
	assert.Len(t, files, 1)

	err = queue.Close()
	require.NoError(t, err)

	files, err = os.ReadDir(dir)
	require.NoError(t, err)
	assert.Empty(t, files)
}

func TestChunkQueue(t *testing.T) {
	queue, teardown := setupChunkQueue(t)
	defer teardown()

	// Adding the first chunk should be fine
	added, err := queue.Add(&chunk{Height: 3, Format: 1, Index: 0, Chunk: []byte{3, 1, 0}})
	require.NoError(t, err)
	assert.True(t, added)

	// Adding the last chunk should also be fine
	added, err = queue.Add(&chunk{Height: 3, Format: 1, Index: 4, Chunk: []byte{3, 1, 4}})
	require.NoError(t, err)
	assert.True(t, added)

	// Adding the first or last chunks again should return false
	added, err = queue.Add(&chunk{Height: 3, Format: 1, Index: 0, Chunk: []byte{3, 1, 0}})
	require.NoError(t, err)
	assert.False(t, added)

	added, err = queue.Add(&chunk{Height: 3, Format: 1, Index: 4, Chunk: []byte{3, 1, 4}})
	require.NoError(t, err)
	assert.False(t, added)

	// Adding the remaining chunks in reverse should be fine
	added, err = queue.Add(&chunk{Height: 3, Format: 1, Index: 3, Chunk: []byte{3, 1, 3}})
	require.NoError(t, err)
	assert.True(t, added)

	added, err = queue.Add(&chunk{Height: 3, Format: 1, Index: 2, Chunk: []byte{3, 1, 2}})
	require.NoError(t, err)
	assert.True(t, added)

	added, err = queue.Add(&chunk{Height: 3, Format: 1, Index: 1, Chunk: []byte{3, 1, 1}})
	require.NoError(t, err)
	assert.True(t, added)

	// At this point, we should be able to retrieve them all via Next
	for i := 0; i < 5; i++ {
		c, err := queue.Next()
		require.NoError(t, err)
		assert.Equal(t, &chunk{Height: 3, Format: 1, Index: uint32(i), Chunk: []byte{3, 1, byte(i)}}, c)
	}
	_, err = queue.Next()
	require.Error(t, err)
	assert.Equal(t, errDone, err)

	// It should still be possible to try to add chunks (which will be ignored)
	added, err = queue.Add(&chunk{Height: 3, Format: 1, Index: 0, Chunk: []byte{3, 1, 0}})
	require.NoError(t, err)
	assert.False(t, added)

	// After closing the queue it will also return false
	err = queue.Close()
	require.NoError(t, err)
	added, err = queue.Add(&chunk{Height: 3, Format: 1, Index: 0, Chunk: []byte{3, 1, 0}})
	require.NoError(t, err)
	assert.False(t, added)

	// Closing the queue again should also be fine
	err = queue.Close()
	require.NoError(t, err)
}

func TestChunkQueue_Add_ChunkErrors(t *testing.T) {
	testcases := map[string]struct {
		chunk *chunk
	}{
		"nil chunk":     {nil},
		"nil body":      {&chunk{Height: 3, Format: 1, Index: 0, Chunk: nil}},
		"wrong height":  {&chunk{Height: 9, Format: 1, Index: 0, Chunk: []byte{3, 1, 0}}},
		"wrong format":  {&chunk{Height: 3, Format: 9, Index: 0, Chunk: []byte{3, 1, 0}}},
		"invalid index": {&chunk{Height: 3, Format: 1, Index: 5, Chunk: []byte{3, 1, 0}}},
	}
	for name, tc := range testcases {
		t.Run(name, func(t *testing.T) {
			queue, teardown := setupChunkQueue(t)
			defer teardown()
			_, err := queue.Add(tc.chunk)
			require.Error(t, err)
		})
	}
}

func TestChunkQueue_Allocate(t *testing.T) {
	queue, teardown := setupChunkQueue(t)
	defer teardown()

	for i := uint32(0); i < queue.Size(); i++ {
		index, err := queue.Allocate()
		require.NoError(t, err)
		assert.EqualValues(t, i, index)
	}

	_, err := queue.Allocate()
	require.Error(t, err)
	assert.Equal(t, errDone, err)

	for i := uint32(0); i < queue.Size(); i++ {
		_, err = queue.Add(&chunk{Height: 3, Format: 1, Index: i, Chunk: []byte{byte(i)}})
		require.NoError(t, err)
	}

	// After all chunks have been allocated and retrieved, discarding a chunk will reallocate it.
	err = queue.Discard(2)
	require.NoError(t, err)

	index, err := queue.Allocate()
	require.NoError(t, err)
	assert.EqualValues(t, 2, index)
	_, err = queue.Allocate()
	require.Error(t, err)
	assert.Equal(t, errDone, err)

	// Discarding a chunk the closing the queue will return errDone.
	err = queue.Discard(2)
	require.NoError(t, err)
	err = queue.Close()
	require.NoError(t, err)
	_, err = queue.Allocate()
	require.Error(t, err)
	assert.Equal(t, errDone, err)
}

func TestChunkQueue_Discard(t *testing.T) {
	queue, teardown := setupChunkQueue(t)
	defer teardown()

	// Add a few chunks to the queue and fetch a couple
	_, err := queue.Add(&chunk{Height: 3, Format: 1, Index: 0, Chunk: []byte{byte(0)}})
	require.NoError(t, err)
	_, err = queue.Add(&chunk{Height: 3, Format: 1, Index: 1, Chunk: []byte{byte(1)}})
	require.NoError(t, err)
	_, err = queue.Add(&chunk{Height: 3, Format: 1, Index: 2, Chunk: []byte{byte(2)}})
	require.NoError(t, err)

	c, err := queue.Next()
	require.NoError(t, err)
	assert.EqualValues(t, 0, c.Index)
	c, err = queue.Next()
	require.NoError(t, err)
	assert.EqualValues(t, 1, c.Index)

	// Discarding the first chunk and re-adding it should cause it to be returned
	// immediately by Next(), before procceeding with chunk 2
	err = queue.Discard(0)
	require.NoError(t, err)
	added, err := queue.Add(&chunk{Height: 3, Format: 1, Index: 0, Chunk: []byte{byte(0)}})
	require.NoError(t, err)
	assert.True(t, added)
	c, err = queue.Next()
	require.NoError(t, err)
	assert.EqualValues(t, 0, c.Index)
	c, err = queue.Next()
	require.NoError(t, err)
	assert.EqualValues(t, 2, c.Index)

	// Discard then allocate, add and fetch all chunks
	for i := uint32(0); i < queue.Size(); i++ {
		err := queue.Discard(i)
		require.NoError(t, err)
	}
	for i := uint32(0); i < queue.Size(); i++ {
		_, err := queue.Allocate()
		require.NoError(t, err)
		_, err = queue.Add(&chunk{Height: 3, Format: 1, Index: i, Chunk: []byte{byte(i)}})
		require.NoError(t, err)
		c, err = queue.Next()
		require.NoError(t, err)
		assert.EqualValues(t, i, c.Index)
	}

	// Discarding a non-existent chunk does nothing.
	err = queue.Discard(99)
	require.NoError(t, err)

	// When discard a couple of chunks, we should be able to allocate, add, and fetch them again.
	err = queue.Discard(3)
	require.NoError(t, err)
	err = queue.Discard(1)
	require.NoError(t, err)

	index, err := queue.Allocate()
	require.NoError(t, err)
	assert.EqualValues(t, 1, index)
	index, err = queue.Allocate()
	require.NoError(t, err)
	assert.EqualValues(t, 3, index)

	added, err = queue.Add(&chunk{Height: 3, Format: 1, Index: 3, Chunk: []byte{3}})
	require.NoError(t, err)
	assert.True(t, added)
	added, err = queue.Add(&chunk{Height: 3, Format: 1, Index: 1, Chunk: []byte{1}})
	require.NoError(t, err)
	assert.True(t, added)

	chunk, err := queue.Next()
	require.NoError(t, err)
	assert.EqualValues(t, 1, chunk.Index)

	chunk, err = queue.Next()
	require.NoError(t, err)
	assert.EqualValues(t, 3, chunk.Index)

	_, err = queue.Next()
	require.Error(t, err)
	assert.Equal(t, errDone, err)

	// After closing the queue, discarding does nothing
	err = queue.Close()
	require.NoError(t, err)
	err = queue.Discard(2)
	require.NoError(t, err)
}

func TestChunkQueue_DiscardSender(t *testing.T) {
	queue, teardown := setupChunkQueue(t)
	defer teardown()

	// Allocate and add all chunks to the queue
	senders := []p2p.ID{"a", "b", "c"}
	for i := uint32(0); i < queue.Size(); i++ {
		_, err := queue.Allocate()
		require.NoError(t, err)
		_, err = queue.Add(&chunk{
			Height: 3,
			Format: 1,
			Index:  i,
			Chunk:  []byte{byte(i)},
			Sender: senders[int(i)%len(senders)],
		})
		require.NoError(t, err)
	}

	// Fetch the first three chunks
	for i := uint32(0); i < 3; i++ {
		_, err := queue.Next()
		require.NoError(t, err)
	}

	// Discarding an unknown sender should do nothing
	err := queue.DiscardSender("x")
	require.NoError(t, err)
	_, err = queue.Allocate()
	assert.Equal(t, errDone, err)

	// Discarding sender b should discard chunk 4, but not chunk 1 which has already been
	// returned.
	err = queue.DiscardSender("b")
	require.NoError(t, err)
	index, err := queue.Allocate()
	require.NoError(t, err)
	assert.EqualValues(t, 4, index)
	_, err = queue.Allocate()
	assert.Equal(t, errDone, err)
}

func TestChunkQueue_GetSender(t *testing.T) {
	queue, teardown := setupChunkQueue(t)
	defer teardown()

	_, err := queue.Add(&chunk{Height: 3, Format: 1, Index: 0, Chunk: []byte{1}, Sender: p2p.ID("a")})
	require.NoError(t, err)
	_, err = queue.Add(&chunk{Height: 3, Format: 1, Index: 1, Chunk: []byte{2}, Sender: p2p.ID("b")})
	require.NoError(t, err)

	assert.EqualValues(t, "a", queue.GetSender(0))
	assert.EqualValues(t, "b", queue.GetSender(1))
	assert.EqualValues(t, "", queue.GetSender(2))

	// After the chunk has been processed, we should still know who the sender was
	chunk, err := queue.Next()
	require.NoError(t, err)
	require.NotNil(t, chunk)
	require.EqualValues(t, 0, chunk.Index)
	assert.EqualValues(t, "a", queue.GetSender(0))
}

func TestChunkQueue_Next(t *testing.T) {
	queue, teardown := setupChunkQueue(t)
	defer teardown()

	// Next should block waiting for the next chunks, even when given out of order.
	chNext := make(chan *chunk, 10)
	go func() {
		for {
			c, err := queue.Next()
			if err == errDone {
				close(chNext)
				break
			}
			require.NoError(t, err)
			chNext <- c
		}
	}()

	assert.Empty(t, chNext)
	_, err := queue.Add(&chunk{Height: 3, Format: 1, Index: 1, Chunk: []byte{3, 1, 1}, Sender: p2p.ID("b")})
	require.NoError(t, err)
	select {
	case <-chNext:
		assert.Fail(t, "channel should be empty")
	default:
	}

	_, err = queue.Add(&chunk{Height: 3, Format: 1, Index: 0, Chunk: []byte{3, 1, 0}, Sender: p2p.ID("a")})
	require.NoError(t, err)

	assert.Equal(t,
		&chunk{Height: 3, Format: 1, Index: 0, Chunk: []byte{3, 1, 0}, Sender: p2p.ID("a")},
		<-chNext)
	assert.Equal(t,
		&chunk{Height: 3, Format: 1, Index: 1, Chunk: []byte{3, 1, 1}, Sender: p2p.ID("b")},
		<-chNext)

	_, err = queue.Add(&chunk{Height: 3, Format: 1, Index: 4, Chunk: []byte{3, 1, 4}, Sender: p2p.ID("e")})
	require.NoError(t, err)
	select {
	case <-chNext:
		assert.Fail(t, "channel should be empty")
	default:
	}

	_, err = queue.Add(&chunk{Height: 3, Format: 1, Index: 2, Chunk: []byte{3, 1, 2}, Sender: p2p.ID("c")})
	require.NoError(t, err)
	_, err = queue.Add(&chunk{Height: 3, Format: 1, Index: 3, Chunk: []byte{3, 1, 3}, Sender: p2p.ID("d")})
	require.NoError(t, err)

	assert.Equal(t,
		&chunk{Height: 3, Format: 1, Index: 2, Chunk: []byte{3, 1, 2}, Sender: p2p.ID("c")},
		<-chNext)
	assert.Equal(t,
		&chunk{Height: 3, Format: 1, Index: 3, Chunk: []byte{3, 1, 3}, Sender: p2p.ID("d")},
		<-chNext)
	assert.Equal(t,
		&chunk{Height: 3, Format: 1, Index: 4, Chunk: []byte{3, 1, 4}, Sender: p2p.ID("e")},
		<-chNext)

	_, ok := <-chNext
	assert.False(t, ok, "channel should be closed")

	// Calling next on a finished queue should return done
	_, err = queue.Next()
	assert.Equal(t, errDone, err)
}

func TestChunkQueue_Next_Closed(t *testing.T) {
	queue, teardown := setupChunkQueue(t)
	defer teardown()

	// Calling Next on a closed queue should return done
	_, err := queue.Add(&chunk{Height: 3, Format: 1, Index: 1, Chunk: []byte{3, 1, 1}})
	require.NoError(t, err)
	err = queue.Close()
	require.NoError(t, err)

	_, err = queue.Next()
	assert.Equal(t, errDone, err)
}

func TestChunkQueue_Retry(t *testing.T) {
	queue, teardown := setupChunkQueue(t)
	defer teardown()

	// Allocate and add all chunks to the queue
	for i := uint32(0); i < queue.Size(); i++ {
		_, err := queue.Allocate()
		require.NoError(t, err)
		_, err = queue.Add(&chunk{Height: 3, Format: 1, Index: i, Chunk: []byte{byte(i)}})
		require.NoError(t, err)
		_, err = queue.Next()
		require.NoError(t, err)
	}

	// Retrying a couple of chunks makes Next() return them, but they are not allocatable
	queue.Retry(3)
	queue.Retry(1)

	_, err := queue.Allocate()
	assert.Equal(t, errDone, err)

	chunk, err := queue.Next()
	require.NoError(t, err)
	assert.EqualValues(t, 1, chunk.Index)

	chunk, err = queue.Next()
	require.NoError(t, err)
	assert.EqualValues(t, 3, chunk.Index)

	_, err = queue.Next()
	assert.Equal(t, errDone, err)
}

func TestChunkQueue_RetryAll(t *testing.T) {
	queue, teardown := setupChunkQueue(t)
	defer teardown()

	// Allocate and add all chunks to the queue
	for i := uint32(0); i < queue.Size(); i++ {
		_, err := queue.Allocate()
		require.NoError(t, err)
		_, err = queue.Add(&chunk{Height: 3, Format: 1, Index: i, Chunk: []byte{byte(i)}})
		require.NoError(t, err)
		_, err = queue.Next()
		require.NoError(t, err)
	}

	_, err := queue.Next()
	assert.Equal(t, errDone, err)

	queue.RetryAll()

	_, err = queue.Allocate()
	assert.Equal(t, errDone, err)

	for i := uint32(0); i < queue.Size(); i++ {
		chunk, err := queue.Next()
		require.NoError(t, err)
		assert.EqualValues(t, i, chunk.Index)
	}

	_, err = queue.Next()
	assert.Equal(t, errDone, err)
}

func TestChunkQueue_Size(t *testing.T) {
	queue, teardown := setupChunkQueue(t)
	defer teardown()

	assert.EqualValues(t, 5, queue.Size())

	err := queue.Close()
	require.NoError(t, err)
	assert.EqualValues(t, 0, queue.Size())
}

func TestChunkQueue_WaitFor(t *testing.T) {
	queue, teardown := setupChunkQueue(t)
	defer teardown()

	waitFor1 := queue.WaitFor(1)
	waitFor4 := queue.WaitFor(4)

	// Adding 0 and 2 should not trigger waiters
	_, err := queue.Add(&chunk{Height: 3, Format: 1, Index: 0, Chunk: []byte{3, 1, 0}})
	require.NoError(t, err)
	_, err = queue.Add(&chunk{Height: 3, Format: 1, Index: 2, Chunk: []byte{3, 1, 2}})
	require.NoError(t, err)
	select {
	case <-waitFor1:
		require.Fail(t, "WaitFor(1) should not trigger on 0 or 2")
	case <-waitFor4:
		require.Fail(t, "WaitFor(4) should not trigger on 0 or 2")
	default:
	}

	// Adding 1 should trigger WaitFor(1), but not WaitFor(4). The channel should be closed.
	_, err = queue.Add(&chunk{Height: 3, Format: 1, Index: 1, Chunk: []byte{3, 1, 1}})
	require.NoError(t, err)
	assert.EqualValues(t, 1, <-waitFor1)
	_, ok := <-waitFor1
	assert.False(t, ok)
	select {
	case <-waitFor4:
		require.Fail(t, "WaitFor(4) should not trigger on 0 or 2")
	default:
	}

	// Fetch the first chunk. At this point, waiting for either 0 (retrieved from pool) or 1
	// (queued in pool) should immediately return true.
	c, err := queue.Next()
	require.NoError(t, err)
	assert.EqualValues(t, 0, c.Index)

	w := queue.WaitFor(0)
	assert.EqualValues(t, 0, <-w)
	_, ok = <-w
	assert.False(t, ok)

	w = queue.WaitFor(1)
	assert.EqualValues(t, 1, <-w)
	_, ok = <-w
	assert.False(t, ok)

	// Close the queue. This should cause the waiter for 4 to close, and also cause any future
	// waiters to get closed channels.
	err = queue.Close()
	require.NoError(t, err)
	_, ok = <-waitFor4
	assert.False(t, ok)

	w = queue.WaitFor(3)
	_, ok = <-w
	assert.False(t, ok)
}
