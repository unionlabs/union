package cachekv_test

import (
	fmt "fmt"
	"testing"

	"github.com/stretchr/testify/require"

	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/store/cachekv"
	"cosmossdk.io/store/dbadapter"
	"cosmossdk.io/store/types"
)

func DoBenchmarkDeepCacheStack(b *testing.B, depth int) {
	b.Helper()
	db := coretesting.NewMemDB()
	initialStore := cachekv.NewStore(dbadapter.Store{DB: db})

	nItems := 20
	for i := 0; i < nItems; i++ {
		initialStore.Set([]byte(fmt.Sprintf("hello%03d", i)), []byte{0})
	}

	var stack CacheStack
	stack.Reset(initialStore)

	for i := 0; i < depth; i++ {
		stack.Snapshot()

		store := stack.CurrentStore()
		store.Set([]byte(fmt.Sprintf("hello%03d", i)), []byte{byte(i)})
	}

	store := stack.CurrentStore()

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		it := store.Iterator(nil, nil)
		items := make([][]byte, 0, nItems)
		for ; it.Valid(); it.Next() {
			items = append(items, it.Key())
			it.Value()
		}
		it.Close()
		require.Equal(b, nItems, len(items))
	}
}

func BenchmarkDeepCacheStack1(b *testing.B) {
	DoBenchmarkDeepCacheStack(b, 1)
}

func BenchmarkDeepCacheStack3(b *testing.B) {
	DoBenchmarkDeepCacheStack(b, 3)
}

func BenchmarkDeepCacheStack10(b *testing.B) {
	DoBenchmarkDeepCacheStack(b, 10)
}

func BenchmarkDeepCacheStack13(b *testing.B) {
	DoBenchmarkDeepCacheStack(b, 13)
}

// CacheStack manages a stack of nested cache store to
// support the evm `StateDB`'s `Snapshot` and `RevertToSnapshot` methods.
type CacheStack struct {
	initialStore types.CacheKVStore
	// Context of the initial state before transaction execution.
	// It's the context used by `StateDB.CommitedState`.
	cacheStores []types.CacheKVStore
}

// CurrentStore returns the top context of cached stack,
// if the stack is empty, returns the initial context.
func (cs *CacheStack) CurrentStore() types.CacheKVStore {
	l := len(cs.cacheStores)
	if l == 0 {
		return cs.initialStore
	}
	return cs.cacheStores[l-1]
}

// Reset sets the initial context and clear the cache context stack.
func (cs *CacheStack) Reset(initialStore types.CacheKVStore) {
	cs.initialStore = initialStore
	cs.cacheStores = nil
}

// IsEmpty returns true if the cache context stack is empty.
func (cs *CacheStack) IsEmpty() bool {
	return len(cs.cacheStores) == 0
}

// Commit commits all the cached contexts from top to bottom in order and clears the stack by setting an empty slice of cache contexts.
func (cs *CacheStack) Commit() {
	// commit in order from top to bottom
	for i := len(cs.cacheStores) - 1; i >= 0; i-- {
		cs.cacheStores[i].Write()
	}
	cs.cacheStores = nil
}

// CommitToRevision commit the cache after the target revision,
// to improve efficiency of db operations.
func (cs *CacheStack) CommitToRevision(target int) error {
	if target < 0 || target >= len(cs.cacheStores) {
		return fmt.Errorf("snapshot index %d out of bound [%d..%d)", target, 0, len(cs.cacheStores))
	}

	// commit in order from top to bottom
	for i := len(cs.cacheStores) - 1; i > target; i-- {
		cs.cacheStores[i].Write()
	}
	cs.cacheStores = cs.cacheStores[0 : target+1]

	return nil
}

// Snapshot pushes a new cached context to the stack,
// and returns the index of it.
func (cs *CacheStack) Snapshot() int {
	cs.cacheStores = append(cs.cacheStores, cachekv.NewStore(cs.CurrentStore()))
	return len(cs.cacheStores) - 1
}

// RevertToSnapshot pops all the cached contexts after the target index (inclusive).
// the target should be snapshot index returned by `Snapshot`.
// This function panics if the index is out of bounds.
func (cs *CacheStack) RevertToSnapshot(target int) {
	if target < 0 || target >= len(cs.cacheStores) {
		panic(fmt.Errorf("snapshot index %d out of bound [%d..%d)", target, 0, len(cs.cacheStores)))
	}
	cs.cacheStores = cs.cacheStores[:target]
}
