package collections

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestSequence(t *testing.T) {
	sk, ctx := deps()
	schema := NewSchemaBuilder(sk)
	seq := NewSequence(schema, NewPrefix(0), "sequence")
	// initially the first available number is DefaultSequenceStart
	n, err := seq.Peek(ctx)
	require.NoError(t, err)
	require.Equal(t, DefaultSequenceStart, n)

	// when we call next when sequence is still unset the first expected value is DefaultSequenceStart
	n, err = seq.Next(ctx)
	require.NoError(t, err)
	require.Equal(t, DefaultSequenceStart, n)
	// when we call peek after the first number is set, then the next expected sequence is DefaultSequenceStart + 1
	n, err = seq.Peek(ctx)
	require.NoError(t, err)
	require.Equal(t, DefaultSequenceStart+1, n)

	// set
	err = seq.Set(ctx, 10)
	require.NoError(t, err)
	n, err = seq.Peek(ctx)
	require.NoError(t, err)
	require.Equal(t, n, uint64(10))
}
