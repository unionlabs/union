package encoding

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

type decoder struct {
	v map[string]any
}

func (d decoder) Decode(_ []byte, v map[string]any) error {
	for key, value := range d.v {
		v[key] = value
	}

	return nil
}

func TestDecoderRegistry_RegisterDecoder(t *testing.T) {
	t.Run("OK", func(t *testing.T) {
		registry := NewDecoderRegistry()

		err := registry.RegisterDecoder("myformat", decoder{})
		require.NoError(t, err)
	})

	t.Run("AlreadyRegistered", func(t *testing.T) {
		registry := NewDecoderRegistry()

		err := registry.RegisterDecoder("myformat", decoder{})
		require.NoError(t, err)

		err = registry.RegisterDecoder("myformat", decoder{})
		assert.ErrorIs(t, err, ErrDecoderFormatAlreadyRegistered)
	})
}

func TestDecoderRegistry_Decode(t *testing.T) {
	t.Run("OK", func(t *testing.T) {
		registry := NewDecoderRegistry()
		decoder := decoder{
			v: map[string]any{
				"key": "value",
			},
		}

		err := registry.RegisterDecoder("myformat", decoder)
		require.NoError(t, err)

		v := map[string]any{}

		err = registry.Decode("myformat", []byte("key: value"), v)
		require.NoError(t, err)

		assert.Equal(t, decoder.v, v)
	})

	t.Run("DecoderNotFound", func(t *testing.T) {
		registry := NewDecoderRegistry()

		v := map[string]any{}

		err := registry.Decode("myformat", nil, v)
		assert.ErrorIs(t, err, ErrDecoderNotFound)
	})
}
