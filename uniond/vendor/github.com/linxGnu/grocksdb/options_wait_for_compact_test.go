package grocksdb

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestWaitForCompactOptions(t *testing.T) {
	t.Parallel()

	opts := NewWaitForCompactOptions()
	defer opts.Destroy()

	require.False(t, opts.AbortOnPause())
	opts.SetAbortOnPause(true)
	require.True(t, opts.AbortOnPause())

	require.False(t, opts.Flush())
	opts.SetFlush(true)
	require.True(t, opts.Flush())

	require.False(t, opts.CloseDB())
	opts.SetCloseDB(true)
	require.True(t, opts.CloseDB())

	require.EqualValues(t, 0, opts.GetTimeout())
	opts.SetTimeout(1234)
	require.EqualValues(t, 1234, opts.GetTimeout())
}
