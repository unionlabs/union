package lru

import (
	"fmt"

	lru "github.com/hashicorp/golang-lru"
)

// New creates an LRU of the given size.
func New(size int) *lru.Cache {
	cache, err := lru.New(size)
	if err != nil {
		panic(fmt.Errorf("lru new failed: %w", err))
	}
	return cache
}

// NewWithEvict constructs a fixed size cache with the given eviction
// callback.
func NewWithEvict(size int, onEvicted func(key interface{}, value interface{})) *lru.Cache {
	cache, err := lru.NewWithEvict(size, onEvicted)
	if err != nil {
		panic(fmt.Errorf("lru new with evict failed: %w", err))
	}
	return cache
}
