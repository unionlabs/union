package collections

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestTryInferIndex(t *testing.T) {
	invalidIdx := 5

	t.Run("not a pointer to struct", func(t *testing.T) {
		_, err := tryInferIndexes[*int, string, string](&invalidIdx)
		require.ErrorIs(t, err, errNotStruct)
	})

	t.Run("not a struct", func(t *testing.T) {
		_, err := tryInferIndexes[int, string, string](invalidIdx)
		require.ErrorIs(t, err, errNotStruct)
	})

	t.Run("not an index field", func(t *testing.T) {
		type invalidIndex struct {
			A int
		}

		_, err := tryInferIndexes[invalidIndex, string, string](invalidIndex{})
		require.ErrorIs(t, err, errNotIndex)
	})
}
