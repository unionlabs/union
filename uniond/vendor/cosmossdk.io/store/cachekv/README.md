# CacheKVStore specification

A `CacheKVStore` is cache wrapper for a `KVStore`. It extends the operations of the `KVStore` to work with a write-back cache, allowing for reduced I/O operations and more efficient disposing of changes (e.g. after processing a failed transaction).

The core goals the CacheKVStore seeks to solve are:

* Buffer all writes to the parent store, so they can be dropped if they need to be reverted
* Allow iteration over contiguous spans of keys
* Act as a cache, improving access time for reads that have already been done (by replacing tree access with hashtable access, avoiding disk I/O)
  * Note: We actually fail to achieve this for iteration right now
  * Note: Need to consider this getting too large and dropping some cached reads
* Make subsequent reads account for prior buffered writes
* Write all buffered changes to the parent store

We should revisit these goals with time (for instance it's unclear that all disk writes need to be buffered to the end of the block), but this is the current status.

## Types and Structs

```go
type Store struct {
	mtx           sync.Mutex
	cache         map[string]*cValue
	deleted       map[string]struct{}
	unsortedCache map[string]struct{}
	sortedCache   *dbm.MemDB // always ascending sorted
	parent        types.KVStore
}
```

The Store struct wraps the underlying `KVStore` (`parent`) with additional data structures for implementing the cache. Mutex is used as IAVL trees (the `KVStore` in application) are not safe for concurrent use.

### `cache`

The main mapping of key-value pairs stored in cache. This map contains both keys that are cached from read operations as well as ‘dirty’ keys which map to a value that is potentially different than what is in the underlying `KVStore`.

Values that are mapped to in `cache` are wrapped in a `cValue` struct, which contains the value and a boolean flag (`dirty`) representing whether the value has been written since the last write-back to `parent`.

```go
type cValue struct {
	value []byte
	dirty bool
}
```

### `deleted`

Key-value pairs that are to be deleted from `parent` are stored in the `deleted` map. Keys are mapped to an empty struct to implement a set.

### `unsortedCache`

Similar to `deleted`, this is a set of keys that are dirty and will need to be updated in the parent `KVStore` upon a write. Keys are mapped to an empty struct to implement a set.

### `sortedCache`

A database that will be populated by the keys in `unsortedCache` during iteration over the cache. The keys are always held in sorted order.

## CRUD Operations and Writing

The `Set`, `Get`, and `Delete` functions all call `setCacheValue()`, which is the only entry point to mutating `cache` (besides `Write()`, which clears it).

`setCacheValue()` inserts a key-value pair into `cache`. Two boolean parameters, `deleted` and `dirty`, are passed in to flag whether the inserted key should also be inserted into the `deleted` and `dirty` sets. Keys will be removed from the `deleted` set if they are written to after being deleted.

### `Get`

`Get` first attempts to return the value from `cache`. If the key does not exist in `cache`, `parent.Get()` is called instead. This value from the parent is passed into `setCacheValue()` with `deleted=false` and `dirty=false`.

### `Has`

`Has` returns true if `Get` returns a non-nil value. As a result of calling `Get`, it may mutate the cache by caching the read.

### `Set`

New values are written by setting or updating the value of a key in `cache`. `Set` does not write to `parent`. 

Calls `setCacheValue()` with `deleted=false` and `dirty=true`.

### `Delete`

A value being deleted from the `KVStore` is represented with a `nil` value in `cache`, and an insertion of the key into the `deleted` set. `Delete` does not write to `parent`. 

Calls `setCacheValue()` with `deleted=true` and `dirty=true`.

### `Write`

Key-value pairs in the cache are written to `parent` in ascending order of their keys. 

A slice of all dirty keys in `cache` is made, then sorted in increasing order. These keys are iterated over to update `parent`.

If a key is marked for deletion (checked with `isDeleted()`), then `parent.Delete()` is called. Otherwise, `parent.Set()` is called to update the underlying `KVStore` with the value in cache.

## Iteration

Efficient iteration over keys in `KVStore` is important for generating Merkle range proofs. Iteration over `CacheKVStore` requires producing all key-value pairs from the underlying `KVStore` while taking into account updated values from the cache. 

In the current implementation, there is no guarantee that all values in `parent` have been cached. As a result, iteration is achieved by interleaved iteration through both `parent` and the cache (failing to actually benefit from caching).

[cacheMergeIterator](https://github.com/cosmos/cosmos-sdk/blob/d8391cb6796d770b02448bee70b865d824e43449/store/cachekv/mergeiterator.go) implements functions to provide a single iterator with an input of iterators over `parent` and the cache. This iterator iterates over keys from both iterators in a shared lexicographic order, and overrides the value provided by the parent iterator if the same key is dirty or deleted in the cache.

### Implementation Overview

Iterators over `parent` and the cache are generated and passed into `cacheMergeIterator`, which returns a single, interleaved iterator. Implementation of the `parent` iterator is up to the underlying `KVStore`. The remainder of this section covers the generation of the cache iterator.

Recall that `unsortedCache` is an unordered set of dirty cache keys. Our goal is to construct an ordered iterator over cache keys that fall within the `start` and `end` bounds requested.

Generating the cache iterator can be decomposed into four parts:

1. Finding all keys that exist in the range we are iterating over
2. Sorting this list of keys
3. Inserting these keys into `sortedCache` and removing them from `unsortedCache`
4. Returning an iterator over `sortedCache` with the desired range

Currently, the implementation for the first two parts is split into two cases, depending on the size of the unsorted cache. The two cases are as follows.

If the size of `unsortedCache` is less than `minSortSize` (currently 1024), a linear time approach is taken to search over keys.

```go
n := len(store.unsortedCache)
unsorted := make([]*kv.Pair, 0)

if n < minSortSize {
	for key := range store.unsortedCache {
		if dbm.IsKeyInDomain(conv.UnsafeStrToBytes(key), start, end) {
			cacheValue := store.cache[key]
			unsorted = append(unsorted, &kv.Pair{Key: []byte(key), Value: cacheValue.value})
		}
	}
	store.clearUnsortedCacheSubset(unsorted, stateUnsorted)
	return
}
```

Here, we iterate through all the keys in `unsortedCache` (i.e., the dirty cache keys), collecting those within the requested range in an unsorted slice called `unsorted`. 

At this point, part 3. is achieved in `clearUnsortedCacheSubset()`. This function iterates through `unsorted`, removing each key from `unsortedCache`. Afterwards, `unsorted` is sorted. Lastly, it iterates through the now sorted slice, inserting key-value pairs into `sortedCache`. Any key marked for deletion is mapped to an arbitrary value (`[]byte{}`).

In the case that the size of `unsortedCache` is larger than `minSortSize`, a linear time approach to finding keys within the desired range is too slow to use. Instead, a slice of all keys in `unsortedCache` is sorted, and binary search is used to find the beginning and ending indices of the desired range. This produces an already-sorted slice that is passed into the same `clearUnsortedCacheSubset()` function. An iota identifier (`sortedState`) is used to skip the sorting step in the function. 

Finally, part 4. is achieved with `memIterator`, which implements an iterator over the items in `sortedCache`. 

As of [PR #12885](https://github.com/cosmos/cosmos-sdk/pull/12885), an optimization to the binary search case mitigates the overhead of sorting the entirety of the key set in `unsortedCache`. To avoid wasting the compute spent sorting, we should ensure that a reasonable amount of values are removed from `unsortedCache`. If the length of the range for iteration is less than `minSortedCache`, we widen the range of values for removal from `unsortedCache` to be up to `minSortedCache` in length. This amortizes the cost of processing elements across multiple calls.