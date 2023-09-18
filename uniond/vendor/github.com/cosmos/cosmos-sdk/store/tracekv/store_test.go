package tracekv_test

import (
	"bytes"
	"fmt"
	"io"
	"testing"

	"github.com/stretchr/testify/require"

	dbm "github.com/cometbft/cometbft-db"

	"github.com/cosmos/cosmos-sdk/store/dbadapter"
	"github.com/cosmos/cosmos-sdk/store/prefix"
	"github.com/cosmos/cosmos-sdk/store/tracekv"
	"github.com/cosmos/cosmos-sdk/store/types"
)

func bz(s string) []byte { return []byte(s) }

func keyFmt(i int) []byte { return bz(fmt.Sprintf("key%0.8d", i)) }
func valFmt(i int) []byte { return bz(fmt.Sprintf("value%0.8d", i)) }

var kvPairs = []types.KVPair{
	{Key: keyFmt(1), Value: valFmt(1)},
	{Key: keyFmt(2), Value: valFmt(2)},
	{Key: keyFmt(3), Value: valFmt(3)},
}

func newTraceKVStore(w io.Writer) *tracekv.Store {
	store := newEmptyTraceKVStore(w)

	for _, kvPair := range kvPairs {
		store.Set(kvPair.Key, kvPair.Value)
	}

	return store
}

func newEmptyTraceKVStore(w io.Writer) *tracekv.Store {
	memDB := dbadapter.Store{DB: dbm.NewMemDB()}
	tc := types.TraceContext(map[string]interface{}{"blockHeight": 64})

	return tracekv.NewStore(memDB, w, tc)
}

func TestTraceKVStoreGet(t *testing.T) {
	testCases := []struct {
		key           []byte
		expectedValue []byte
		expectedOut   string
	}{
		{
			key:           kvPairs[0].Key,
			expectedValue: kvPairs[0].Value,
			expectedOut:   "{\"operation\":\"read\",\"key\":\"a2V5MDAwMDAwMDE=\",\"value\":\"dmFsdWUwMDAwMDAwMQ==\",\"metadata\":{\"blockHeight\":64}}\n",
		},
		{
			key:           []byte("does-not-exist"),
			expectedValue: nil,
			expectedOut:   "{\"operation\":\"read\",\"key\":\"ZG9lcy1ub3QtZXhpc3Q=\",\"value\":\"\",\"metadata\":{\"blockHeight\":64}}\n",
		},
	}

	for _, tc := range testCases {
		var buf bytes.Buffer

		store := newTraceKVStore(&buf)
		buf.Reset()
		value := store.Get(tc.key)

		require.Equal(t, tc.expectedValue, value)
		require.Equal(t, tc.expectedOut, buf.String())
	}
}

func TestTraceKVStoreSet(t *testing.T) {
	testCases := []struct {
		key         []byte
		value       []byte
		expectedOut string
	}{
		{
			key:         kvPairs[0].Key,
			value:       kvPairs[0].Value,
			expectedOut: "{\"operation\":\"write\",\"key\":\"a2V5MDAwMDAwMDE=\",\"value\":\"dmFsdWUwMDAwMDAwMQ==\",\"metadata\":{\"blockHeight\":64}}\n",
		},
		{
			key:         kvPairs[1].Key,
			value:       kvPairs[1].Value,
			expectedOut: "{\"operation\":\"write\",\"key\":\"a2V5MDAwMDAwMDI=\",\"value\":\"dmFsdWUwMDAwMDAwMg==\",\"metadata\":{\"blockHeight\":64}}\n",
		},
		{
			key:         kvPairs[2].Key,
			value:       kvPairs[2].Value,
			expectedOut: "{\"operation\":\"write\",\"key\":\"a2V5MDAwMDAwMDM=\",\"value\":\"dmFsdWUwMDAwMDAwMw==\",\"metadata\":{\"blockHeight\":64}}\n",
		},
	}

	for _, tc := range testCases {
		var buf bytes.Buffer

		store := newEmptyTraceKVStore(&buf)
		buf.Reset()
		store.Set(tc.key, tc.value)

		require.Equal(t, tc.expectedOut, buf.String())
	}

	var buf bytes.Buffer
	store := newEmptyTraceKVStore(&buf)
	require.Panics(t, func() { store.Set([]byte(""), []byte("value")) }, "setting an empty key should panic")
	require.Panics(t, func() { store.Set(nil, []byte("value")) }, "setting a nil key should panic")
}

func TestTraceKVStoreDelete(t *testing.T) {
	testCases := []struct {
		key         []byte
		expectedOut string
	}{
		{
			key:         kvPairs[0].Key,
			expectedOut: "{\"operation\":\"delete\",\"key\":\"a2V5MDAwMDAwMDE=\",\"value\":\"\",\"metadata\":{\"blockHeight\":64}}\n",
		},
	}

	for _, tc := range testCases {
		var buf bytes.Buffer

		store := newTraceKVStore(&buf)
		buf.Reset()
		store.Delete(tc.key)

		require.Equal(t, tc.expectedOut, buf.String())
	}
}

func TestTraceKVStoreHas(t *testing.T) {
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

		store := newTraceKVStore(&buf)
		buf.Reset()
		ok := store.Has(tc.key)

		require.Equal(t, tc.expected, ok)
	}
}

func TestTestTraceKVStoreIterator(t *testing.T) {
	var buf bytes.Buffer

	store := newTraceKVStore(&buf)
	iterator := store.Iterator(nil, nil)

	s, e := iterator.Domain()
	require.Equal(t, []byte(nil), s)
	require.Equal(t, []byte(nil), e)

	testCases := []struct {
		expectedKey      []byte
		expectedValue    []byte
		expectedKeyOut   string
		expectedvalueOut string
	}{
		{
			expectedKey:      kvPairs[0].Key,
			expectedValue:    kvPairs[0].Value,
			expectedKeyOut:   "{\"operation\":\"iterKey\",\"key\":\"a2V5MDAwMDAwMDE=\",\"value\":\"\",\"metadata\":{\"blockHeight\":64}}\n",
			expectedvalueOut: "{\"operation\":\"iterValue\",\"key\":\"\",\"value\":\"dmFsdWUwMDAwMDAwMQ==\",\"metadata\":{\"blockHeight\":64}}\n",
		},
		{
			expectedKey:      kvPairs[1].Key,
			expectedValue:    kvPairs[1].Value,
			expectedKeyOut:   "{\"operation\":\"iterKey\",\"key\":\"a2V5MDAwMDAwMDI=\",\"value\":\"\",\"metadata\":{\"blockHeight\":64}}\n",
			expectedvalueOut: "{\"operation\":\"iterValue\",\"key\":\"\",\"value\":\"dmFsdWUwMDAwMDAwMg==\",\"metadata\":{\"blockHeight\":64}}\n",
		},
		{
			expectedKey:      kvPairs[2].Key,
			expectedValue:    kvPairs[2].Value,
			expectedKeyOut:   "{\"operation\":\"iterKey\",\"key\":\"a2V5MDAwMDAwMDM=\",\"value\":\"\",\"metadata\":{\"blockHeight\":64}}\n",
			expectedvalueOut: "{\"operation\":\"iterValue\",\"key\":\"\",\"value\":\"dmFsdWUwMDAwMDAwMw==\",\"metadata\":{\"blockHeight\":64}}\n",
		},
	}

	for _, tc := range testCases {
		buf.Reset()
		ka := iterator.Key()
		require.Equal(t, tc.expectedKeyOut, buf.String())

		buf.Reset()
		va := iterator.Value()
		require.Equal(t, tc.expectedvalueOut, buf.String())

		require.Equal(t, tc.expectedKey, ka)
		require.Equal(t, tc.expectedValue, va)

		iterator.Next()
	}

	require.False(t, iterator.Valid())
	require.Panics(t, iterator.Next)
	require.NoError(t, iterator.Close())
}

func TestTestTraceKVStoreReverseIterator(t *testing.T) {
	var buf bytes.Buffer

	store := newTraceKVStore(&buf)
	iterator := store.ReverseIterator(nil, nil)

	s, e := iterator.Domain()
	require.Equal(t, []byte(nil), s)
	require.Equal(t, []byte(nil), e)

	testCases := []struct {
		expectedKey      []byte
		expectedValue    []byte
		expectedKeyOut   string
		expectedvalueOut string
	}{
		{
			expectedKey:      kvPairs[2].Key,
			expectedValue:    kvPairs[2].Value,
			expectedKeyOut:   "{\"operation\":\"iterKey\",\"key\":\"a2V5MDAwMDAwMDM=\",\"value\":\"\",\"metadata\":{\"blockHeight\":64}}\n",
			expectedvalueOut: "{\"operation\":\"iterValue\",\"key\":\"\",\"value\":\"dmFsdWUwMDAwMDAwMw==\",\"metadata\":{\"blockHeight\":64}}\n",
		},
		{
			expectedKey:      kvPairs[1].Key,
			expectedValue:    kvPairs[1].Value,
			expectedKeyOut:   "{\"operation\":\"iterKey\",\"key\":\"a2V5MDAwMDAwMDI=\",\"value\":\"\",\"metadata\":{\"blockHeight\":64}}\n",
			expectedvalueOut: "{\"operation\":\"iterValue\",\"key\":\"\",\"value\":\"dmFsdWUwMDAwMDAwMg==\",\"metadata\":{\"blockHeight\":64}}\n",
		},
		{
			expectedKey:      kvPairs[0].Key,
			expectedValue:    kvPairs[0].Value,
			expectedKeyOut:   "{\"operation\":\"iterKey\",\"key\":\"a2V5MDAwMDAwMDE=\",\"value\":\"\",\"metadata\":{\"blockHeight\":64}}\n",
			expectedvalueOut: "{\"operation\":\"iterValue\",\"key\":\"\",\"value\":\"dmFsdWUwMDAwMDAwMQ==\",\"metadata\":{\"blockHeight\":64}}\n",
		},
	}

	for _, tc := range testCases {
		buf.Reset()
		ka := iterator.Key()
		require.Equal(t, tc.expectedKeyOut, buf.String())

		buf.Reset()
		va := iterator.Value()
		require.Equal(t, tc.expectedvalueOut, buf.String())

		require.Equal(t, tc.expectedKey, ka)
		require.Equal(t, tc.expectedValue, va)

		iterator.Next()
	}

	require.False(t, iterator.Valid())
	require.Panics(t, iterator.Next)
	require.NoError(t, iterator.Close())
}

func TestTraceKVStorePrefix(t *testing.T) {
	store := newEmptyTraceKVStore(nil)
	pStore := prefix.NewStore(store, []byte("trace_prefix"))
	require.IsType(t, prefix.Store{}, pStore)
}

func TestTraceKVStoreGetStoreType(t *testing.T) {
	memDB := dbadapter.Store{DB: dbm.NewMemDB()}
	store := newEmptyTraceKVStore(nil)
	require.Equal(t, memDB.GetStoreType(), store.GetStoreType())
}

func TestTraceKVStoreCacheWrap(t *testing.T) {
	store := newEmptyTraceKVStore(nil)
	require.Panics(t, func() { store.CacheWrap() })
}

func TestTraceKVStoreCacheWrapWithTrace(t *testing.T) {
	store := newEmptyTraceKVStore(nil)
	require.Panics(t, func() { store.CacheWrapWithTrace(nil, nil) })
}
