package encoding

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

type encoder struct {
	b []byte
}

func (e encoder) Encode(_ map[string]any) ([]byte, error) {
	return e.b, nil
}

func TestEncoderRegistry_RegisterEncoder(t *testing.T) {
	t.Run("OK", func(t *testing.T) {
		registry := NewEncoderRegistry()

		err := registry.RegisterEncoder("myformat", encoder{})
		require.NoError(t, err)
	})

	t.Run("AlreadyRegistered", func(t *testing.T) {
		registry := NewEncoderRegistry()

		err := registry.RegisterEncoder("myformat", encoder{})
		require.NoError(t, err)

		err = registry.RegisterEncoder("myformat", encoder{})
		assert.ErrorIs(t, err, ErrEncoderFormatAlreadyRegistered)
	})
}

func TestEncoderRegistry_Decode(t *testing.T) {
	t.Run("OK", func(t *testing.T) {
		registry := NewEncoderRegistry()
		encoder := encoder{
			b: []byte("key: value"),
		}

		err := registry.RegisterEncoder("myformat", encoder)
		require.NoError(t, err)

		b, err := registry.Encode("myformat", map[string]any{"key": "value"})
		require.NoError(t, err)

		assert.Equal(t, "key: value", string(b))
	})

	t.Run("EncoderNotFound", func(t *testing.T) {
		registry := NewEncoderRegistry()

		_, err := registry.Encode("myformat", map[string]any{"key": "value"})
		assert.ErrorIs(t, err, ErrEncoderNotFound)
	})
}
