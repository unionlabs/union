package cache

import (
	"fmt"

	lru "github.com/hashicorp/golang-lru"

	"cosmossdk.io/store/cachekv"
	"cosmossdk.io/store/types"
)

var (
	_ types.CommitKVStore             = (*CommitKVStoreCache)(nil)
	_ types.MultiStorePersistentCache = (*CommitKVStoreCacheManager)(nil)

	// DefaultCommitKVStoreCacheSize defines the persistent ARC cache size for a
	// CommitKVStoreCache.
	DefaultCommitKVStoreCacheSize uint = 1000
)

type (
	// CommitKVStoreCache implements an inter-block (persistent) cache that wraps a
	// CommitKVStore. Reads first hit the internal ARC (Adaptive Replacement Cache).
	// During a cache miss, the read is delegated to the underlying CommitKVStore
	// and cached. Deletes and writes always happen to both the cache and the
	// CommitKVStore in a write-through manner. Caching performed in the
	// CommitKVStore and below is completely irrelevant to this layer.
	CommitKVStoreCache struct {
		types.CommitKVStore
		cache *lru.ARCCache
	}

	// CommitKVStoreCacheManager maintains a mapping from a StoreKey to a
	// CommitKVStoreCache. Each CommitKVStore, per StoreKey, is meant to be used
	// in an inter-block (persistent) manner and typically provided by a
	// CommitMultiStore.
	CommitKVStoreCacheManager struct {
		cacheSize uint
		caches    map[string]types.CommitKVStore
	}
)

func NewCommitKVStoreCache(store types.CommitKVStore, size uint) *CommitKVStoreCache {
	cache, err := lru.NewARC(int(size))
	if err != nil {
		panic(fmt.Errorf("failed to create KVStore cache: %s", err))
	}

	return &CommitKVStoreCache{
		CommitKVStore: store,
		cache:         cache,
	}
}

func NewCommitKVStoreCacheManager(size uint) *CommitKVStoreCacheManager {
	return &CommitKVStoreCacheManager{
		cacheSize: size,
		caches:    make(map[string]types.CommitKVStore),
	}
}

// GetStoreCache returns a Cache from the CommitStoreCacheManager for a given
// StoreKey. If no Cache exists for the StoreKey, then one is created and set.
// The returned Cache is meant to be used in a persistent manner.
func (cmgr *CommitKVStoreCacheManager) GetStoreCache(key types.StoreKey, store types.CommitKVStore) types.CommitKVStore {
	if cmgr.caches[key.Name()] == nil {
		cmgr.caches[key.Name()] = NewCommitKVStoreCache(store, cmgr.cacheSize)
	}

	return cmgr.caches[key.Name()]
}

// Unwrap returns the underlying CommitKVStore for a given StoreKey.
func (cmgr *CommitKVStoreCacheManager) Unwrap(key types.StoreKey) types.CommitKVStore {
	if ckv, ok := cmgr.caches[key.Name()]; ok {
		return ckv.(*CommitKVStoreCache).CommitKVStore
	}

	return nil
}

// Reset resets in the internal caches.
func (cmgr *CommitKVStoreCacheManager) Reset() {
	// Clear the map.
	// Please note that we are purposefully using the map clearing idiom.
	// See https://github.com/cosmos/cosmos-sdk/issues/6681.
	for key := range cmgr.caches {
		delete(cmgr.caches, key)
	}
}

// CacheWrap implements the CacheWrapper interface
func (ckv *CommitKVStoreCache) CacheWrap() types.CacheWrap {
	return cachekv.NewStore(ckv)
}

// Get retrieves a value by key. It will first look in the write-through cache.
// If the value doesn't exist in the write-through cache, the query is delegated
// to the underlying CommitKVStore.
func (ckv *CommitKVStoreCache) Get(key []byte) []byte {
	types.AssertValidKey(key)

	keyStr := string(key)
	valueI, ok := ckv.cache.Get(keyStr)
	if ok {
		// cache hit
		return valueI.([]byte)
	}

	// cache miss; write to cache
	value := ckv.CommitKVStore.Get(key)
	ckv.cache.Add(keyStr, value)

	return value
}

// Set inserts a key/value pair into both the write-through cache and the
// underlying CommitKVStore.
func (ckv *CommitKVStoreCache) Set(key, value []byte) {
	types.AssertValidKey(key)
	types.AssertValidValue(value)

	ckv.cache.Add(string(key), value)
	ckv.CommitKVStore.Set(key, value)
}

// Delete removes a key/value pair from both the write-through cache and the
// underlying CommitKVStore.
func (ckv *CommitKVStoreCache) Delete(key []byte) {
	ckv.cache.Remove(string(key))
	ckv.CommitKVStore.Delete(key)
}
