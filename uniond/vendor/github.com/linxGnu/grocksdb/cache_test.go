package grocksdb

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestLRUCache(t *testing.T) {
	t.Parallel()

	cache := NewLRUCache(19)
	defer cache.Destroy()

	require.EqualValues(t, 19, cache.GetCapacity())
	cache.SetCapacity(128)
	require.EqualValues(t, 128, cache.GetCapacity())

	cache.DisownData()
}

func TestHyperClockCache(t *testing.T) {
	t.Parallel()

	cache := NewHyperClockCache(100, 10)
	defer cache.Destroy()

	require.EqualValues(t, 100, cache.GetCapacity())
	cache.SetCapacity(128)
	require.EqualValues(t, 128, cache.GetCapacity())

	cache.DisownData()
}

func TestLRUCacheWithOpts(t *testing.T) {
	t.Parallel()

	opts := NewLRUCacheOptions()
	opts.SetCapacity(19)
	opts.SetNumShardBits(2)
	defer opts.Destroy()

	cache := NewLRUCacheWithOptions(opts)
	defer cache.Destroy()

	require.EqualValues(t, 19, cache.GetCapacity())
	cache.SetCapacity(128)
	require.EqualValues(t, 128, cache.GetCapacity())

	cache.DisownData()
}

func TestHyperClockCacheWithOpts(t *testing.T) {
	t.Parallel()

	opts := NewHyperClockCacheOptions(100, 10)
	opts.SetCapacity(19)
	opts.SetEstimatedEntryCharge(10)
	opts.SetNumShardBits(2)
	defer opts.Destroy()

	cache := NewHyperClockCacheWithOpts(opts)
	defer cache.Destroy()

	require.EqualValues(t, 19, cache.GetCapacity())
	cache.SetCapacity(128)
	require.EqualValues(t, 128, cache.GetCapacity())

	cache.DisownData()
}
