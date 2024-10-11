package grocksdb

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestFlushOption(t *testing.T) {
	t.Parallel()

	fo := NewDefaultFlushOptions()
	defer fo.Destroy()

	require.EqualValues(t, true, fo.IsWait())
	fo.SetWait(false)
	require.EqualValues(t, false, fo.IsWait())
}
