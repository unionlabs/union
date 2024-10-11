package collections_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/collections"
	"cosmossdk.io/core/testing"
)

func TestLookupMap(t *testing.T) {
	ctx := coretesting.Context()
	sk := coretesting.KVStoreService(ctx, "test")
	schema := collections.NewSchemaBuilder(sk)

	lm := collections.NewLookupMap(schema, collections.NewPrefix("hi"), "lm", collections.Uint64Key, collections.Uint64Value)
	_, err := schema.Build()
	require.NoError(t, err)

	// test not has
	has, err := lm.Has(ctx, 1)
	require.NoError(t, err)
	require.False(t, has)
	// test get error
	_, err = lm.Get(ctx, 1)
	require.ErrorIs(t, err, collections.ErrNotFound)

	// test set/get
	err = lm.Set(ctx, 1, 100)
	require.NoError(t, err)
	v, err := lm.Get(ctx, 1)
	require.NoError(t, err)
	require.Equal(t, uint64(100), v)

	// test remove
	err = lm.Remove(ctx, 1)
	require.NoError(t, err)
	has, err = lm.Has(ctx, 1)
	require.NoError(t, err)
	require.False(t, has)
}
