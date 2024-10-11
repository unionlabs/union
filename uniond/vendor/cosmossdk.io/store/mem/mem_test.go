package mem_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/store/cachekv"
	"cosmossdk.io/store/mem"
	pruningtypes "cosmossdk.io/store/pruning/types"
	"cosmossdk.io/store/types"
)

func TestStore(t *testing.T) {
	db := mem.NewStore()
	require.Equal(t, types.StoreTypeMemory, db.GetStoreType())

	key, value := []byte("key"), []byte("value")

	require.Nil(t, db.Get(key))
	db.Set(key, value)
	require.Equal(t, value, db.Get(key))

	newValue := []byte("newValue")
	db.Set(key, newValue)
	require.Equal(t, newValue, db.Get(key))

	db.Delete(key)
	require.Nil(t, db.Get(key))

	cacheWrapper := db.CacheWrap()
	require.IsType(t, &cachekv.Store{}, cacheWrapper)

	cacheWrappedWithTrace := db.CacheWrapWithTrace(nil, nil)
	require.IsType(t, &cachekv.Store{}, cacheWrappedWithTrace)
}

func TestCommit(t *testing.T) {
	db := mem.NewStore()
	key, value := []byte("key"), []byte("value")

	db.Set(key, value)
	id := db.Commit()
	require.True(t, id.IsZero())
	require.True(t, db.LastCommitID().IsZero())
	require.Equal(t, value, db.Get(key))
}

func TestStorePrunningOptions(t *testing.T) {
	// this is a no-op
	db := mem.NewStore()
	require.Equal(t, pruningtypes.NewPruningOptions(pruningtypes.PruningUndefined), db.GetPruning())
}
