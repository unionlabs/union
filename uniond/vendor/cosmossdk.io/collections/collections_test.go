package collections

import (
	"context"
	"math"
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/core/store"
	"cosmossdk.io/core/testing"
)

func deps() (store.KVStoreService, context.Context) {
	ctx := coretesting.Context()
	kv := coretesting.KVStoreService(ctx, "test")
	return kv, ctx
}

func TestPrefix(t *testing.T) {
	t.Run("panics on invalid int", func(t *testing.T) {
		require.Panics(t, func() {
			NewPrefix(math.MaxUint8 + 1)
		})
	})

	t.Run("string", func(t *testing.T) {
		require.Equal(t, []byte("prefix"), NewPrefix("prefix").Bytes())
	})

	t.Run("int", func(t *testing.T) {
		require.Equal(t, []byte{0x1}, NewPrefix(1).Bytes())
	})

	t.Run("[]byte", func(t *testing.T) {
		bytes := []byte("prefix")
		prefix := NewPrefix(bytes)
		require.Equal(t, bytes, prefix.Bytes())
		// assert if modification happen they do not propagate to prefix
		bytes[0] = 0x0
		require.Equal(t, []byte("prefix"), prefix.Bytes())
	})
}
