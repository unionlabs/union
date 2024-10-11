// Copyright 2020 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package downloader

import (
	"fmt"
	"sync"
	"sync/atomic"

	"github.com/ethereum/go-ethereum/core/types"
)

// resultStore implements a structure for maintaining fetchResults, tracking their
// download-progress and delivering (finished) results.
type resultStore struct {
	items        []*fetchResult // Downloaded but not yet delivered fetch results
	resultOffset uint64         // Offset of the first cached fetch result in the block chain

	// Internal index of first non-completed entry, updated atomically when needed.
	// If all items are complete, this will equal length(items), so
	// *important* : is not safe to use for indexing without checking against length
	indexIncomplete atomic.Int32

	// throttleThreshold is the limit up to which we _want_ to fill the
	// results. If blocks are large, we want to limit the results to less
	// than the number of available slots, and maybe only fill 1024 out of
	// 8192 possible places. The queue will, at certain times, recalibrate
	// this index.
	throttleThreshold uint64

	lock sync.RWMutex
}

func newResultStore(size int) *resultStore {
	return &resultStore{
		resultOffset:      0,
		items:             make([]*fetchResult, size),
		throttleThreshold: uint64(size),
	}
}

// SetThrottleThreshold updates the throttling threshold based on the requested
// limit and the total queue capacity. It returns the (possibly capped) threshold
func (r *resultStore) SetThrottleThreshold(threshold uint64) uint64 {
	r.lock.Lock()
	defer r.lock.Unlock()

	limit := uint64(len(r.items))
	if threshold >= limit {
		threshold = limit
	}
	r.throttleThreshold = threshold
	return r.throttleThreshold
}

// AddFetch adds a header for body/receipt fetching. This is used when the queue
// wants to reserve headers for fetching.
//
// It returns the following:
//
//	stale     - if true, this item is already passed, and should not be requested again
//	throttled - if true, the store is at capacity, this particular header is not prio now
//	item      - the result to store data into
//	err       - any error that occurred
func (r *resultStore) AddFetch(header *types.Header, fastSync bool) (stale, throttled bool, item *fetchResult, err error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	var index int
	item, index, stale, throttled, err = r.getFetchResult(header.Number.Uint64())
	if err != nil || stale || throttled {
		return stale, throttled, item, err
	}
	if item == nil {
		item = newFetchResult(header, fastSync)
		r.items[index] = item
	}
	return stale, throttled, item, err
}

// GetDeliverySlot returns the fetchResult for the given header. If the 'stale' flag
// is true, that means the header has already been delivered 'upstream'. This method
// does not bubble up the 'throttle' flag, since it's moot at the point in time when
// the item is downloaded and ready for delivery
func (r *resultStore) GetDeliverySlot(headerNumber uint64) (*fetchResult, bool, error) {
	r.lock.RLock()
	defer r.lock.RUnlock()

	res, _, stale, _, err := r.getFetchResult(headerNumber)
	return res, stale, err
}

// getFetchResult returns the fetchResult corresponding to the given item, and
// the index where the result is stored.
func (r *resultStore) getFetchResult(headerNumber uint64) (item *fetchResult, index int, stale, throttle bool, err error) {
	index = int(int64(headerNumber) - int64(r.resultOffset))
	throttle = index >= int(r.throttleThreshold)
	stale = index < 0

	if index >= len(r.items) {
		err = fmt.Errorf("%w: index allocation went beyond available resultStore space "+
			"(index [%d] = header [%d] - resultOffset [%d], len(resultStore) = %d", errInvalidChain,
			index, headerNumber, r.resultOffset, len(r.items))
		return nil, index, stale, throttle, err
	}
	if stale {
		return nil, index, stale, throttle, nil
	}
	item = r.items[index]
	return item, index, stale, throttle, nil
}

// HasCompletedItems returns true if there are processable items available
// this method is cheaper than countCompleted
func (r *resultStore) HasCompletedItems() bool {
	r.lock.RLock()
	defer r.lock.RUnlock()

	if len(r.items) == 0 {
		return false
	}
	if item := r.items[0]; item != nil && item.AllDone() {
		return true
	}
	return false
}

// countCompleted returns the number of items ready for delivery, stopping at
// the first non-complete item.
//
// The mthod assumes (at least) rlock is held.
func (r *resultStore) countCompleted() int {
	// We iterate from the already known complete point, and see
	// if any more has completed since last count
	index := r.indexIncomplete.Load()
	for ; ; index++ {
		if index >= int32(len(r.items)) {
			break
		}
		result := r.items[index]
		if result == nil || !result.AllDone() {
			break
		}
	}
	r.indexIncomplete.Store(index)
	return int(index)
}

// GetCompleted returns the next batch of completed fetchResults
func (r *resultStore) GetCompleted(limit int) []*fetchResult {
	r.lock.Lock()
	defer r.lock.Unlock()

	completed := r.countCompleted()
	if limit > completed {
		limit = completed
	}
	results := make([]*fetchResult, limit)
	copy(results, r.items[:limit])

	// Delete the results from the cache and clear the tail.
	copy(r.items, r.items[limit:])
	for i := len(r.items) - limit; i < len(r.items); i++ {
		r.items[i] = nil
	}
	// Advance the expected block number of the first cache entry
	r.resultOffset += uint64(limit)
	r.indexIncomplete.Add(int32(-limit))

	return results
}

// Prepare initialises the offset with the given block number
func (r *resultStore) Prepare(offset uint64) {
	r.lock.Lock()
	defer r.lock.Unlock()

	if r.resultOffset < offset {
		r.resultOffset = offset
	}
}
