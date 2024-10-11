package collections

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestPair(t *testing.T) {
	keyCodec := PairKeyCodec(StringKey, StringKey)
	t.Run("stringify", func(t *testing.T) {
		s := keyCodec.Stringify(Join("a", "b"))
		require.Equal(t, `("a", "b")`, s)
		s = keyCodec.Stringify(PairPrefix[string, string]("a"))
		require.Equal(t, `("a", <nil>)`, s)
		s = keyCodec.Stringify(Pair[string, string]{})
		require.Equal(t, `(<nil>, <nil>)`, s)
	})

	t.Run("json", func(t *testing.T) {
		b, err := keyCodec.EncodeJSON(Join("k1", "k2"))
		require.NoError(t, err)
		require.Equal(t, []byte(`["k1","k2"]`), b)
	})
}

func TestPairRange(t *testing.T) {
	sk, ctx := deps()
	schema := NewSchemaBuilder(sk)
	pc := PairKeyCodec(StringKey, Uint64Key)
	m := NewMap(schema, NewPrefix(0), "pair", pc, Uint64Value)

	require.NoError(t, m.Set(ctx, Join("A", uint64(0)), 1))
	require.NoError(t, m.Set(ctx, Join("A", uint64(1)), 0))
	require.NoError(t, m.Set(ctx, Join("A", uint64(2)), 0))
	require.NoError(t, m.Set(ctx, Join("B", uint64(3)), 0))

	v, err := m.Get(ctx, Join("A", uint64(0)))
	require.NoError(t, err)
	require.Equal(t, uint64(1), v)

	// EXPECT only A1,2
	iter, err := m.Iterate(ctx, NewPrefixedPairRange[string, uint64]("A").StartInclusive(1).EndInclusive(2))
	require.NoError(t, err)
	keys, err := iter.Keys()
	require.NoError(t, err)
	require.Equal(t, []Pair[string, uint64]{Join("A", uint64(1)), Join("A", uint64(2))}, keys)

	// expect the whole "A" prefix
	iter, err = m.Iterate(ctx, NewPrefixedPairRange[string, uint64]("A"))
	require.NoError(t, err)
	keys, err = iter.Keys()
	require.NoError(t, err)
	require.Equal(t, []Pair[string, uint64]{Join("A", uint64(0)), Join("A", uint64(1)), Join("A", uint64(2))}, keys)

	// expect only A1
	iter, err = m.Iterate(ctx, NewPrefixedPairRange[string, uint64]("A").StartExclusive(0).EndExclusive(2))
	require.NoError(t, err)
	keys, err = iter.Keys()
	require.NoError(t, err)
	require.Equal(t, []Pair[string, uint64]{Join("A", uint64(1))}, keys)

	// expect A2, A1
	iter, err = m.Iterate(ctx, NewPrefixedPairRange[string, uint64]("A").Descending().StartExclusive(0).EndInclusive(2))
	require.NoError(t, err)
	keys, err = iter.Keys()
	require.NoError(t, err)
	require.Equal(t, []Pair[string, uint64]{Join("A", uint64(2)), Join("A", uint64(1))}, keys)
}
