package sims

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"
	"gotest.tools/v3/assert"

	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/log"
	"cosmossdk.io/store/metrics"
	"cosmossdk.io/store/rootmulti"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/types/kv"
	"github.com/cosmos/cosmos-sdk/types/simulation"
)

const (
	authStoreKey           = "acc"
	GlobalAccountNumberKey = 0x1
)

func TestGetSimulationLog(t *testing.T) {
	legacyAmino := codec.NewLegacyAmino()
	decoders := make(simulation.StoreDecoderRegistry)
	decoders[authStoreKey] = func(kvAs, kvBs kv.Pair) string { return "10" }

	tests := []struct {
		store       string
		kvPairs     []kv.Pair
		expectedLog string
	}{
		{
			"Empty",
			[]kv.Pair{{}},
			"",
		},
		{
			authStoreKey,
			[]kv.Pair{{Key: []byte{GlobalAccountNumberKey}, Value: legacyAmino.MustMarshal(uint64(10))}},
			"10",
		},
		{
			"OtherStore",
			[]kv.Pair{{Key: []byte("key"), Value: []byte("value")}},
			fmt.Sprintf("store A %X => %X\nstore B %X => %X\n", []byte("key"), []byte("value"), []byte("key"), []byte("value")),
		},
	}

	for _, tt := range tests {
		t.Run(tt.store, func(t *testing.T) {
			require.Equal(t, tt.expectedLog, GetSimulationLog(tt.store, decoders, tt.kvPairs, tt.kvPairs), tt.store)
		})
	}
}

func TestDiffKVStores(t *testing.T) {
	store1, store2 := initTestStores(t)
	// Two equal stores
	k1, v1 := []byte("k1"), []byte("v1")
	store1.Set(k1, v1)
	store2.Set(k1, v1)

	checkDiffResults(t, store1, store2, true, nil)

	// delete k1 from store2, which is now empty
	store2.Delete(k1)
	checkDiffResults(t, store1, store2, false, nil)

	// set k1 in store2, different value than what store1 holds for k1
	v2 := []byte("v2")
	store2.Set(k1, v2)
	checkDiffResults(t, store1, store2, false, nil)

	// add k2 to store2
	k2 := []byte("k2")
	store2.Set(k2, v2)
	checkDiffResults(t, store1, store2, false, nil)

	// add k3 to store1
	k3 := []byte("k3")
	store1.Set(k3, v2)
	checkDiffResults(t, store1, store2, false, nil)

	// Reset stores
	store1.Delete(k1)
	store1.Delete(k3)
	store2.Delete(k1)
	store2.Delete(k2)

	// Same keys, different value. Comparisons will be nil as prefixes are skipped.
	prefix := []byte("prefix:")
	k1Prefixed := append(prefix, k1...)
	store1.Set(k1Prefixed, v1)
	store2.Set(k1Prefixed, v2)
	checkDiffResults(t, store1, store2, true, [][]byte{prefix})
}

func checkDiffResults(t *testing.T, store1, store2 storetypes.KVStore, noDiff bool, skipPrefixes [][]byte) {
	t.Helper()

	kvAs1, kvBs1 := DiffKVStores(store1, store2, skipPrefixes)

	if noDiff {
		assert.Assert(t, len(kvAs1) == 0)
		assert.Assert(t, len(kvBs1) == 0)
	} else {
		assert.Assert(t, len(kvAs1) > 0 || len(kvBs1) > 0)
	}
}

func initTestStores(t *testing.T) (storetypes.KVStore, storetypes.KVStore) {
	t.Helper()
	db := coretesting.NewMemDB()
	ms := rootmulti.NewStore(db, log.NewNopLogger(), metrics.NewNoOpMetrics())

	key1 := storetypes.NewKVStoreKey("store1")
	key2 := storetypes.NewKVStoreKey("store2")
	require.NotPanics(t, func() { ms.MountStoreWithDB(key1, storetypes.StoreTypeIAVL, db) })
	require.NotPanics(t, func() { ms.MountStoreWithDB(key2, storetypes.StoreTypeIAVL, db) })
	require.NotPanics(t, func() {
		_ = ms.LoadLatestVersion()
	})
	return ms.GetKVStore(key1), ms.GetKVStore(key2)
}
