package listenkv_test

import (
	"bytes"
	"fmt"
	"io"
	"testing"

	"github.com/cosmos/cosmos-sdk/codec"
	codecTypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/store/dbadapter"
	"github.com/cosmos/cosmos-sdk/store/listenkv"
	"github.com/cosmos/cosmos-sdk/store/prefix"
	"github.com/cosmos/cosmos-sdk/store/types"

	"github.com/stretchr/testify/require"

	dbm "github.com/cometbft/cometbft-db"
)

func bz(s string) []byte { return []byte(s) }

func keyFmt(i int) []byte { return bz(fmt.Sprintf("key%0.8d", i)) }
func valFmt(i int) []byte { return bz(fmt.Sprintf("value%0.8d", i)) }

var kvPairs = []types.KVPair{
	{Key: keyFmt(1), Value: valFmt(1)},
	{Key: keyFmt(2), Value: valFmt(2)},
	{Key: keyFmt(3), Value: valFmt(3)},
}

var (
	testStoreKey      = types.NewKVStoreKey("listen_test")
	interfaceRegistry = codecTypes.NewInterfaceRegistry()
	testMarshaller    = codec.NewProtoCodec(interfaceRegistry)
)

func newListenKVStore(w io.Writer) *listenkv.Store {
	store := newEmptyListenKVStore(w)

	for _, kvPair := range kvPairs {
		store.Set(kvPair.Key, kvPair.Value)
	}

	return store
}

func newEmptyListenKVStore(w io.Writer) *listenkv.Store {
	listener := types.NewStoreKVPairWriteListener(w, testMarshaller)
	memDB := dbadapter.Store{DB: dbm.NewMemDB()}

	return listenkv.NewStore(memDB, testStoreKey, []types.WriteListener{listener})
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
		var buf bytes.Buffer

		store := newListenKVStore(&buf)
		buf.Reset()
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
		var buf bytes.Buffer

		store := newEmptyListenKVStore(&buf)
		buf.Reset()
		store.Set(tc.key, tc.value)
		storeKVPair := new(types.StoreKVPair)
		testMarshaller.UnmarshalLengthPrefixed(buf.Bytes(), storeKVPair)

		require.Equal(t, tc.expectedOut, storeKVPair)
	}

	var buf bytes.Buffer
	store := newEmptyListenKVStore(&buf)
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
		var buf bytes.Buffer

		store := newListenKVStore(&buf)
		buf.Reset()
		store.Delete(tc.key)
		storeKVPair := new(types.StoreKVPair)
		testMarshaller.UnmarshalLengthPrefixed(buf.Bytes(), storeKVPair)

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
		var buf bytes.Buffer

		store := newListenKVStore(&buf)
		buf.Reset()
		ok := store.Has(tc.key)

		require.Equal(t, tc.expected, ok)
	}
}

func TestTestListenKVStoreIterator(t *testing.T) {
	var buf bytes.Buffer

	store := newListenKVStore(&buf)
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
	var buf bytes.Buffer

	store := newListenKVStore(&buf)
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
	memDB := dbadapter.Store{DB: dbm.NewMemDB()}
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
