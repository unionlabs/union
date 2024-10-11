package types

import (
	"testing"

	iavl2 "github.com/cosmos/iavl"
	dbm "github.com/cosmos/iavl/db"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/log"
	"cosmossdk.io/store"
	"cosmossdk.io/store/iavl"
)

// This is modeled close to
// https://github.com/CosmWasm/cosmwasm-plus/blob/f97a7de44b6a930fd1d5179ee6f95b786a532f32/packages/storage-plus/src/prefix.rs#L183
// and designed to ensure the IAVL store handles bounds the same way as the mock storage we use in Rust contract tests
func TestIavlRangeBounds(t *testing.T) {
	memdb := dbm.NewMemDB()
	tree := iavl2.NewMutableTree(memdb, 50, false, log.NewTestLogger(t))
	kvstore := iavl.UnsafeNewStore(tree)

	// values to compare with
	expected := []KV{
		{[]byte("bar"), []byte("1")},
		{[]byte("ra"), []byte("2")},
		{[]byte("zi"), []byte("3")},
	}
	reversed := []KV{
		{[]byte("zi"), []byte("3")},
		{[]byte("ra"), []byte("2")},
		{[]byte("bar"), []byte("1")},
	}

	// set up test cases, like `ensure_proper_range_bounds` in `cw-storage-plus`
	for _, kv := range expected {
		kvstore.Set(kv.Key, kv.Value)
	}

	cases := map[string]struct {
		start    []byte
		end      []byte
		reverse  bool
		expected []KV
	}{
		"all ascending":             {nil, nil, false, expected},
		"ascending start inclusive": {[]byte("ra"), nil, false, expected[1:]},
		"ascending end exclusive":   {nil, []byte("ra"), false, expected[:1]},
		"ascending both points":     {[]byte("bar"), []byte("zi"), false, expected[:2]},

		"all descending":             {nil, nil, true, reversed},
		"descending start inclusive": {[]byte("ra"), nil, true, reversed[:2]},           // "zi", "ra"
		"descending end inclusive":   {nil, []byte("ra"), true, reversed[2:]},           // "bar"
		"descending both points":     {[]byte("bar"), []byte("zi"), true, reversed[1:]}, // "ra", "bar"
	}

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			var iter store.Iterator
			if tc.reverse {
				iter = kvstore.ReverseIterator(tc.start, tc.end)
			} else {
				iter = kvstore.Iterator(tc.start, tc.end)
			}
			items := consume(iter)
			require.Equal(t, tc.expected, items)
			iter.Close()
		})
	}
}

type KV struct {
	Key   []byte
	Value []byte
}

func consume(itr store.Iterator) []KV {
	var res []KV
	for ; itr.Valid(); itr.Next() {
		k, v := itr.Key(), itr.Value()
		res = append(res, KV{k, v})
	}
	return res
}
