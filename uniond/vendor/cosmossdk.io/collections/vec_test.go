package collections

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestVec(t *testing.T) {
	sk, ctx := deps()
	schemaBuilder := NewSchemaBuilder(sk)
	vec := NewVec(schemaBuilder, NewPrefix(0), "vec", StringValue)
	_, err := schemaBuilder.Build()
	require.NoError(t, err)

	// length when empty
	length, err := vec.Len(ctx)
	require.NoError(t, err)
	require.Equal(t, uint64(0), length)

	// pop when empty should error with an empty vec error
	_, err = vec.Pop(ctx)
	require.ErrorIs(t, err, ErrEmptyVec)

	// replace when out of bounds should error with an out of bounds error
	err = vec.Replace(ctx, 0, "foo")
	require.ErrorIs(t, err, ErrOutOfBounds)

	// get out of bounds should error with an out of bounds error
	_, err = vec.Get(ctx, 0)
	require.ErrorIs(t, err, ErrOutOfBounds)

	// push
	err = vec.Push(ctx, "foo")
	require.NoError(t, err)

	// push more
	err = vec.Push(ctx, "bar")
	require.NoError(t, err)

	// check length
	length, err = vec.Len(ctx)
	require.NoError(t, err)
	require.Equal(t, uint64(2), length)

	// get
	v, err := vec.Get(ctx, 0)
	require.NoError(t, err)
	require.Equal(t, "foo", v)

	// replace
	err = vec.Replace(ctx, 0, "bar")
	require.NoError(t, err)

	v, err = vec.Get(ctx, 0)
	require.NoError(t, err)
	require.Equal(t, "bar", v)

	// pop
	v, err = vec.Pop(ctx)
	require.NoError(t, err)
	require.Equal(t, "bar", v)
}
