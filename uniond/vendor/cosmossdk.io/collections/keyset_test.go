package collections

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestKeySet(t *testing.T) {
	sk, ctx := deps()
	schema := NewSchemaBuilder(sk)
	ks := NewKeySet(schema, NewPrefix("keyset"), "keyset", StringKey)

	// set
	require.NoError(t, ks.Set(ctx, "A"))
	require.NoError(t, ks.Set(ctx, "B"))
	require.NoError(t, ks.Set(ctx, "C"))
	require.NoError(t, ks.Set(ctx, "CC"))

	// exists
	exists, err := ks.Has(ctx, "CC")
	require.NoError(t, err)
	require.True(t, exists)

	// remove
	err = ks.Remove(ctx, "A")
	require.NoError(t, err)

	// non exists
	exists, err = ks.Has(ctx, "A")
	require.NoError(t, err)
	require.False(t, exists)

	// iter
	iter, err := ks.Iterate(ctx, nil)
	require.NoError(t, err)

	// iter next
	iter.Next()

	// iter key
	key, err := iter.Key()
	require.NoError(t, err)
	require.Equal(t, "C", key)

	// iter keys
	keys, err := iter.Keys()
	require.NoError(t, err)
	require.Equal(t, []string{"C", "CC"}, keys)

	// validity
	require.False(t, iter.Valid())
}

func Test_noValue(t *testing.T) {
	require.Equal(t, noValueValueType, noValueCodec.ValueType())
	require.Equal(t, noValueValueType, noValueCodec.Stringify(NoValue{}))

	b, err := noValueCodec.Encode(NoValue{})
	require.NoError(t, err)
	require.Equal(t, []byte{}, b)

	nv, err := noValueCodec.Decode(b)
	require.NoError(t, err)
	require.Equal(t, NoValue{}, nv)

	_, err = noValueCodec.Decode([]byte("bad"))
	require.ErrorIs(t, err, ErrEncoding)
}

func TestUncheckedKeySet(t *testing.T) {
	sk, ctx := deps()
	schema := NewSchemaBuilder(sk)
	uncheckedKs := NewKeySet(schema, NewPrefix("keyset"), "keyset", StringKey, WithKeySetUncheckedValue())
	ks := NewKeySet(schema, NewPrefix("keyset"), "keyset", StringKey)
	// we set a NoValue unfriendly value.
	require.NoError(t, sk.OpenKVStore(ctx).Set([]byte("keyset1"), []byte("A")))
	require.NoError(t, sk.OpenKVStore(ctx).Set([]byte("keyset2"), []byte("B")))

	// the standard KeySet errors here, because it doesn't like the fact that the value is []byte("A")
	// and not []byte{}.
	err := ks.Walk(ctx, nil, func(key string) (stop bool, err error) {
		return true, nil
	})
	require.ErrorIs(t, err, ErrEncoding)

	// the unchecked KeySet doesn't care about the value, so it works.
	err = uncheckedKs.Walk(ctx, nil, func(key string) (stop bool, err error) {
		require.Equal(t, "1", key)
		return true, nil
	})
	require.NoError(t, err)

	// now we set it again
	require.NoError(t, uncheckedKs.Set(ctx, "1"))
	// and we will see that the value which was []byte("A") has been cleared to be []byte{}
	raw, err := sk.OpenKVStore(ctx).Get([]byte("keyset1"))
	require.NoError(t, err)
	require.Equal(t, []byte{}, raw)
}
