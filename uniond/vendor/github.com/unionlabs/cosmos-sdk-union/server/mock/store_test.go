package mock

import (
	"testing"

	"github.com/stretchr/testify/require"

	coretesting "cosmossdk.io/core/testing"
	storetypes "cosmossdk.io/store/types"
)

func TestStore(t *testing.T) {
	db := coretesting.NewMemDB()

	cms := NewCommitMultiStore()

	key := storetypes.NewKVStoreKey("test")
	cms.MountStoreWithDB(key, storetypes.StoreTypeIAVL, db)
	err := cms.LoadLatestVersion()
	require.Nil(t, err)

	store := cms.GetKVStore(key)
	require.NotNil(t, store)

	k := []byte("hello")
	v := []byte("world")
	require.False(t, store.Has(k))
	store.Set(k, v)
	require.True(t, store.Has(k))
	require.Equal(t, v, store.Get(k))
	store.Delete(k)
	require.False(t, store.Has(k))
	require.Panics(t, func() { store.Set([]byte(""), v) }, "setting an empty key should panic")
	require.Panics(t, func() { store.Set(nil, v) }, "setting a nil key should panic")
}
