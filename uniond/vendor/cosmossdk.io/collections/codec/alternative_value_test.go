package codec_test

import (
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/collections/codec"
	"cosmossdk.io/collections/colltest"
)

type altValue struct {
	Value uint64 `json:"value"`
}

func TestAltValueCodec(t *testing.T) {
	// we assume we want to migrate the value from json(altValue) to just be
	// the raw value uint64.
	canonical := codec.KeyToValueCodec(codec.NewUint64Key[uint64]())
	alternative := func(v []byte) (uint64, error) {
		var alt altValue
		err := json.Unmarshal(v, &alt)
		if err != nil {
			return 0, err
		}
		return alt.Value, nil
	}

	cdc := codec.NewAltValueCodec(canonical, alternative)

	t.Run("decodes alternative value", func(t *testing.T) {
		expected := uint64(100)
		alternativeEncodedBytes, err := json.Marshal(altValue{Value: expected})
		require.NoError(t, err)
		got, err := cdc.Decode(alternativeEncodedBytes)
		require.NoError(t, err)
		require.Equal(t, expected, got)
	})

	t.Run("decodes canonical value", func(t *testing.T) {
		expected := uint64(100)
		canonicalEncodedBytes, err := cdc.Encode(expected)
		require.NoError(t, err)
		got, err := cdc.Decode(canonicalEncodedBytes)
		require.NoError(t, err)
		require.Equal(t, expected, got)
	})

	t.Run("conformance", func(t *testing.T) {
		colltest.TestValueCodec(t, cdc, uint64(100))
	})
}
