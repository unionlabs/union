package indexes

import (
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/collections"
)

func TestMultiIndex(t *testing.T) {
	sk, ctx := deps()
	schema := collections.NewSchemaBuilder(sk)

	mi := NewMulti(schema, collections.NewPrefix(1), "multi_index", collections.StringKey, collections.Uint64Key, func(_ uint64, value company) (string, error) {
		return value.City, nil
	})

	// we create two reference keys for primary key 1 and 2 associated with "milan"
	require.NoError(t, mi.Reference(ctx, 1, company{City: "milan"}, func() (company, error) { return company{}, collections.ErrNotFound }))
	require.NoError(t, mi.Reference(ctx, 2, company{City: "milan"}, func() (company, error) { return company{}, collections.ErrNotFound }))

	iter, err := mi.MatchExact(ctx, "milan")
	require.NoError(t, err)
	pks, err := iter.PrimaryKeys()
	require.NoError(t, err)
	require.Equal(t, []uint64{1, 2}, pks)

	// replace
	require.NoError(t, mi.Reference(ctx, 1, company{City: "new york"}, func() (company, error) { return company{City: "milan"}, nil }))

	// assert after replace only company with id 2 is referenced by milan
	iter, err = mi.MatchExact(ctx, "milan")
	require.NoError(t, err)
	pks, err = iter.PrimaryKeys()
	require.NoError(t, err)
	require.Equal(t, []uint64{2}, pks)

	// assert after replace company with id 1 is referenced by new york
	iter, err = mi.MatchExact(ctx, "new york")
	require.NoError(t, err)
	pks, err = iter.PrimaryKeys()
	require.NoError(t, err)
	require.Equal(t, []uint64{1}, pks)

	// test iter methods
	iter, err = mi.Iterate(ctx, nil)
	require.NoError(t, err)

	fullKey, err := iter.FullKey()
	require.NoError(t, err)
	require.Equal(t, collections.Join("milan", uint64(2)), fullKey)

	pk, err := iter.PrimaryKey()
	require.NoError(t, err)
	require.Equal(t, uint64(2), pk)

	iter.Next()
	require.True(t, iter.Valid())
	iter.Next()
	require.False(t, iter.Valid())
	require.NoError(t, iter.Close())
}

func TestMultiUnchecked(t *testing.T) {
	sk, ctx := deps()
	schema := collections.NewSchemaBuilder(sk)

	uncheckedMi := NewMulti(schema, collections.NewPrefix("prefix"), "multi_index", collections.StringKey, collections.Uint64Key, func(_ uint64, value company) (string, error) {
		return value.City, nil
	}, WithMultiUncheckedValue())

	mi := NewMulti(schema, collections.NewPrefix("prefix"), "multi_index", collections.StringKey, collections.Uint64Key, func(_ uint64, value company) (string, error) {
		return value.City, nil
	})

	rawKey, err := collections.EncodeKeyWithPrefix(
		collections.NewPrefix("prefix"),
		uncheckedMi.KeyCodec(),
		collections.Join("milan", uint64(2)))
	require.NoError(t, err)

	// set value to be something different from []byte{}
	require.NoError(t, sk.OpenKVStore(ctx).Set(rawKey, []byte("something")))

	// normal multi index will fail.
	err = mi.Walk(ctx, nil, func(indexingKey string, indexedKey uint64) (stop bool, err error) {
		return true, err
	})
	require.ErrorIs(t, err, collections.ErrEncoding)

	// unchecked multi index will not fail.
	err = uncheckedMi.Walk(ctx, nil, func(indexingKey string, indexedKey uint64) (stop bool, err error) {
		require.Equal(t, "milan", indexingKey)
		require.Equal(t, uint64(2), indexedKey)
		return true, err
	})
	require.NoError(t, err)

	// unchecked multi will also reset the value
	err = mi.Reference(ctx, 2, company{City: "milan"}, func() (company, error) {
		return company{
			City: "milan",
		}, nil
	})
	require.NoError(t, err)

	// value reset to []byte{}
	rawValue, err := sk.OpenKVStore(ctx).Get(rawKey)
	require.NoError(t, err)
	require.Equal(t, []byte{}, rawValue)
}
