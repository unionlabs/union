package ini

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// original form of the data.
const original = `; key-value pair
key=value ; key-value pair

# map
[map] # map
key=%(key)s

`

// encoded form of the data.
const encoded = `key=value

[map]
key=value
`

// decoded form of the data.
//
// In case of INI it's slightly different from Viper's internal representation
// (e.g. top level keys land in a section called default).
var decoded = map[string]any{
	"DEFAULT": map[string]any{
		"key": "value",
	},
	"map": map[string]any{
		"key": "value",
	},
}

// data is Viper's internal representation.
var data = map[string]any{
	"key": "value",
	"map": map[string]any{
		"key": "value",
	},
}

func TestCodec_Encode(t *testing.T) {
	t.Run("OK", func(t *testing.T) {
		codec := Codec{}

		b, err := codec.Encode(data)
		require.NoError(t, err)

		assert.Equal(t, encoded, string(b))
	})

	t.Run("Default", func(t *testing.T) {
		codec := Codec{}

		data := map[string]any{
			"default": map[string]any{
				"key": "value",
			},
			"map": map[string]any{
				"key": "value",
			},
		}

		b, err := codec.Encode(data)
		require.NoError(t, err)

		assert.Equal(t, encoded, string(b))
	})
}

func TestCodec_Decode(t *testing.T) {
	t.Run("OK", func(t *testing.T) {
		codec := Codec{}

		v := map[string]any{}

		err := codec.Decode([]byte(original), v)
		require.NoError(t, err)

		assert.Equal(t, decoded, v)
	})

	t.Run("InvalidData", func(t *testing.T) {
		codec := Codec{}

		v := map[string]any{}

		err := codec.Decode([]byte(`invalid data`), v)
		require.Error(t, err)

		t.Logf("decoding failed as expected: %s", err)
	})
}
