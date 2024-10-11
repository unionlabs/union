package codec

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestUntypedValueCodec(t *testing.T) {
	vc := NewUntypedValueCodec(KeyToValueCodec(NewStringKeyCodec[string]()))

	t.Run("encode/decode", func(t *testing.T) {
		_, err := vc.Encode(0)
		require.ErrorIs(t, err, ErrEncoding)
		b, err := vc.Encode("hello")
		require.NoError(t, err)
		value, err := vc.Decode(b)
		require.NoError(t, err)
		require.Equal(t, "hello", value)
	})

	t.Run("json encode/decode", func(t *testing.T) {
		_, err := vc.EncodeJSON(0)
		require.ErrorIs(t, err, ErrEncoding)
		b, err := vc.EncodeJSON("hello")
		require.NoError(t, err)
		value, err := vc.DecodeJSON(b)
		require.NoError(t, err)
		require.Equal(t, "hello", value)
	})

	t.Run("stringify", func(t *testing.T) {
		_, err := vc.Stringify(0)
		require.ErrorIs(t, err, ErrEncoding)
		s, err := vc.Stringify("hello")
		require.NoError(t, err)
		require.Equal(t, "hello", s)
	})
}
