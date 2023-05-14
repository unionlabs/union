package levigo

// #cgo LDFLAGS: -lleveldb
// #include <stdint.h>
// #include "leveldb/c.h"
import "C"

// Cache is a cache used to store data read from data in memory.
//
// Typically, NewLRUCache is all you will need, but advanced users may
// implement their own *C.leveldb_cache_t and create a Cache.
//
// To prevent memory leaks, a Cache must have Close called on it when it is
// no longer needed by the program. Note: if the process is shutting down,
// this may not be necessary and could be avoided to shorten shutdown time.
type Cache struct {
	Cache *C.leveldb_cache_t
}

// NewLRUCache creates a new Cache object with the capacity given.
//
// To prevent memory leaks, Close should be called on the Cache when the
// program no longer needs it. Note: if the process is shutting down, this may
// not be necessary and could be avoided to shorten shutdown time.
func NewLRUCache(capacity int) *Cache {
	return &Cache{C.leveldb_cache_create_lru(C.size_t(capacity))}
}

// Close deallocates the underlying memory of the Cache object.
func (c *Cache) Close() {
	C.leveldb_cache_destroy(c.Cache)
}
