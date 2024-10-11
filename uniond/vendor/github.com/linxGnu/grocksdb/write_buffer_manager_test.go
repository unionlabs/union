package grocksdb

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestWriteBufferManager(t *testing.T) {
	t.Parallel()

	wbm := NewWriteBufferManager(12345, false)
	defer wbm.Destroy()

	require.True(t, wbm.Enabled())
	require.False(t, wbm.CostToCache())
	require.EqualValues(t, 0, wbm.MemoryUsage())
	require.EqualValues(t, 0, wbm.MemtableMemoryUsage())
	require.EqualValues(t, 0, wbm.DummyEntriesInCacheUsage())

	wbm.ToggleAllowStall(true)

	require.Equal(t, 12345, wbm.BufferSize())
	wbm.SetBufferSize(123456)
	require.Equal(t, 123456, wbm.BufferSize())
}

func TestWriteBufferManagerWithCache(t *testing.T) {
	t.Parallel()

	cache := NewLRUCache(123)
	defer cache.Destroy()

	wbm := NewWriteBufferManagerWithCache(12345, cache, false)
	defer wbm.Destroy()

	require.True(t, wbm.Enabled())
	require.True(t, wbm.CostToCache())
	require.EqualValues(t, 0, wbm.MemoryUsage())
	require.EqualValues(t, 0, wbm.MemtableMemoryUsage())
	require.EqualValues(t, 0, wbm.DummyEntriesInCacheUsage())

	wbm.ToggleAllowStall(true)

	require.Equal(t, 12345, wbm.BufferSize())
	wbm.SetBufferSize(123456)
	require.Equal(t, 123456, wbm.BufferSize())
}
