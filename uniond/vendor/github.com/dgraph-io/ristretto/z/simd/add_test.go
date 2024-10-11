package simd

import (
	"math"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestSearch(t *testing.T) {
	keys := make([]uint64, 512)
	for i := 0; i < len(keys); i += 2 {
		keys[i] = uint64(i)
		keys[i+1] = 1
	}

	for i := 0; i < len(keys); i++ {
		idx := int(Search(keys, uint64(i)))
		require.Equal(t, (i+1)/2, idx, "%v\n%v", i, keys)
	}
	require.Equal(t, 256, int(Search(keys, math.MaxInt64>>1)))
	require.Equal(t, 256, int(Search(keys, math.MaxInt64)))
}
