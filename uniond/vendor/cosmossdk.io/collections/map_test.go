package collections

import (
	"context"
	"errors"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestMap(t *testing.T) {
	sk, ctx := deps()
	schemaBuilder := NewSchemaBuilder(sk)
	m := NewMap(schemaBuilder, NewPrefix("hi"), "m", Uint64Key, Uint64Value)
	_, err := schemaBuilder.Build()
	require.NoError(t, err)

	// test not has
	has, err := m.Has(ctx, 1)
	require.NoError(t, err)
	require.False(t, has)
	// test get error
	_, err = m.Get(ctx, 1)
	require.ErrorIs(t, err, ErrNotFound)

	// test set/get
	err = m.Set(ctx, 1, 100)
	require.NoError(t, err)
	v, err := m.Get(ctx, 1)
	require.NoError(t, err)
	require.Equal(t, uint64(100), v)

	// test remove
	err = m.Remove(ctx, 1)
	require.NoError(t, err)
	has, err = m.Has(ctx, 1)
	require.NoError(t, err)
	require.False(t, has)
}

func TestMap_Clear(t *testing.T) {
	makeTest := func() (context.Context, Map[uint64, uint64]) {
		sk, ctx := deps()
		m := NewMap(NewSchemaBuilder(sk), NewPrefix(0), "test", Uint64Key, Uint64Value)
		for i := uint64(0); i < clearBatchSize*2; i++ {
			require.NoError(t, m.Set(ctx, i, i))
		}
		return ctx, m
	}

	t.Run("nil ranger", func(t *testing.T) {
		ctx, m := makeTest()
		err := m.Clear(ctx, nil)
		require.NoError(t, err)
		err = m.Walk(ctx, nil, func(key, value uint64) (bool, error) {
			return false, errors.New("should never be called")
		})
		require.NoError(t, err)
	})

	t.Run("custom ranger", func(t *testing.T) {
		ctx, m := makeTest()
		// delete from 0 to 100
		err := m.Clear(ctx, new(Range[uint64]).StartInclusive(0).EndInclusive(100))
		require.NoError(t, err)

		iter, err := m.Iterate(ctx, nil)
		require.NoError(t, err)
		keys, err := iter.Keys()
		require.NoError(t, err)
		require.Len(t, keys, clearBatchSize*2-101)
		require.Equal(t, keys[0], uint64(101))
		require.Equal(t, keys[len(keys)-1], uint64(clearBatchSize*2-1))
	})
}

func TestMap_IterateRaw(t *testing.T) {
	sk, ctx := deps()
	// safety check to ensure prefix boundaries are not crossed
	require.NoError(t, sk.OpenKVStore(ctx).Set([]byte{0x0, 0x0}, []byte("before prefix")))
	require.NoError(t, sk.OpenKVStore(ctx).Set([]byte{0x2, 0x0}, []byte("after prefix")))

	sb := NewSchemaBuilder(sk)

	m := NewMap(sb, NewPrefix(1), "m", Uint64Key, Uint64Value)
	require.NoError(t, m.Set(ctx, 0, 0))
	require.NoError(t, m.Set(ctx, 1, 1))
	require.NoError(t, m.Set(ctx, 2, 2))

	// test non nil end in ascending order
	twoBigEndian, err := EncodeKeyWithPrefix(nil, Uint64Key, 2)
	require.NoError(t, err)
	iter, err := m.IterateRaw(ctx, nil, twoBigEndian, OrderAscending)
	require.NoError(t, err)
	defer iter.Close()

	keys, err := iter.Keys()
	require.NoError(t, err)

	require.Equal(t, []uint64{0, 1}, keys)

	// test nil end in reverse
	iter, err = m.IterateRaw(ctx, nil, nil, OrderDescending)
	require.NoError(t, err)
	defer iter.Close()

	keys, err = iter.Keys()
	require.NoError(t, err)
	require.Equal(t, []uint64{2, 1, 0}, keys)

	// test invalid iter
	_, err = m.IterateRaw(ctx, []byte{0x2, 0x0}, []byte{0x0, 0x0}, OrderAscending)
	require.ErrorIs(t, err, ErrInvalidIterator)

	// test on empty collection iterating does not error
	require.NoError(t, m.Clear(ctx, nil))
	_, err = m.IterateRaw(ctx, nil, nil, OrderAscending)
	require.NoError(t, err)
}

func Test_encodeKey(t *testing.T) {
	prefix := "prefix"
	number := []byte{0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0}
	expectedKey := append([]byte(prefix), number...)

	gotKey, err := EncodeKeyWithPrefix(NewPrefix(prefix).Bytes(), Uint64Key, 0)
	require.NoError(t, err)
	require.Equal(t, expectedKey, gotKey)
}
