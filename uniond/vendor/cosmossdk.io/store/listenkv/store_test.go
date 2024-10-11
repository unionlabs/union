package listenkv_test

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/store/dbadapter"
	"cosmossdk.io/store/internal/kv"
	"cosmossdk.io/store/listenkv"
	"cosmossdk.io/store/prefix"
	"cosmossdk.io/store/types"
)

func bz(s string) []byte { return []byte(s) }

func keyFmt(i int) []byte { return bz(fmt.Sprintf("key%0.8d", i)) }
func valFmt(i int) []byte { return bz(fmt.Sprintf("value%0.8d", i)) }

var kvPairs = []kv.Pair{
	{Key: keyFmt(1), Value: valFmt(1)},
	{Key: keyFmt(2), Value: valFmt(2)},
	{Key: keyFmt(3), Value: valFmt(3)},
}

var testStoreKey = types.NewKVStoreKey("listen_test")

func newListenKVStore(listener *types.MemoryListener) *listenkv.Store {
	store := newEmptyListenKVStore(listener)

	for _, kvPair := range kvPairs {
		store.Set(kvPair.Key, kvPair.Value)
	}

	return store
}

func newEmptyListenKVStore(listener *types.MemoryListener) *listenkv.Store {
	memDB := dbadapter.Store{DB: coretesting.NewMemDB()}

	return listenkv.NewStore(memDB, testStoreKey, listener)
}

func TestListenKVStoreGet(t *testing.T) {
	testCases := []struct {
		key           []byte
		expectedValue []byte
	}{
		{
			key:           kvPairs[0].Key,
			expectedValue: kvPairs[0].Value,
		},
		{
			key:           []byte("does-not-exist"),
			expectedValue: nil,
		},
	}

	for _, tc := range testCases {
		listener := types.NewMemoryListener()

		store := newListenKVStore(listener)
		value := store.Get(tc.key)

		require.Equal(t, tc.expectedValue, value)
	}
}

func TestListenKVStoreSet(t *testing.T) {
	testCases := []struct {
		key         []byte
		value       []byte
		expectedOut *types.StoreKVPair
	}{
		{
			key:   kvPairs[0].Key,
			value: kvPairs[0].Value,
			expectedOut: &types.StoreKVPair{
				Key:      kvPairs[0].Key,
				Value:    kvPairs[0].Value,
				StoreKey: testStoreKey.Name(),
				Delete:   false,
			},
		},
		{
			key:   kvPairs[1].Key,
			value: kvPairs[1].Value,
			expectedOut: &types.StoreKVPair{
				Key:      kvPairs[1].Key,
				Value:    kvPairs[1].Value,
				StoreKey: testStoreKey.Name(),
				Delete:   false,
			},
		},
		{
			key:   kvPairs[2].Key,
			value: kvPairs[2].Value,
			expectedOut: &types.StoreKVPair{
				Key:      kvPairs[2].Key,
				Value:    kvPairs[2].Value,
				StoreKey: testStoreKey.Name(),
				Delete:   false,
			},
		},
	}

	for _, tc := range testCases {
		listener := types.NewMemoryListener()

		store := newEmptyListenKVStore(listener)
		store.Set(tc.key, tc.value)
		storeKVPair := listener.PopStateCache()[0]

		require.Equal(t, tc.expectedOut, storeKVPair)
	}

	listener := types.NewMemoryListener()
	store := newEmptyListenKVStore(listener)
	require.Panics(t, func() { store.Set([]byte(""), []byte("value")) }, "setting an empty key should panic")
	require.Panics(t, func() { store.Set(nil, []byte("value")) }, "setting a nil key should panic")
}

func TestListenKVStoreDelete(t *testing.T) {
	testCases := []struct {
		key         []byte
		expectedOut *types.StoreKVPair
	}{
		{
			key: kvPairs[0].Key,
			expectedOut: &types.StoreKVPair{
				Key:      kvPairs[0].Key,
				Value:    nil,
				StoreKey: testStoreKey.Name(),
				Delete:   true,
			},
		},
	}

	for _, tc := range testCases {
		listener := types.NewMemoryListener()

		store := newListenKVStore(listener)
		store.Delete(tc.key)
		cache := listener.PopStateCache()
		require.NotEmpty(t, cache)
		storeKVPair := cache[len(cache)-1]

		require.Equal(t, tc.expectedOut, storeKVPair)
	}
}

func TestListenKVStoreHas(t *testing.T) {
	testCases := []struct {
		key      []byte
		expected bool
	}{
		{
			key:      kvPairs[0].Key,
			expected: true,
		},
	}

	for _, tc := range testCases {
		listener := types.NewMemoryListener()

		store := newListenKVStore(listener)
		ok := store.Has(tc.key)

		require.Equal(t, tc.expected, ok)
	}
}

func TestTestListenKVStoreIterator(t *testing.T) {
	listener := types.NewMemoryListener()

	store := newListenKVStore(listener)
	iterator := store.Iterator(nil, nil)

	s, e := iterator.Domain()
	require.Equal(t, []byte(nil), s)
	require.Equal(t, []byte(nil), e)

	testCases := []struct {
		expectedKey   []byte
		expectedValue []byte
	}{
		{
			expectedKey:   kvPairs[0].Key,
			expectedValue: kvPairs[0].Value,
		},
		{
			expectedKey:   kvPairs[1].Key,
			expectedValue: kvPairs[1].Value,
		},
		{
			expectedKey:   kvPairs[2].Key,
			expectedValue: kvPairs[2].Value,
		},
	}

	for _, tc := range testCases {
		ka := iterator.Key()
		require.Equal(t, tc.expectedKey, ka)

		va := iterator.Value()
		require.Equal(t, tc.expectedValue, va)

		iterator.Next()
	}

	require.False(t, iterator.Valid())
	require.Panics(t, iterator.Next)
	require.NoError(t, iterator.Close())
}

func TestTestListenKVStoreReverseIterator(t *testing.T) {
	listener := types.NewMemoryListener()

	store := newListenKVStore(listener)
	iterator := store.ReverseIterator(nil, nil)

	s, e := iterator.Domain()
	require.Equal(t, []byte(nil), s)
	require.Equal(t, []byte(nil), e)

	testCases := []struct {
		expectedKey   []byte
		expectedValue []byte
	}{
		{
			expectedKey:   kvPairs[2].Key,
			expectedValue: kvPairs[2].Value,
		},
		{
			expectedKey:   kvPairs[1].Key,
			expectedValue: kvPairs[1].Value,
		},
		{
			expectedKey:   kvPairs[0].Key,
			expectedValue: kvPairs[0].Value,
		},
	}

	for _, tc := range testCases {
		ka := iterator.Key()
		require.Equal(t, tc.expectedKey, ka)

		va := iterator.Value()
		require.Equal(t, tc.expectedValue, va)

		iterator.Next()
	}

	require.False(t, iterator.Valid())
	require.Panics(t, iterator.Next)
	require.NoError(t, iterator.Close())
}

func TestListenKVStorePrefix(t *testing.T) {
	store := newEmptyListenKVStore(nil)
	pStore := prefix.NewStore(store, []byte("listen_prefix"))
	require.IsType(t, prefix.Store{}, pStore)
}

func TestListenKVStoreGetStoreType(t *testing.T) {
	memDB := dbadapter.Store{DB: coretesting.NewMemDB()}
	store := newEmptyListenKVStore(nil)
	require.Equal(t, memDB.GetStoreType(), store.GetStoreType())
}

func TestListenKVStoreCacheWrap(t *testing.T) {
	store := newEmptyListenKVStore(nil)
	require.Panics(t, func() { store.CacheWrap() })
}

func TestListenKVStoreCacheWrapWithTrace(t *testing.T) {
	store := newEmptyListenKVStore(nil)
	require.Panics(t, func() { store.CacheWrapWithTrace(nil, nil) })
}
