package types_test

import (
	"bytes"
	"testing"

	dbm "github.com/cometbft/cometbft-db"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/store/rootmulti"
	sdk "github.com/cosmos/cosmos-sdk/store/types"
)

func initTestStores(t *testing.T) (sdk.KVStore, sdk.KVStore) {
	db := dbm.NewMemDB()
	ms := rootmulti.NewStore(db, log.NewNopLogger())

	key1 := sdk.NewKVStoreKey("store1")
	key2 := sdk.NewKVStoreKey("store2")
	require.NotPanics(t, func() { ms.MountStoreWithDB(key1, sdk.StoreTypeIAVL, db) })
	require.NotPanics(t, func() { ms.MountStoreWithDB(key2, sdk.StoreTypeIAVL, db) })
	require.NoError(t, ms.LoadLatestVersion())
	return ms.GetKVStore(key1), ms.GetKVStore(key2)
}

func TestDiffKVStores(t *testing.T) {
	t.Parallel()
	store1, store2 := initTestStores(t)
	// Two equal stores
	k1, v1 := []byte("k1"), []byte("v1")
	store1.Set(k1, v1)
	store2.Set(k1, v1)

	kvAs, kvBs := sdk.DiffKVStores(store1, store2, nil)
	require.Equal(t, 0, len(kvAs))
	require.Equal(t, len(kvAs), len(kvBs))

	// delete k1 from store2, which is now empty
	store2.Delete(k1)
	kvAs, kvBs = sdk.DiffKVStores(store1, store2, nil)
	require.Equal(t, 1, len(kvAs))
	require.Equal(t, len(kvAs), len(kvBs))

	// set k1 in store2, different value than what store1 holds for k1
	v2 := []byte("v2")
	store2.Set(k1, v2)
	kvAs, kvBs = sdk.DiffKVStores(store1, store2, nil)
	require.Equal(t, 1, len(kvAs))
	require.Equal(t, len(kvAs), len(kvBs))

	// add k2 to store2
	k2 := []byte("k2")
	store2.Set(k2, v2)
	kvAs, kvBs = sdk.DiffKVStores(store1, store2, nil)
	require.Equal(t, 2, len(kvAs))
	require.Equal(t, len(kvAs), len(kvBs))

	// Reset stores
	store1.Delete(k1)
	store2.Delete(k1)
	store2.Delete(k2)

	// Same keys, different value. Comparisons will be nil as prefixes are skipped.
	prefix := []byte("prefix:")
	k1Prefixed := append(prefix, k1...)
	store1.Set(k1Prefixed, v1)
	store2.Set(k1Prefixed, v2)
	kvAs, kvBs = sdk.DiffKVStores(store1, store2, [][]byte{prefix})
	require.Equal(t, 0, len(kvAs))
	require.Equal(t, len(kvAs), len(kvBs))
}

func TestPrefixEndBytes(t *testing.T) {
	t.Parallel()
	bs1 := []byte{0x23, 0xA5, 0x06}
	require.True(t, bytes.Equal([]byte{0x23, 0xA5, 0x07}, sdk.PrefixEndBytes(bs1)))
	bs2 := []byte{0x23, 0xA5, 0xFF}
	require.True(t, bytes.Equal([]byte{0x23, 0xA6}, sdk.PrefixEndBytes(bs2)))
	require.Nil(t, sdk.PrefixEndBytes([]byte{0xFF}))
	require.Nil(t, sdk.PrefixEndBytes(nil))
}

func TestInclusiveEndBytes(t *testing.T) {
	t.Parallel()
	require.True(t, bytes.Equal([]byte{0x00}, sdk.InclusiveEndBytes(nil)))
	bs := []byte("test")
	require.True(t, bytes.Equal(append(bs, byte(0x00)), sdk.InclusiveEndBytes(bs)))
}
