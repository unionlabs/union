package toml

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// original form of the data.
const original = `# key-value pair
key = "value"
list = ["item1", "item2", "item3"]

[map]
key = "value"

# nested
# map
[nested_map]
[nested_map.map]
key = "value"
list = [
  "item1",
  "item2",
  "item3",
]
`

// encoded form of the data.
const encoded = `key = 'value'
list = ['item1', 'item2', 'item3']

[map]
key = 'value'

[nested_map]
[nested_map.map]
key = 'value'
list = ['item1', 'item2', 'item3']
`

// data is Viper's internal representation.
var data = map[string]any{
	"key": "value",
	"list": []any{
		"item1",
		"item2",
		"item3",
	},
	"map": map[string]any{
		"key": "value",
	},
	"nested_map": map[string]any{
		"map": map[string]any{
			"key": "value",
			"list": []any{
				"item1",
				"item2",
				"item3",
			},
		},
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
		codec := Codec{}

		v := map[string]any{}

		err := codec.Decode([]byte(`invalid data`), v)
		require.Error(t, err)

		t.Logf("decoding failed as expected: %s", err)
	})
}
