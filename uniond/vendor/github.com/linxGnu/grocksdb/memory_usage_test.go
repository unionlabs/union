package grocksdb

import (
	"crypto/rand"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestMemoryUsage(t *testing.T) {
	t.Parallel()

	// create database with cache
	cache := NewLRUCache(8 * 1024 * 1024)
	cache.SetCapacity(90)

	bbto := NewDefaultBlockBasedTableOptions()
	bbto.SetBlockCache(cache)
	defer cache.Destroy()

	rowCache := NewLRUCache(8 * 1024 * 1024)
	defer rowCache.Destroy()

	blobCache := NewLRUCache(8 * 1024 * 1024)
	defer blobCache.Destroy()

	applyOpts := func(opts *Options) {
		opts.SetBlockBasedTableFactory(bbto)
		opts.SetRowCache(rowCache)
		opts.SetBlobCache(blobCache)
	}

	db := newTestDB(t, applyOpts)
	defer db.Close()

	// take first memory usage snapshot
	mu1, err := GetApproximateMemoryUsageByType([]*DB{db}, []*Cache{cache})
	require.Nil(t, err)

	// perform IO operations that will affect in-memory tables (and maybe cache as well)
	wo := NewDefaultWriteOptions()
	defer wo.Destroy()
	ro := NewDefaultReadOptions()
	defer ro.Destroy()

	key := []byte("key")
	value := make([]byte, 1024)
	_, err = rand.Read(value)
	require.Nil(t, err)

	err = db.Put(wo, key, value)
	require.Nil(t, err)
	_, err = db.Get(ro, key)
	require.Nil(t, err)

	// take second memory usage snapshot
	mu2, err := GetApproximateMemoryUsageByType([]*DB{db}, []*Cache{cache})
	require.Nil(t, err)

	// the amount of memory used by memtables should increase after write/read;
	// cache memory usage is not likely to be changed, perhaps because requested key is kept by memtable
	assert.True(t, mu2.CacheTotal >= mu1.CacheTotal)
	assert.True(t, mu2.MemTableReadersTotal >= mu1.MemTableReadersTotal)

	// check cached
	require.EqualValues(t, 0, rowCache.GetPinnedUsage())
	require.EqualValues(t, 0, rowCache.GetUsage())
}
