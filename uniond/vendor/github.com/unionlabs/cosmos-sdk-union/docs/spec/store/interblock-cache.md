# Inter-block Cache

* [Inter-block Cache](#inter-block-cache)
    * [Synopsis](#synopsis)
    * [Overview and basic concepts](#overview-and-basic-concepts)
        * [Motivation](#motivation)
        * [Definitions](#definitions)
    * [System model and properties](#system-model-and-properties)
        * [Assumptions](#assumptions)
        * [Properties](#properties)
            * [Thread safety](#thread-safety)
            * [Crash recovery](#crash-recovery)
            * [Iteration](#iteration)
    * [Technical specification](#technical-specification)
        * [General design](#general-design)
        * [API](#api)
            * [CommitKVCacheManager](#commitkvcachemanager)
            * [CommitKVStoreCache](#commitkvstorecache)
        * [Implementation details](#implementation-details)
    * [History](#history)
    * [Copyright](#copyright)

## Synopsis

The inter-block cache is an in-memory cache storing (in-most-cases) immutable state that modules need to read in between blocks. When enabled, all sub-stores of a multi store, e.g., `rootmulti`, are wrapped.

## Overview and basic concepts

### Motivation

The goal of the inter-block cache is to allow SDK modules to have fast access to data that it is typically queried during the execution of every block. This is data that do not change often, e.g. module parameters. The inter-block cache wraps each `CommitKVStore` of a multi store such as `rootmulti` with a fixed size, write-through cache. Caches are not cleared after a block is committed, as opposed to other caching layers such as `cachekv`.

### Definitions

* `Store key` uniquely identifies a store.
* `KVCache` is a `CommitKVStore` wrapped with a cache.
* `Cache manager` is a key component of the inter-block cache responsible for maintaining a map from `store keys` to `KVCaches`.

## System model and properties

### Assumptions

This specification assumes that there exists a cache implementation accessible to the inter-block cache feature.

> The implementation uses adaptive replacement cache (ARC), an enhancement over the standard last-recently-used (LRU) cache in that tracks both frequency and recency of use.

The inter-block cache requires that the cache implementation to provide methods to create a cache, add a key/value pair, remove a key/value pair and retrieve the value associated to a key. In this specification, we assume that a `Cache` feature offers this functionality through the following methods:

* `NewCache(size int)` creates a new cache with `size` capacity and returns it.
* `Get(key string)` attempts to retrieve a key/value pair from `Cache.` It returns `(value []byte, success bool)`. If `Cache` contains the key, it `value` contains the associated value and `success=true`. Otherwise, `success=false` and `value` should be ignored.
* `Add(key string, value []byte)` inserts a key/value pair into the `Cache`.
* `Remove(key string)` removes the key/value pair identified by `key` from `Cache`.

The specification also assumes that `CommitKVStore` offers the following API:

* `Get(key string)` attempts to retrieve a key/value pair from `CommitKVStore`.
* `Set(key, string, value []byte)` inserts a key/value pair into the `CommitKVStore`.
* `Delete(key string)` removes the key/value pair identified by `key` from `CommitKVStore`.

> Ideally, both `Cache` and `CommitKVStore` should be specified in a different document and referenced here.

### Properties

#### Thread safety

Accessing the `cache manager` or a `KVCache` is not thread-safe: no method is guarded with a lock.
Note that this is true even if the cache implementation is thread-safe.

> For instance, assume that two `Set` operations are executed concurrently on the same key, each writing a different value. After both are executed, the cache and the underlying store may be inconsistent, each storing a different value under the same key.

#### Crash recovery

The inter-block cache transparently delegates `Commit()` to its aggregate `CommitKVStore`. If the 
aggregate `CommitKVStore` supports atomic writes and use them to guarantee that the store is always in a consistent state in disk, the inter-block cache can be transparently moved to a consistent state when a failure occurs.

> Note that this is the case for `IAVLStore`, the preferred `CommitKVStore`. On commit, it calls `SaveVersion()` on the underlying `MutableTree`. `SaveVersion` writes to disk are atomic via batching. This means that only consistent versions of the store (the tree) are written to the disk. Thus, in case of a failure during a `SaveVersion` call, on recovery from disk, the version of the store will be consistent.

#### Iteration

Iteration over each wrapped store is supported via the embedded `CommitKVStore` interface.

## Technical specification

### General design

The inter-block cache feature is composed by two components: `CommitKVCacheManager` and `CommitKVCache`.

`CommitKVCacheManager` implements the cache manager. It maintains a mapping from a store key to a `KVStore`.

```go
type CommitKVStoreCacheManager interface{
    cacheSize uint
    caches map[string]CommitKVStore
}
```

`CommitKVStoreCache` implements a `KVStore`: a write-through cache that wraps a `CommitKVStore`. This means that deletes and writes always happen to both the cache and the underlying `CommitKVStore`. Reads on the other hand first hit the internal cache. During a cache miss, the read is delegated to the underlying `CommitKVStore` and cached.

```go
type CommitKVStoreCache interface{
    store CommitKVStore
    cache Cache
}
```

To enable inter-block cache on `rootmulti`, one needs to instantiate a `CommitKVCacheManager` and set it by calling `SetInterBlockCache()` before calling one of `LoadLatestVersion()`, `LoadLatestVersionAndUpgrade(...)`, `LoadVersionAndUpgrade(...)` and `LoadVersion(version)`.

### API

#### CommitKVCacheManager

The method `NewCommitKVStoreCacheManager` creates a new cache manager and returns it.

| Name  | Type | Description |
| ------------- | ---------|------- |
| size  | integer | Determines the capacity of each of the KVCache maintained by the manager |

```go
func NewCommitKVStoreCacheManager(size uint) CommitKVStoreCacheManager {
    manager = CommitKVStoreCacheManager{size, make(map[string]CommitKVStore)}
    return manager
}
```

`GetStoreCache` returns a cache from the CommitStoreCacheManager for a given store key. If no cache exists for the store key, then one is created and set.

| Name  | Type | Description |
| ------------- | ---------|------- |
| manager  | `CommitKVStoreCacheManager` | The cache manager |
| storeKey  | string | The store key of the store being retrieved |
| store  | `CommitKVStore` | The store that it is cached in case the manager does not have any in its map of caches |

```go
func GetStoreCache(
    manager CommitKVStoreCacheManager,
    storeKey string,
    store CommitKVStore) CommitKVStore {

    if manager.caches.has(storeKey) {
        return manager.caches.get(storeKey)
    } else {
        cache = CommitKVStoreCacheManager{store, manager.cacheSize}
        manager.set(storeKey, cache)
        return cache
    }
}
```

`Unwrap` returns the underlying CommitKVStore for a given store key.

| Name  | Type | Description |
| ------------- | ---------|------- |
| manager  | `CommitKVStoreCacheManager` | The cache manager |
| storeKey  | string | The store key of the store being unwrapped |

```go
func Unwrap(
    manager CommitKVStoreCacheManager,
    storeKey string) CommitKVStore {

    if manager.caches.has(storeKey) {
        cache = manager.caches.get(storeKey)
        return cache.store
    } else {
        return nil
    }
}
```

`Reset` resets the manager's map of caches.

| Name  | Type | Description |
| ------------- | ---------|------- |
| manager  | `CommitKVStoreCacheManager` | The cache manager |

```go
function Reset(manager CommitKVStoreCacheManager) {

    for (let storeKey of manager.caches.keys()) {
        manager.caches.delete(storeKey)
    }
}
```

#### CommitKVStoreCache

`NewCommitKVStoreCache` creates a new `CommitKVStoreCache` and returns it.

| Name  | Type | Description |
| ------------- | ---------|------- |
| store  | CommitKVStore | The store to be cached |
| size  | string | Determines the capacity of the cache being created |

```go
func NewCommitKVStoreCache(
    store CommitKVStore,
    size uint) CommitKVStoreCache {
    KVCache = CommitKVStoreCache{store, NewCache(size)}
    return KVCache
}
```

`Get` retrieves a value by key. It first looks in the cache. If the key is not in the cache, the query is delegated to the underlying `CommitKVStore`. In the latter case, the key/value pair is cached. The method returns the value.

| Name  | Type | Description |
| ------------- | ---------|------- |
| KVCache  | `CommitKVStoreCache` | The `CommitKVStoreCache` from which the key/value pair is retrieved  |
| key  | string | Key of the key/value pair being retrieved |

```go
func Get(
    KVCache CommitKVStoreCache,
    key string) []byte {
    valueCache, success := KVCache.cache.Get(key)
    if success {
        // cache hit
        return valueCache
    } else {
        // cache miss
        valueStore = KVCache.store.Get(key)
        KVCache.cache.Add(key, valueStore)
        return valueStore
    }
}
```

`Set` inserts a key/value pair into both the write-through cache and the underlying `CommitKVStore`.

| Name  | Type | Description |
| ------------- | ---------|------- |
| KVCache  | `CommitKVStoreCache` | The `CommitKVStoreCache` to which the key/value pair is inserted |
| key  | string | Key of the key/value pair being inserted |
| value  | []byte | Value of the key/value pair being inserted |

```go
func Set(
    KVCache CommitKVStoreCache,
    key string,
    value []byte) {

    KVCache.cache.Add(key, value)
    KVCache.store.Set(key, value)
}
```

`Delete` removes a key/value pair from both the write-through cache and the underlying `CommitKVStore`.

| Name  | Type | Description |
| ------------- | ---------|------- |
| KVCache  | `CommitKVStoreCache` | The `CommitKVStoreCache` from which the key/value pair is deleted |
| key  | string | Key of the key/value pair being deleted |

```go
func Delete(
    KVCache CommitKVStoreCache,
    key string) {

    KVCache.cache.Remove(key)
    KVCache.store.Delete(key)
}
```

`CacheWrap` wraps a `CommitKVStoreCache` with another caching layer (`CacheKV`). 

> It is unclear whether there is a use case for `CacheWrap`. 

| Name  | Type | Description |
| ------------- | ---------|------- |
| KVCache  | `CommitKVStoreCache` | The `CommitKVStoreCache` being wrapped |

```go
func CacheWrap(
    KVCache CommitKVStoreCache) {
     
    return CacheKV.NewStore(KVCache)
}
```

### Implementation details
<!-- markdown-link-check-disable -->
The inter-block cache implementation uses a fixed-sized adaptive replacement cache (ARC) as cache. [The ARC implementation](https://github.com/hashicorp/golang-lru/blob/main/arc/arc.go) is thread-safe. ARC is an enhancement over the standard LRU cache in that tracks both frequency and recency of use. This avoids a burst in access to new entries from evicting the frequently used older entries. It adds some additional tracking overhead to a standard LRU cache, computationally it is roughly `2x` the cost, and the extra memory overhead is linear with the size of the cache. The default cache size is `1000`.
<!-- markdown-link-check-enable -->
## History

Dec 20, 2022 - Initial draft finished and submitted as a PR

## Copyright

All content herein is licensed under [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0).
