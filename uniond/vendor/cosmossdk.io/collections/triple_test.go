package collections_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/collections"
	"cosmossdk.io/collections/colltest"
	"cosmossdk.io/core/testing"
)

func TestTriple(t *testing.T) {
	kc := collections.TripleKeyCodec(collections.Uint64Key, collections.StringKey, collections.BytesKey)

	t.Run("conformance", func(t *testing.T) {
		colltest.TestKeyCodec(t, kc, collections.Join3(uint64(1), "2", []byte("3")))
	})
}

func TestTripleRange(t *testing.T) {
	ctx := coretesting.Context()
	sk := coretesting.KVStoreService(ctx, "test")
	schema := collections.NewSchemaBuilder(sk)
	// this is a key composed of 3 parts: uint64, string, []byte
	kc := collections.TripleKeyCodec(collections.Uint64Key, collections.StringKey, collections.BytesKey)

	keySet := collections.NewKeySet(schema, collections.NewPrefix(0), "triple", kc)

	keys := []collections.Triple[uint64, string, []byte]{
		collections.Join3(uint64(1), "A", []byte("1")),
		collections.Join3(uint64(1), "A", []byte("2")),
		collections.Join3(uint64(1), "B", []byte("3")),
		collections.Join3(uint64(2), "B", []byte("4")),
	}

	for _, k := range keys {
		require.NoError(t, keySet.Set(ctx, k))
	}

	// we prefix over (1) we expect 3 results
	iter, err := keySet.Iterate(ctx, collections.NewPrefixedTripleRange[uint64, string, []byte](uint64(1)))
	require.NoError(t, err)
	gotKeys, err := iter.Keys()
	require.NoError(t, err)
	require.Equal(t, keys[:3], gotKeys)

	// we super prefix over Join(1, "A") we expect 2 results
	iter, err = keySet.Iterate(ctx, collections.NewSuperPrefixedTripleRange[uint64, string, []byte](1, "A"))
	require.NoError(t, err)
	gotKeys, err = iter.Keys()
	require.NoError(t, err)
	require.Equal(t, keys[:2], gotKeys)
}
