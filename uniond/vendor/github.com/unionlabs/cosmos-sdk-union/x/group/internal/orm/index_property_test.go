package orm

import (
	"testing"

	"github.com/stretchr/testify/require"
	"pgregory.net/rapid"
)

func TestPrefixRangeProperty(t *testing.T) {
	t.Run("TestPrefixRange", rapid.MakeCheck(func(t *rapid.T) {
		prefix := rapid.SliceOf(rapid.Byte()).Draw(t, "prefix")

		start, end := PrefixRange(prefix)

		// len(prefix) == 0 => start == nil && end == nil
		if len(prefix) == 0 {
			require.Nil(t, start)
			require.Nil(t, end)
		} else {
			// start == prefix
			require.Equal(t, prefix, start)

			// Would overflow if all bytes are 255
			wouldOverflow := true
			for _, b := range prefix {
				if b != 255 {
					wouldOverflow = false
				}
			}

			// Overflow => end == nil
			if wouldOverflow {
				require.Nil(t, end)
			} else {
				require.Equal(t, len(start), len(end))

				// Scan back and find last value that isn't 255
				overflowIndex := len(start) - 1
				for overflowIndex > 0 && prefix[overflowIndex] == 255 {
					overflowIndex--
				}

				// bytes should be the same up to overflow
				// index, one greater at overflow and 0 from
				// then on
				for i, b := range start {
					if i < overflowIndex {
						require.Equal(t, b, end[i])
					} else if i == overflowIndex {
						require.Equal(t, b+1, end[i])
					} else {
						require.Equal(t, uint8(0), end[i])
					}
				}
			}
		}
	}))
}
