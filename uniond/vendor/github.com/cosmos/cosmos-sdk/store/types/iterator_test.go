package types_test

import (
	"testing"

	dbm "github.com/cometbft/cometbft-db"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/store/iavl"
	"github.com/cosmos/cosmos-sdk/store/types"
)

func newMemTestKVStore(t *testing.T) types.KVStore {
	db := dbm.NewMemDB()
	store, err := iavl.LoadStore(db, log.NewNopLogger(), types.NewKVStoreKey("test"), types.CommitID{}, false, iavl.DefaultIAVLCacheSize, false)
	require.NoError(t, err)
	return store
}

func TestPaginatedIterator(t *testing.T) {
	kvs := newMemTestKVStore(t)
	total := 10
	lth := total - 1
	asc := make([][]byte, total)
	desc := make([][]byte, total)
	// store returns values in lexicographic order (or reverse lex order)
	for i := 0; i < total; i++ {
		key := []byte{byte(i)}
		kvs.Set(key, key)
		asc[i] = key
		desc[lth-i] = key
	}
	type testCase struct {
		desc        string
		page, limit uint
		result      [][]byte
		reverse     bool
	}
	for _, tc := range []testCase{
		{
			desc:   "FirstChunk",
			page:   1,
			limit:  4,
			result: asc[:4],
		},
		{
			desc:   "SecondChunk",
			page:   2,
			limit:  4,
			result: asc[4:8],
		},
		{
			desc:   "ThirdChunkHalf",
			page:   3,
			limit:  4,
			result: asc[8:],
		},
		{
			desc:   "OverLimit",
			page:   10,
			limit:  10,
			result: [][]byte{},
		},
		{
			desc:   "ZeroLimit",
			page:   1,
			result: [][]byte{},
		},
		{
			desc:    "ReverseFirstChunk",
			page:    1,
			limit:   6,
			result:  desc[:6],
			reverse: true,
		},
		{
			desc:    "ReverseSecondChunk",
			page:    2,
			limit:   6,
			result:  desc[6:],
			reverse: true,
		},
	} {
		tc := tc
		t.Run(tc.desc, func(t *testing.T) {
			var iter types.Iterator
			if tc.reverse {
				iter = types.KVStoreReversePrefixIteratorPaginated(kvs, nil, tc.page, tc.limit)
			} else {
				iter = types.KVStorePrefixIteratorPaginated(kvs, nil, tc.page, tc.limit)
			}
			defer iter.Close()

			result := [][]byte{}
			for ; iter.Valid(); iter.Next() {
				result = append(result, iter.Key())
			}

			require.Equal(t, tc.result, result)
			require.False(t, iter.Valid())
		})
	}
}

func TestPaginatedIteratorPanicIfInvalid(t *testing.T) {
	kvs := newMemTestKVStore(t)

	iter := types.KVStorePrefixIteratorPaginated(kvs, nil, 1, 1)
	defer iter.Close()
	require.False(t, iter.Valid())
	require.Panics(t, func() { iter.Next() }) // "iterator is empty"

	kvs.Set([]byte{1}, []byte{})

	iter = types.KVStorePrefixIteratorPaginated(kvs, nil, 1, 0)
	defer iter.Close()
	require.False(t, iter.Valid())
	require.Panics(t, func() { iter.Next() }) // "not empty but limit is zero"
}
