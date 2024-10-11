package stategen

import (
	"sync"

	lru "github.com/hashicorp/golang-lru"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	lruwrpr "github.com/prysmaticlabs/prysm/v4/cache/lru"
)

var (
	// hotStateCacheSize defines the max number of hot state this can cache.
	hotStateCacheSize = 32
	// Metrics
	hotStateCacheHit = promauto.NewCounter(prometheus.CounterOpts{
		Name: "hot_state_cache_hit",
		Help: "The total number of cache hits on the hot state cache.",
	})
	hotStateCacheMiss = promauto.NewCounter(prometheus.CounterOpts{
		Name: "hot_state_cache_miss",
		Help: "The total number of cache misses on the hot state cache.",
	})
)

// hotStateCache is used to store the processed beacon state after finalized check point.
type hotStateCache struct {
	cache *lru.Cache
	lock  sync.RWMutex
}

// newHotStateCache initializes the map and underlying cache.
func newHotStateCache() *hotStateCache {
	return &hotStateCache{
		cache: lruwrpr.New(hotStateCacheSize),
	}
}

// Get returns a cached response via input block root, if any.
// The response is copied by default.
func (c *hotStateCache) get(blockRoot [32]byte) state.BeaconState {
	c.lock.RLock()
	defer c.lock.RUnlock()
	item, exists := c.cache.Get(blockRoot)

	if exists && item != nil {
		hotStateCacheHit.Inc()
		return item.(state.BeaconState).Copy()
	}
	hotStateCacheMiss.Inc()
	return nil
}

func (c *hotStateCache) ByBlockRoot(r [32]byte) (state.BeaconState, error) {
	st := c.get(r)
	if st == nil {
		return nil, ErrNotInCache
	}
	return st, nil
}

// GetWithoutCopy returns a non-copied cached response via input block root.
func (c *hotStateCache) getWithoutCopy(blockRoot [32]byte) state.BeaconState {
	c.lock.RLock()
	defer c.lock.RUnlock()
	item, exists := c.cache.Get(blockRoot)
	if exists && item != nil {
		hotStateCacheHit.Inc()
		return item.(state.BeaconState)
	}
	hotStateCacheMiss.Inc()
	return nil
}

// put the response in the cache.
func (c *hotStateCache) put(blockRoot [32]byte, state state.BeaconState) {
	c.lock.Lock()
	defer c.lock.Unlock()
	c.cache.Add(blockRoot, state)
}

// has returns true if the key exists in the cache.
func (c *hotStateCache) has(blockRoot [32]byte) bool {
	c.lock.RLock()
	defer c.lock.RUnlock()
	return c.cache.Contains(blockRoot)
}

// delete deletes the key exists in the cache.
func (c *hotStateCache) delete(blockRoot [32]byte) bool {
	c.lock.Lock()
	defer c.lock.Unlock()
	return c.cache.Remove(blockRoot)
}
