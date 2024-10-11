package tracker

import (
	"testing"
	"unsafe"

	"github.com/stretchr/testify/require"
)

func TestEntrySize(t *testing.T) {
	// Validate no regression on the size of entry{}. This is a critical bit for
	// performance of unmarshaling documents. Should only be increased with care
	// and a very good reason.
	maxExpectedEntrySize := 48
	require.LessOrEqual(t, int(unsafe.Sizeof(entry{})), maxExpectedEntrySize)
}
