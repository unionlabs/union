package statesync

import (
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"time"

	cmtsync "github.com/cometbft/cometbft/libs/sync"
	"github.com/cometbft/cometbft/p2p"
)

// errDone is returned by chunkQueue.Next() when all chunks have been returned.
var errDone = errors.New("chunk queue has completed")

// chunk contains data for a chunk.
type chunk struct {
	Height uint64
	Format uint32
	Index  uint32
	Chunk  []byte
	Sender p2p.ID
}

// chunkQueue manages chunks for a state sync process, ordering them if requested. It acts as an
// iterator over all chunks, but callers can request chunks to be retried, optionally after
// refetching.
type chunkQueue struct {
	cmtsync.Mutex
	snapshot       *snapshot                  // if this is nil, the queue has been closed
	dir            string                     // temp dir for on-disk chunk storage
	chunkFiles     map[uint32]string          // path to temporary chunk file
	chunkSenders   map[uint32]p2p.ID          // the peer who sent the given chunk
	chunkAllocated map[uint32]bool            // chunks that have been allocated via Allocate()
	chunkReturned  map[uint32]bool            // chunks returned via Next()
	waiters        map[uint32][]chan<- uint32 // signals WaitFor() waiters about chunk arrival
}

// newChunkQueue creates a new chunk queue for a snapshot, using a temp dir for storage.
// Callers must call Close() when done.
func newChunkQueue(snapshot *snapshot, tempDir string) (*chunkQueue, error) {
	dir, err := os.MkdirTemp(tempDir, "tm-statesync")
	if err != nil {
		return nil, fmt.Errorf("unable to create temp dir for state sync chunks: %w", err)
	}
	if snapshot.Chunks == 0 {
		return nil, errors.New("snapshot has no chunks")
	}
	return &chunkQueue{
		snapshot:       snapshot,
		dir:            dir,
		chunkFiles:     make(map[uint32]string, snapshot.Chunks),
		chunkSenders:   make(map[uint32]p2p.ID, snapshot.Chunks),
		chunkAllocated: make(map[uint32]bool, snapshot.Chunks),
		chunkReturned:  make(map[uint32]bool, snapshot.Chunks),
		waiters:        make(map[uint32][]chan<- uint32),
	}, nil
}

// Add adds a chunk to the queue. It ignores chunks that already exist, returning false.
func (q *chunkQueue) Add(chunk *chunk) (bool, error) {
	if chunk == nil || chunk.Chunk == nil {
		return false, errors.New("cannot add nil chunk")
	}
	q.Lock()
	defer q.Unlock()
	if q.snapshot == nil {
		return false, nil // queue is closed
	}
	if chunk.Height != q.snapshot.Height {
		return false, fmt.Errorf("invalid chunk height %v, expected %v", chunk.Height, q.snapshot.Height)
	}
	if chunk.Format != q.snapshot.Format {
		return false, fmt.Errorf("invalid chunk format %v, expected %v", chunk.Format, q.snapshot.Format)
	}
	if chunk.Index >= q.snapshot.Chunks {
		return false, fmt.Errorf("received unexpected chunk %v", chunk.Index)
	}
	if q.chunkFiles[chunk.Index] != "" {
		return false, nil
	}

	path := filepath.Join(q.dir, strconv.FormatUint(uint64(chunk.Index), 10))
	err := os.WriteFile(path, chunk.Chunk, 0600)
	if err != nil {
		return false, fmt.Errorf("failed to save chunk %v to file %v: %w", chunk.Index, path, err)
	}
	q.chunkFiles[chunk.Index] = path
	q.chunkSenders[chunk.Index] = chunk.Sender

	// Signal any waiters that the chunk has arrived.
	for _, waiter := range q.waiters[chunk.Index] {
		waiter <- chunk.Index
		close(waiter)
	}
	delete(q.waiters, chunk.Index)

	return true, nil
}

// Allocate allocates a chunk to the caller, making it responsible for fetching it. Returns
// errDone once no chunks are left or the queue is closed.
func (q *chunkQueue) Allocate() (uint32, error) {
	q.Lock()
	defer q.Unlock()
	if q.snapshot == nil {
		return 0, errDone
	}
	if uint32(len(q.chunkAllocated)) >= q.snapshot.Chunks {
		return 0, errDone
	}
	for i := uint32(0); i < q.snapshot.Chunks; i++ {
		if !q.chunkAllocated[i] {
			q.chunkAllocated[i] = true
			return i, nil
		}
	}
	return 0, errDone
}

// Close closes the chunk queue, cleaning up all temporary files.
func (q *chunkQueue) Close() error {
	q.Lock()
	defer q.Unlock()
	if q.snapshot == nil {
		return nil
	}
	for _, waiters := range q.waiters {
		for _, waiter := range waiters {
			close(waiter)
		}
	}
	q.waiters = nil
	q.snapshot = nil
	err := os.RemoveAll(q.dir)
	if err != nil {
		return fmt.Errorf("failed to clean up state sync tempdir %v: %w", q.dir, err)
	}
	return nil
}

// Discard discards a chunk. It will be removed from the queue, available for allocation, and can
// be added and returned via Next() again. If the chunk is not already in the queue this does
// nothing, to avoid it being allocated to multiple fetchers.
func (q *chunkQueue) Discard(index uint32) error {
	q.Lock()
	defer q.Unlock()
	return q.discard(index)
}

// discard discards a chunk, scheduling it for refetching. The caller must hold the mutex lock.
func (q *chunkQueue) discard(index uint32) error {
	if q.snapshot == nil {
		return nil
	}
	path := q.chunkFiles[index]
	if path == "" {
		return nil
	}
	err := os.Remove(path)
	if err != nil {
		return fmt.Errorf("failed to remove chunk %v: %w", index, err)
	}
	delete(q.chunkFiles, index)
	delete(q.chunkReturned, index)
	delete(q.chunkAllocated, index)
	return nil
}

// DiscardSender discards all *unreturned* chunks from a given sender. If the caller wants to
// discard already returned chunks, this can be done via Discard().
func (q *chunkQueue) DiscardSender(peerID p2p.ID) error {
	q.Lock()
	defer q.Unlock()

	for index, sender := range q.chunkSenders {
		if sender == peerID && !q.chunkReturned[index] {
			err := q.discard(index)
			if err != nil {
				return err
			}
			delete(q.chunkSenders, index)
		}
	}
	return nil
}

// GetSender returns the sender of the chunk with the given index, or empty if not found.
func (q *chunkQueue) GetSender(index uint32) p2p.ID {
	q.Lock()
	defer q.Unlock()
	return q.chunkSenders[index]
}

// Has checks whether a chunk exists in the queue.
func (q *chunkQueue) Has(index uint32) bool {
	q.Lock()
	defer q.Unlock()
	return q.chunkFiles[index] != ""
}

// load loads a chunk from disk, or nil if the chunk is not in the queue. The caller must hold the
// mutex lock.
func (q *chunkQueue) load(index uint32) (*chunk, error) {
	path, ok := q.chunkFiles[index]
	if !ok {
		return nil, nil
	}
	body, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("failed to load chunk %v: %w", index, err)
	}
	return &chunk{
		Height: q.snapshot.Height,
		Format: q.snapshot.Format,
		Index:  index,
		Chunk:  body,
		Sender: q.chunkSenders[index],
	}, nil
}

// Next returns the next chunk from the queue, or errDone if all chunks have been returned. It
// blocks until the chunk is available. Concurrent Next() calls may return the same chunk.
func (q *chunkQueue) Next() (*chunk, error) {
	q.Lock()
	var chunk *chunk
	index, err := q.nextUp()
	if err == nil {
		chunk, err = q.load(index)
		if err == nil {
			q.chunkReturned[index] = true
		}
	}
	q.Unlock()
	if chunk != nil || err != nil {
		return chunk, err
	}

	select {
	case _, ok := <-q.WaitFor(index):
		if !ok {
			return nil, errDone // queue closed
		}
	case <-time.After(chunkTimeout):
		return nil, errTimeout
	}

	q.Lock()
	defer q.Unlock()
	chunk, err = q.load(index)
	if err != nil {
		return nil, err
	}
	q.chunkReturned[index] = true
	return chunk, nil
}

// nextUp returns the next chunk to be returned, or errDone if all chunks have been returned. The
// caller must hold the mutex lock.
func (q *chunkQueue) nextUp() (uint32, error) {
	if q.snapshot == nil {
		return 0, errDone
	}
	for i := uint32(0); i < q.snapshot.Chunks; i++ {
		if !q.chunkReturned[i] {
			return i, nil
		}
	}
	return 0, errDone
}

// Retry schedules a chunk to be retried, without refetching it.
func (q *chunkQueue) Retry(index uint32) {
	q.Lock()
	defer q.Unlock()
	delete(q.chunkReturned, index)
}

// RetryAll schedules all chunks to be retried, without refetching them.
func (q *chunkQueue) RetryAll() {
	q.Lock()
	defer q.Unlock()
	q.chunkReturned = make(map[uint32]bool)
}

// Size returns the total number of chunks for the snapshot and queue, or 0 when closed.
func (q *chunkQueue) Size() uint32 {
	q.Lock()
	defer q.Unlock()
	if q.snapshot == nil {
		return 0
	}
	return q.snapshot.Chunks
}

// WaitFor returns a channel that receives a chunk index when it arrives in the queue, or
// immediately if it has already arrived. The channel is closed without a value if the queue is
// closed or if the chunk index is not valid.
func (q *chunkQueue) WaitFor(index uint32) <-chan uint32 {
	q.Lock()
	defer q.Unlock()
	ch := make(chan uint32, 1)
	switch {
	case q.snapshot == nil:
		close(ch)
	case index >= q.snapshot.Chunks:
		close(ch)
	case q.chunkFiles[index] != "":
		ch <- index
		close(ch)
	default:
		if q.waiters[index] == nil {
			q.waiters[index] = make([]chan<- uint32, 0)
		}
		q.waiters[index] = append(q.waiters[index], ch)
	}
	return ch
}
