package slasher

import (
	"sync"

	slashertypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/slasher/types"
)

// Struct for handling a thread-safe list of indexed attestation wrappers.
type attestationsQueue struct {
	sync.RWMutex
	items []*slashertypes.IndexedAttestationWrapper
}

// Struct for handling a thread-safe list of beacon block header wrappers.
type blocksQueue struct {
	lock  sync.RWMutex
	items []*slashertypes.SignedBlockHeaderWrapper
}

func newAttestationsQueue() *attestationsQueue {
	return &attestationsQueue{
		items: make([]*slashertypes.IndexedAttestationWrapper, 0),
	}
}

func newBlocksQueue() *blocksQueue {
	return &blocksQueue{
		items: make([]*slashertypes.SignedBlockHeaderWrapper, 0),
	}
}

func (q *attestationsQueue) push(att *slashertypes.IndexedAttestationWrapper) {
	q.Lock()
	defer q.Unlock()
	q.items = append(q.items, att)
}

func (q *attestationsQueue) dequeue() []*slashertypes.IndexedAttestationWrapper {
	q.Lock()
	defer q.Unlock()
	items := q.items
	q.items = make([]*slashertypes.IndexedAttestationWrapper, 0)
	return items
}

func (q *attestationsQueue) size() int {
	q.RLock()
	defer q.RUnlock()
	return len(q.items)
}

func (q *attestationsQueue) extend(atts []*slashertypes.IndexedAttestationWrapper) {
	q.Lock()
	defer q.Unlock()
	q.items = append(q.items, atts...)
}

func (q *blocksQueue) push(blk *slashertypes.SignedBlockHeaderWrapper) {
	q.lock.Lock()
	defer q.lock.Unlock()
	q.items = append(q.items, blk)
}

func (q *blocksQueue) dequeue() []*slashertypes.SignedBlockHeaderWrapper {
	q.lock.Lock()
	defer q.lock.Unlock()
	items := q.items
	q.items = make([]*slashertypes.SignedBlockHeaderWrapper, 0)
	return items
}

func (q *blocksQueue) size() int {
	q.lock.RLock()
	defer q.lock.RUnlock()
	return len(q.items)
}

func (q *blocksQueue) extend(blks []*slashertypes.SignedBlockHeaderWrapper) {
	q.lock.Lock()
	defer q.lock.Unlock()
	q.items = append(q.items, blks...)
}
