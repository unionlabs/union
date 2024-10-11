package codec

import (
	"bytes"
	"sort"
	"testing"

	"github.com/stretchr/testify/require"
	"pgregory.net/rapid"
)

// TestInt64Keys creates a random slice of int64. They're sorted, and then
// they're encoded to bytes. It ensures proper ordering of their bytes
// representation such as bytes(int64::lowest) <= bytes(int64(x)) <= bytes(int64::highest)
func TestInt64Keys(t *testing.T) {
	kc := NewInt64Key[int64]()
	rapid.Check(t, func(t *rapid.T) {
		slice := rapid.SliceOfN(rapid.Int64(), 50_000, 100_000).Draw(t, "random ints")
		sort.Slice(slice, func(i, j int) bool {
			return slice[i] < slice[j]
		})

		var current []byte
		for _, i := range slice {
			next := make([]byte, kc.Size(i))
			_, err := kc.Encode(next, i)
			require.NoError(t, err)
			cmp := bytes.Compare(current, next)
			require.True(t, cmp == 0 || cmp == -1)
			current = next
		}
	})
}

// TestInt32Keys applies the same logic as TestInt64Keys
func TestInt32Keys(t *testing.T) {
	kc := NewInt32Key[int32]()
	rapid.Check(t, func(t *rapid.T) {
		slice := rapid.SliceOfN(rapid.Int32(), 50_000, 100_000).Draw(t, "random ints")
		sort.Slice(slice, func(i, j int) bool {
			return slice[i] < slice[j]
		})

		var current []byte
		for _, i := range slice {
			next := make([]byte, kc.Size(i))
			_, err := kc.Encode(next, i)
			require.NoError(t, err)
			cmp := bytes.Compare(current, next)
			require.True(t, cmp == 0 || cmp == -1)
			current = next
		}
	})
}
