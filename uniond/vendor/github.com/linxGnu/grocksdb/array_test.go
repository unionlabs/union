package grocksdb

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestBytesToCSlice(t *testing.T) {
	t.Parallel()

	v, err := byteSlicesToCSlices(nil)
	require.Nil(t, v)
	require.Nil(t, err)
}
