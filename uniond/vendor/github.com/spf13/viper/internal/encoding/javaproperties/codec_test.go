package javaproperties

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// original form of the data.
const original = `#key-value pair
key = value
map.key = value
`

// encoded form of the data.
const encoded = `key = value
map.key = value
`

// data is Viper's internal representation.
var data = map[string]any{
	"key": "value",
	"map": map[string]any{
		"key": "value",
	},
}

func TestCodec_Encode(t *testing.T) {
	codec := Codec{}

	b, err := codec.Encode(data)
	require.NoError(t, err)

	assert.Equal(t, encoded, string(b))
}

func TestCodec_Decode(t *testing.T) {
	t.Run("OK", func(t *testing.T) {
		codec := Codec{}

		v := map[string]any{}

		err := codec.Decode([]byte(original), v)
		require.NoError(t, err)

		assert.Equal(t, data, v)
	})

	t.Run("InvalidData", func(t *testing.T) {
		t.Skip("TODO: needs invalid data example")

		codec := Codec{}

		v := map[string]any{}

		codec.Decode([]byte(``), v)

		assert.Empty(t, v)
	})
}

func TestCodec_DecodeEncode(t *testing.T) {
	codec := Codec{}

	v := map[string]any{}

	err := codec.Decode([]byte(original), v)
	require.NoError(t, err)

	b, err := codec.Encode(data)
	require.NoError(t, err)

	assert.Equal(t, original, string(b))
}
