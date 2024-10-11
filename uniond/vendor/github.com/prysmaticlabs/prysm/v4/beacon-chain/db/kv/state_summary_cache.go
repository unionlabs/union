package kv

import (
	"sync"

	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

const stateSummaryCachePruneCount = 128

// stateSummaryCache caches state summary object.
type stateSummaryCache struct {
	initSyncStateSummaries     map[[32]byte]*ethpb.StateSummary
	initSyncStateSummariesLock sync.RWMutex
}

// newStateSummaryCache creates a new state summary cache.
func newStateSummaryCache() *stateSummaryCache {
	return &stateSummaryCache{
		initSyncStateSummaries: make(map[[32]byte]*ethpb.StateSummary),
	}
}

// put saves a state summary to the initial sync state summaries cache.
func (c *stateSummaryCache) put(r [32]byte, b *ethpb.StateSummary) {
	c.initSyncStateSummariesLock.Lock()
	defer c.initSyncStateSummariesLock.Unlock()
	c.initSyncStateSummaries[r] = b
}

// has checks if a state summary exists in the initial sync state summaries cache using the root
// of the block.
func (c *stateSummaryCache) has(r [32]byte) bool {
	c.initSyncStateSummariesLock.RLock()
	defer c.initSyncStateSummariesLock.RUnlock()
	_, ok := c.initSyncStateSummaries[r]
	return ok
}

// delete state summary in cache.
func (c *stateSummaryCache) delete(r [32]byte) {
	c.initSyncStateSummariesLock.Lock()
	defer c.initSyncStateSummariesLock.Unlock()
	delete(c.initSyncStateSummaries, r)
}

// get retrieves a state summary from the initial sync state summaries cache using the root of
// the block.
func (c *stateSummaryCache) get(r [32]byte) *ethpb.StateSummary {
	c.initSyncStateSummariesLock.RLock()
	defer c.initSyncStateSummariesLock.RUnlock()
	b := c.initSyncStateSummaries[r]
	return b
}

// len retrieves the state summary count from the state summaries cache.
func (c *stateSummaryCache) len() int {
	c.initSyncStateSummariesLock.RLock()
	defer c.initSyncStateSummariesLock.RUnlock()
	return len(c.initSyncStateSummaries)
}

// GetAll retrieves all the beacon state summaries from the initial sync state summaries cache, the returned
// state summaries are unordered.
func (c *stateSummaryCache) getAll() []*ethpb.StateSummary {
	c.initSyncStateSummariesLock.RLock()
	defer c.initSyncStateSummariesLock.RUnlock()

	summaries := make([]*ethpb.StateSummary, 0, len(c.initSyncStateSummaries))
	for _, b := range c.initSyncStateSummaries {
		summaries = append(summaries, b)
	}
	return summaries
}

// Clear clears out the initial sync state summaries cache.
func (c *stateSummaryCache) clear() {
	c.initSyncStateSummariesLock.Lock()
	defer c.initSyncStateSummariesLock.Unlock()
	c.initSyncStateSummaries = make(map[[32]byte]*ethpb.StateSummary)
}
