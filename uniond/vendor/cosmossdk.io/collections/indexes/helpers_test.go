package indexes

import (
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/collections"
)

func TestHelpers(t *testing.T) {
	// uses ReversePair scenario.
	// We store balances as:
	// Key: Pair[Address=string, Denom=string] => Value: Amount=uint64

	sk, ctx := deps()
	sb := collections.NewSchemaBuilder(sk)

	keyCodec := collections.PairKeyCodec(collections.StringKey, collections.StringKey)
	indexedMap := collections.NewIndexedMap(
		sb,
		collections.NewPrefix("balances"), "balances",
		keyCodec,
		collections.Uint64Value,
		balanceIndex{
			Denom: NewReversePair[Amount](sb, collections.NewPrefix("denom_index"), "denom_index", keyCodec),
		},
	)

	err := indexedMap.Set(ctx, collections.Join("address1", "atom"), 100)
	require.NoError(t, err)

	err = indexedMap.Set(ctx, collections.Join("address1", "osmo"), 200)
	require.NoError(t, err)

	err = indexedMap.Set(ctx, collections.Join("address2", "osmo"), 300)
	require.NoError(t, err)

	// test collect values
	iter, err := indexedMap.Indexes.Denom.MatchExact(ctx, "osmo")
	require.NoError(t, err)

	values, err := CollectValues(ctx, indexedMap, iter)
	require.NoError(t, err)
	require.Equal(t, []Amount{200, 300}, values)

	// test collect key values

	iter, err = indexedMap.Indexes.Denom.MatchExact(ctx, "osmo")
	require.NoError(t, err)
	kvs, err := CollectKeyValues(ctx, indexedMap, iter)
	require.NoError(t, err)

	require.Equal(t, []collections.KeyValue[collections.Pair[Address, Denom], Amount]{
		{
			Key:   collections.Join("address1", "osmo"),
			Value: 200,
		},
		{
			Key:   collections.Join("address2", "osmo"),
			Value: 300,
		},
	}, kvs)

	// test scan values with early termination
	iter, err = indexedMap.Indexes.Denom.MatchExact(ctx, "osmo")
	require.NoError(t, err)
	numCalled := 0
	err = ScanValues(ctx, indexedMap, iter, func(v Amount) bool {
		require.Equal(t, Amount(200), v)
		numCalled++
		require.Equal(t, numCalled, 1)
		return true // says to stop
	})
	require.NoError(t, err)

	// test scan kv with early termination
	iter, err = indexedMap.Indexes.Denom.MatchExact(ctx, "osmo")
	require.NoError(t, err)
	numCalled = 0
	err = ScanKeyValues(ctx, indexedMap, iter, func(kv collections.KeyValue[collections.Pair[Address, Denom], Amount]) bool {
		require.Equal(t, Amount(200), kv.Value)
		require.Equal(t, collections.Join("address1", "osmo"), kv.Key)
		numCalled++
		require.Equal(t, numCalled, 1)
		return true // says to stop
	})
	require.NoError(t, err)
}
