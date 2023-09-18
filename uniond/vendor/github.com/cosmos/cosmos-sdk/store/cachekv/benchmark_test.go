package cachekv_test

import (
	fmt "fmt"
	"testing"

	dbm "github.com/cometbft/cometbft-db"
	"github.com/cometbft/cometbft/libs/log"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/cosmos/cosmos-sdk/store"
	storetypes "github.com/cosmos/cosmos-sdk/store/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func DoBenchmarkDeepContextStack(b *testing.B, depth int) {
	begin := []byte{0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00}
	end := []byte{0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff}
	key := storetypes.NewKVStoreKey("test")

	db := dbm.NewMemDB()
	cms := store.NewCommitMultiStore(db)
	cms.MountStoreWithDB(key, storetypes.StoreTypeIAVL, db)
	cms.LoadLatestVersion()
	ctx := sdk.NewContext(cms, tmproto.Header{}, false, log.NewNopLogger())

	var stack ContextStack
	stack.Reset(ctx)

	for i := 0; i < depth; i++ {
		stack.Snapshot()

		store := stack.CurrentContext().KVStore(key)
		store.Set(begin, []byte("value"))
	}

	store := stack.CurrentContext().KVStore(key)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		it := store.Iterator(begin, end)
		it.Valid()
		it.Key()
		it.Value()
		it.Next()
		it.Close()
	}
}

func BenchmarkDeepContextStack1(b *testing.B) {
	DoBenchmarkDeepContextStack(b, 1)
}

func BenchmarkDeepContextStack3(b *testing.B) {
	DoBenchmarkDeepContextStack(b, 3)
}

func BenchmarkDeepContextStack10(b *testing.B) {
	DoBenchmarkDeepContextStack(b, 10)
}

func BenchmarkDeepContextStack13(b *testing.B) {
	DoBenchmarkDeepContextStack(b, 13)
}

// cachedContext is a pair of cache context and its corresponding commit method.
// They are obtained from the return value of `context.CacheContext()`.
type cachedContext struct {
	ctx    sdk.Context
	commit func()
}

// ContextStack manages the initial context and a stack of cached contexts,
// to support the `StateDB.Snapshot` and `StateDB.RevertToSnapshot` methods.
//
// Copied from an old version of ethermint
type ContextStack struct {
	// Context of the initial state before transaction execution.
	// It's the context used by `StateDB.CommitedState`.
	initialCtx     sdk.Context
	cachedContexts []cachedContext
}

// CurrentContext returns the top context of cached stack,
// if the stack is empty, returns the initial context.
func (cs *ContextStack) CurrentContext() sdk.Context {
	l := len(cs.cachedContexts)
	if l == 0 {
		return cs.initialCtx
	}
	return cs.cachedContexts[l-1].ctx
}

// Reset sets the initial context and clear the cache context stack.
func (cs *ContextStack) Reset(ctx sdk.Context) {
	cs.initialCtx = ctx
	if len(cs.cachedContexts) > 0 {
		cs.cachedContexts = []cachedContext{}
	}
}

// IsEmpty returns true if the cache context stack is empty.
func (cs *ContextStack) IsEmpty() bool {
	return len(cs.cachedContexts) == 0
}

// Commit commits all the cached contexts from top to bottom in order and clears the stack by setting an empty slice of cache contexts.
func (cs *ContextStack) Commit() {
	// commit in order from top to bottom
	for i := len(cs.cachedContexts) - 1; i >= 0; i-- {
		if cs.cachedContexts[i].commit == nil {
			panic(fmt.Sprintf("commit function at index %d should not be nil", i))
		} else {
			cs.cachedContexts[i].commit()
		}
	}
	cs.cachedContexts = []cachedContext{}
}

// CommitToRevision commit the cache after the target revision,
// to improve efficiency of db operations.
func (cs *ContextStack) CommitToRevision(target int) error {
	if target < 0 || target >= len(cs.cachedContexts) {
		return fmt.Errorf("snapshot index %d out of bound [%d..%d)", target, 0, len(cs.cachedContexts))
	}

	// commit in order from top to bottom
	for i := len(cs.cachedContexts) - 1; i > target; i-- {
		if cs.cachedContexts[i].commit == nil {
			return fmt.Errorf("commit function at index %d should not be nil", i)
		}
		cs.cachedContexts[i].commit()
	}
	cs.cachedContexts = cs.cachedContexts[0 : target+1]

	return nil
}

// Snapshot pushes a new cached context to the stack,
// and returns the index of it.
func (cs *ContextStack) Snapshot() int {
	i := len(cs.cachedContexts)
	ctx, commit := cs.CurrentContext().CacheContext()
	cs.cachedContexts = append(cs.cachedContexts, cachedContext{ctx: ctx, commit: commit})
	return i
}

// RevertToSnapshot pops all the cached contexts after the target index (inclusive).
// the target should be snapshot index returned by `Snapshot`.
// This function panics if the index is out of bounds.
func (cs *ContextStack) RevertToSnapshot(target int) {
	if target < 0 || target >= len(cs.cachedContexts) {
		panic(fmt.Errorf("snapshot index %d out of bound [%d..%d)", target, 0, len(cs.cachedContexts)))
	}
	cs.cachedContexts = cs.cachedContexts[:target]
}

// RevertAll discards all the cache contexts.
func (cs *ContextStack) RevertAll() {
	if len(cs.cachedContexts) > 0 {
		cs.RevertToSnapshot(0)
	}
}
