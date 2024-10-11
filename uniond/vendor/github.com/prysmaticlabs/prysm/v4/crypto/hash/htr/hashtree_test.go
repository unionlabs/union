package htr

import (
	"sync"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func Test_VectorizedSha256(t *testing.T) {
	largeSlice := make([][32]byte, 32*minSliceSizeToParallelize)
	secondLargeSlice := make([][32]byte, 32*minSliceSizeToParallelize)
	hash1 := make([][32]byte, 16*minSliceSizeToParallelize)
	wg := sync.WaitGroup{}
	wg.Add(1)
	go func() {
		defer wg.Done()
		tempHash := VectorizedSha256(largeSlice)
		copy(hash1, tempHash)
	}()
	wg.Wait()
	hash2 := VectorizedSha256(secondLargeSlice)
	require.Equal(t, len(hash1), len(hash2))
	for i, r := range hash1 {
		require.Equal(t, r, hash2[i])
	}
}
