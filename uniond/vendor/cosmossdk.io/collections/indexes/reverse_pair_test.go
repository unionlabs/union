package indexes

import (
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/collections"
)

type (
	Address = string
	Denom   = string
	Amount  = uint64
)

// our balance index, allows us to efficiently create an index between the key that maps
// balances which is a collections.Pair[Address, Denom] and the Denom.
type balanceIndex struct {
	Denom *ReversePair[Address, Denom, Amount]
}

func (b balanceIndex) IndexesList() []collections.Index[collections.Pair[Address, Denom], Amount] {
	return []collections.Index[collections.Pair[Address, Denom], Amount]{b.Denom}
}

func TestReversePair(t *testing.T) {
	sk, ctx := deps()
	sb := collections.NewSchemaBuilder(sk)
	// we create an indexed map that maps balances, which are saved as
	// key: Pair[Address, Denom]
	// value: Amount
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

	// assert if we iterate over osmo we find address1 and address2
	iter, err := indexedMap.Indexes.Denom.MatchExact(ctx, "osmo")
	require.NoError(t, err)
	defer iter.Close()

	pks, err := iter.PrimaryKeys()
	require.NoError(t, err)
	require.Equal(t, "address1", pks[0].K1())
	require.Equal(t, "address2", pks[1].K1())

	// assert if we remove address1 atom balance, we can no longer find it in the index
	err = indexedMap.Remove(ctx, collections.Join("address1", "atom"))
	require.NoError(t, err)
	iter, err = indexedMap.Indexes.Denom.MatchExact(ctx, "atom")
	require.NoError(t, err)
	defer iter.Close()
	pks, err = iter.PrimaryKeys()
	require.NoError(t, err)
	require.Empty(t, pks)
}

func TestUncheckedReversePair(t *testing.T) {
	sk, ctx := deps()
	sb := collections.NewSchemaBuilder(sk)
	prefix := collections.NewPrefix("prefix")
	keyCodec := collections.PairKeyCodec(collections.StringKey, collections.StringKey)

	uncheckedRp := NewReversePair[Amount](sb, prefix, "denom_index", keyCodec, WithReversePairUncheckedValue())
	rp := NewReversePair[Amount](sb, prefix, "denom_index", keyCodec)

	rawKey, err := collections.EncodeKeyWithPrefix(prefix, uncheckedRp.KeyCodec(), collections.Join("atom", "address1"))
	require.NoError(t, err)

	require.NoError(t, sk.OpenKVStore(ctx).Set(rawKey, []byte("i should not be here")))

	// normal reverse pair fails
	err = rp.Walk(ctx, nil, func(denom, address string) (bool, error) {
		return false, nil
	})
	require.ErrorIs(t, err, collections.ErrEncoding)

	// unchecked reverse pair succeeds
	err = uncheckedRp.Walk(ctx, nil, func(indexingKey, indexedKey string) (stop bool, err error) {
		require.Equal(t, "atom", indexingKey)
		return true, nil
	})
	require.NoError(t, err)

	// unchecked reverse pair lazily updates
	err = uncheckedRp.Reference(ctx, collections.Join("address1", "atom"), 0, nil)
	require.NoError(t, err)
	rawValue, err := sk.OpenKVStore(ctx).Get(rawKey)
	require.NoError(t, err)
	require.Equal(t, []byte{}, rawValue)
}
