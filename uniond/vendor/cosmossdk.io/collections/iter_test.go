package collections

import (
	"errors"
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestIteratorBasic(t *testing.T) {
	sk, ctx := deps()
	// safety check to ensure that iteration does not cross prefix boundaries
	err := sk.OpenKVStore(ctx).Set([]byte{0, 0}, []byte("before prefix"))
	require.NoError(t, err)
	err = sk.OpenKVStore(ctx).Set([]byte{2, 1}, []byte("after prefix"))
	require.NoError(t, err)
	schemaBuilder := NewSchemaBuilder(sk)
	m := NewMap(schemaBuilder, NewPrefix(1), "m", StringKey, Uint64Value)
	_, err = schemaBuilder.Build()
	require.NoError(t, err)

	for i := uint64(1); i <= 2; i++ {
		require.NoError(t, m.Set(ctx, fmt.Sprintf("%d", i), i))
	}

	iter, err := m.Iterate(ctx, nil)
	require.NoError(t, err)
	defer iter.Close()

	// key codec
	key, err := iter.Key()
	require.NoError(t, err)
	require.Equal(t, "1", key)

	// value codec
	value, err := iter.Value()
	require.NoError(t, err)
	require.Equal(t, uint64(1), value)

	// assert expected prefixing on iter
	require.Equal(t, m.prefix, iter.iter.Key()[:len(m.prefix)])

	// advance iter
	iter.Next()
	require.True(t, iter.Valid())

	// key 2
	key, err = iter.Key()
	require.NoError(t, err)
	require.Equal(t, "2", key)

	// value 2
	value, err = iter.Value()
	require.NoError(t, err)
	require.Equal(t, uint64(2), value)

	// call next, invalid
	iter.Next()
	require.False(t, iter.Valid())
	// close no errors
	require.NoError(t, iter.Close())
}

func TestIteratorKeyValues(t *testing.T) {
	sk, ctx := deps()
	schemaBuilder := NewSchemaBuilder(sk)
	m := NewMap(schemaBuilder, NewPrefix("some super amazing prefix"), "m", StringKey, Uint64Value)
	_, err := schemaBuilder.Build()
	require.NoError(t, err)

	for i := uint64(0); i <= 5; i++ {
		require.NoError(t, m.Set(ctx, fmt.Sprintf("%d", i), i))
	}

	// test keys
	iter, err := m.Iterate(ctx, nil)
	require.NoError(t, err)
	keys, err := iter.Keys()
	require.NoError(t, err)

	for i, key := range keys {
		require.Equal(t, fmt.Sprintf("%d", i), key)
	}
	require.NoError(t, iter.Close())
	require.False(t, iter.Valid())

	// test values
	iter, err = m.Iterate(ctx, nil)
	require.NoError(t, err)
	values, err := iter.Values()
	require.NoError(t, err)

	for i, value := range values {
		require.Equal(t, uint64(i), value)
	}
	require.NoError(t, iter.Close())
	require.False(t, iter.Valid())

	// test key value pairings
	iter, err = m.Iterate(ctx, nil)
	require.NoError(t, err)
	kvs, err := iter.KeyValues()
	require.NoError(t, err)

	for i, kv := range kvs {
		require.Equal(t, fmt.Sprintf("%d", i), kv.Key)
		require.Equal(t, uint64(i), kv.Value)
	}
	require.NoError(t, iter.Close())
	require.False(t, iter.Valid())
}

func TestIteratorPrefixing(t *testing.T) {
	sk, ctx := deps()
	schemaBuilder := NewSchemaBuilder(sk)
	m := NewMap(schemaBuilder, NewPrefix("cool"), "cool", StringKey, Uint64Value)
	_, err := schemaBuilder.Build()
	require.NoError(t, err)

	require.NoError(t, m.Set(ctx, "A1", 11))
	require.NoError(t, m.Set(ctx, "A2", 12))
	require.NoError(t, m.Set(ctx, "B1", 21))

	iter, err := m.Iterate(ctx, new(Range[string]).Prefix("A"))
	require.NoError(t, err)
	keys, err := iter.Keys()
	require.NoError(t, err)
	require.Equal(t, []string{"A1", "A2"}, keys)
}

func TestIteratorRanging(t *testing.T) {
	sk, ctx := deps()
	schemaBuilder := NewSchemaBuilder(sk)
	m := NewMap(schemaBuilder, NewPrefix("cool"), "cool", Uint64Key, Uint64Value)
	_, err := schemaBuilder.Build()
	require.NoError(t, err)

	for i := uint64(0); i <= 7; i++ {
		require.NoError(t, m.Set(ctx, i, i))
	}

	// let's range (1-5]; expected: 2..5
	iter, err := m.Iterate(ctx, (&Range[uint64]{}).StartExclusive(1).EndInclusive(5))
	require.NoError(t, err)
	result, err := iter.Keys()
	require.NoError(t, err)
	require.Equal(t, []uint64{2, 3, 4, 5}, result)

	// let's range [1-5); expected 1..4
	iter, err = m.Iterate(ctx, (&Range[uint64]{}).StartInclusive(1).EndExclusive(5))
	require.NoError(t, err)
	result, err = iter.Keys()
	require.NoError(t, err)
	require.Equal(t, []uint64{1, 2, 3, 4}, result)

	// let's range [1-5) descending; expected 4..1
	iter, err = m.Iterate(ctx, (&Range[uint64]{}).StartInclusive(1).EndExclusive(5).Descending())
	require.NoError(t, err)
	result, err = iter.Keys()
	require.NoError(t, err)
	require.Equal(t, []uint64{4, 3, 2, 1}, result)

	// test iterator invalid
	_, err = m.Iterate(ctx, new(Range[uint64]).StartInclusive(10).EndInclusive(1))
	require.ErrorIs(t, err, ErrInvalidIterator)
}

func TestWalk(t *testing.T) {
	sk, ctx := deps()
	schemaBuilder := NewSchemaBuilder(sk)
	m := NewMap(schemaBuilder, NewPrefix("cool"), "cool", Uint64Key, Uint64Value)
	_, err := schemaBuilder.Build()
	require.NoError(t, err)

	for i := uint64(0); i <= 7; i++ {
		require.NoError(t, m.Set(ctx, i, i))
	}

	u := uint64(0)
	err = m.Walk(ctx, nil, func(key, value uint64) (bool, error) {
		if key == 5 {
			return true, nil
		}
		require.Equal(t, u, key)
		require.Equal(t, u, value)
		u++
		return false, nil
	})
	require.NoError(t, err)

	sentinelErr := errors.New("sentinel error")
	err = m.Walk(ctx, nil, func(key, value uint64) (stop bool, err error) {
		require.LessOrEqual(t, key, uint64(3)) // asserts that after the number three we stop
		if key == 3 {
			return false, sentinelErr
		}
		return false, nil
	})
	require.ErrorIs(t, err, sentinelErr) // asserts correct error propagation
}
