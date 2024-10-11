package grocksdb

import (
	"os"
	"runtime"
	"strconv"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestOpenDb(t *testing.T) {
	t.Parallel()

	db := newTestDB(t, nil)
	defer db.Close()

	require.EqualValues(t, "0", db.GetProperty("rocksdb.num-immutable-mem-table"))
	v, success := db.GetIntProperty("rocksdb.num-immutable-mem-table")
	require.EqualValues(t, uint64(0), v)
	require.True(t, success)
}

func TestSetOptions(t *testing.T) {
	t.Parallel()

	db := newTestDB(t, nil)
	defer db.Close()

	for i := 0; i < 100; i++ {
		require.Error(t, db.SetOptions([]string{"a"}, []string{"b"}))
		runtime.GC()
	}
}

func TestSetOptionsCF(t *testing.T) {
	t.Parallel()

	db, cfh, cleanup := newTestDBMultiCF(t, []string{"default", "custom"}, nil)
	defer cleanup()

	for i := 0; i < 100; i++ {
		require.Error(t, db.SetOptionsCF(cfh[1], []string{"a"}, []string{"b"}))
		runtime.GC()
	}
}

func TestDBCRUD(t *testing.T) {
	t.Parallel()

	db := newTestDB(t, nil)
	defer db.Close()

	var (
		givenKey  = []byte("hello")
		givenVal1 = []byte("")
		givenVal2 = []byte("world1")
		wo        = NewDefaultWriteOptions()
		ro        = NewDefaultReadOptions()
	)

	// create
	require.Nil(t, db.Put(wo, givenKey, givenVal1))

	// retrieve
	v1, err := db.Get(ro, givenKey)
	defer v1.Free()
	require.Nil(t, err)
	require.EqualValues(t, v1.Data(), givenVal1)

	// retrieve bytes
	_v1, err := db.GetBytes(ro, givenKey)
	require.Nil(t, err)
	require.EqualValues(t, _v1, givenVal1)

	// update
	require.Nil(t, db.Put(wo, givenKey, givenVal2))
	v2, err := db.Get(ro, givenKey)
	defer v2.Free()
	require.Nil(t, err)
	require.EqualValues(t, v2.Data(), givenVal2)

	// retrieve pinned
	for i := 0; i < 1000; i++ {
		v3, e := db.GetPinned(ro, givenKey)
		require.Nil(t, e)
		require.EqualValues(t, v3.Data(), givenVal2)
		v3.Destroy()
		v3.Destroy()

		v3NE, e := db.GetPinned(ro, []byte("justFake"))
		require.Nil(t, e)
		require.False(t, v3NE.Exists())
		v3NE.Destroy()
		v3NE.Destroy()
	}

	// delete
	require.Nil(t, db.Delete(wo, givenKey))
	v4, err := db.Get(ro, givenKey)
	require.Nil(t, err)
	require.True(t, v4.Data() == nil)

	// retrieve missing pinned
	v5, err := db.GetPinned(ro, givenKey)
	defer v5.Destroy()
	require.Nil(t, err)
	require.True(t, v5.Data() == nil)
}

func TestDBCRUDDBPaths(t *testing.T) {
	t.Parallel()

	names := make([]string, 4)
	targetSizes := make([]uint64, len(names))

	for i := range names {
		names[i] = "TestDBGet_" + strconv.FormatInt(int64(i), 10)
		targetSizes[i] = uint64(1024 * 1024 * (i + 1))
	}

	db := newTestDBPathNames(t, names, targetSizes, nil)
	defer db.Close()

	var (
		givenKey  = []byte("hello")
		givenVal1 = []byte("")
		givenVal2 = []byte("world1")
		givenVal3 = []byte("world2")
		wo        = NewDefaultWriteOptions()
		ro        = NewDefaultReadOptions()
	)

	// retrieve before create
	noexist, err := db.Get(ro, givenKey)
	defer noexist.Free()
	require.Nil(t, err)
	require.False(t, noexist.Exists())
	require.EqualValues(t, noexist.Data(), []byte(nil))

	// create
	require.Nil(t, db.Put(wo, givenKey, givenVal1))

	// retrieve
	v1, err := db.Get(ro, givenKey)
	defer v1.Free()
	require.Nil(t, err)
	require.True(t, v1.Exists())
	require.EqualValues(t, v1.Data(), givenVal1)

	// update
	require.Nil(t, db.Put(wo, givenKey, givenVal2))
	v2, err := db.Get(ro, givenKey)
	defer v2.Free()
	require.Nil(t, err)
	require.True(t, v2.Exists())
	require.EqualValues(t, v2.Data(), givenVal2)

	// update
	require.Nil(t, db.Put(wo, givenKey, givenVal3))
	v3, err := db.Get(ro, givenKey)
	defer v3.Free()
	require.Nil(t, err)
	require.True(t, v3.Exists())
	require.EqualValues(t, v3.Data(), givenVal3)

	{
		v4 := db.KeyMayExists(ro, givenKey, "")
		defer v4.Free()
		require.True(t, v4.Size() > 0)
	}

	// delete
	require.Nil(t, db.SingleDelete(wo, givenKey))
	v4, err := db.Get(ro, givenKey)
	defer v4.Free()
	require.Nil(t, err)
	require.False(t, v4.Exists())
	require.EqualValues(t, v4.Data(), []byte(nil))
}

func newTestDB(t *testing.T, applyOpts func(opts *Options)) *DB {
	dir := t.TempDir()

	opts := NewDefaultOptions()
	// test the ratelimiter
	rateLimiter := NewRateLimiter(1024, 100*1000, 10)
	opts.SetRateLimiter(rateLimiter)
	opts.SetCreateIfMissing(true)
	opts.SetCompression(ZSTDCompression)
	if applyOpts != nil {
		applyOpts(opts)
	}
	db, err := OpenDb(opts, dir)
	require.Nil(t, err)

	db.EnableManualCompaction()
	db.DisableManualCompaction()

	return db
}

func newTestDBAndOpts(t *testing.T, applyOpts func(opts *Options)) (*DB, *Options) {
	dir := t.TempDir()

	opts := NewDefaultOptions()
	// test the ratelimiter
	rateLimiter := NewAutoTunedRateLimiter(1024, 100*1000, 10)
	opts.SetRateLimiter(rateLimiter)
	opts.SetCreateIfMissing(true)
	opts.SetCompression(ZSTDCompression)
	if applyOpts != nil {
		applyOpts(opts)
	}
	db, err := OpenDb(opts, dir)
	require.Nil(t, err)

	return db, opts
}

func newTestDBMultiCF(t *testing.T, columns []string, applyOpts func(opts *Options)) (db *DB, cfh []*ColumnFamilyHandle, cleanup func()) {
	dir := t.TempDir()

	opts := NewDefaultOptions()
	rateLimiter := NewGenericRateLimiter(1024, 100*1000, 10, RateLimiterModeAllIo, true)
	opts.SetRateLimiter(rateLimiter)
	opts.SetCreateIfMissingColumnFamilies(true)
	opts.SetCreateIfMissing(true)
	opts.SetCompression(ZSTDCompression)
	opts.SetSkipCheckingSSTFileSizesOnDBOpen(true)
	opts.SetRateLimiter(NewRateLimiter(2<<30, 1<<20, 100<<20))
	opts.SetUniversalCompactionOptions(NewDefaultUniversalCompactionOptions())

	options := make([]*Options, len(columns))
	for i := range options {
		options[i] = opts
	}

	if applyOpts != nil {
		for _, opts := range options {
			applyOpts(opts)
		}
	}

	db, cfh, err := OpenDbColumnFamilies(opts, dir, columns, options)
	require.Nil(t, err)
	cleanup = func() {
		for _, cf := range cfh {
			cf.Destroy()
		}
		db.Close()
	}
	return db, cfh, cleanup
}

func newTestDBPathNames(t *testing.T, names []string, targetSizes []uint64, applyOpts func(opts *Options)) *DB {
	require.EqualValues(t, len(targetSizes), len(names))
	require.True(t, len(names) > 0)

	dir := t.TempDir()

	paths := make([]string, len(names))
	for i, name := range names {
		directory, e := os.MkdirTemp(dir, "gorocksdb-"+name)
		require.Nil(t, e)
		paths[i] = directory
	}

	dbpaths := NewDBPathsFromData(paths, targetSizes)
	defer DestroyDBPaths(dbpaths)

	opts := NewDefaultOptions()
	opts.SetDBPaths(dbpaths)
	opts.SetCFPaths(dbpaths)

	// test the ratelimiter
	rateLimiter := NewRateLimiter(1024, 100*1000, 10)
	opts.SetRateLimiter(rateLimiter)
	opts.SetCreateIfMissing(true)
	if applyOpts != nil {
		applyOpts(opts)
	}
	db, err := OpenDb(opts, dir)
	require.Nil(t, err)

	return db
}

func TestDBMultiGet(t *testing.T) {
	t.Parallel()

	db := newTestDB(t, nil)
	defer db.Close()

	var (
		givenKey1 = []byte("hello1")
		givenKey2 = []byte("hello2")
		givenKey3 = []byte("hello3")
		givenVal1 = []byte("world1")
		givenVal2 = []byte("world2")
		givenVal3 = []byte("world3")
		wo        = NewDefaultWriteOptions()
		ro        = NewDefaultReadOptions()
	)

	// create
	require.Nil(t, db.Put(wo, givenKey1, givenVal1))
	require.Nil(t, db.Put(wo, givenKey2, givenVal2))
	require.Nil(t, db.Put(wo, givenKey3, givenVal3))

	// retrieve
	values, err := db.MultiGet(ro, []byte("noexist"), givenKey1, givenKey2, givenKey3)
	defer values.Destroy()
	require.Nil(t, err)
	require.EqualValues(t, len(values), 4)

	require.EqualValues(t, values[0].Data(), []byte(nil))
	require.EqualValues(t, values[1].Data(), givenVal1)
	require.EqualValues(t, values[2].Data(), givenVal2)
	require.EqualValues(t, values[3].Data(), givenVal3)

	waitForCompactOpts := NewWaitForCompactOptions()
	defer waitForCompactOpts.Destroy()

	err = db.WaitForCompact(waitForCompactOpts)
	require.NoError(t, err)
}

func TestLoadLatestOpts(t *testing.T) {
	t.Parallel()

	dir := t.TempDir()

	opts := NewDefaultOptions()
	defer opts.Destroy()

	opts.SetCreateIfMissing(true)
	for i := 0; i < 100; i++ {
		opts.SetEnv(NewDefaultEnv())
	}

	db, err := OpenDb(opts, dir)
	require.NoError(t, err)
	_, err = db.CreateColumnFamily(opts, "abc")
	require.NoError(t, err)
	require.NoError(t, db.Flush(NewDefaultFlushOptions()))
	db.Close()

	for i := 0; i < 10; i++ {
		o, err := LoadLatestOptions(dir, NewDefaultEnv(), true, NewLRUCache(1))
		runtime.GC()
		require.NoError(t, err)
		require.NotEmpty(t, o.ColumnFamilyNames())
		require.NotEmpty(t, o.ColumnFamilyOpts())
		o.Destroy()
		runtime.GC()
	}

	_, err = LoadLatestOptions("", nil, true, nil)
	require.Error(t, err)
}

func TestDBGetApproximateSizes(t *testing.T) {
	t.Parallel()

	db := newTestDB(t, nil)
	defer db.Close()

	// no ranges
	sizes, err := db.GetApproximateSizes(nil)
	require.EqualValues(t, len(sizes), 0)
	require.NoError(t, err)

	// range will nil start and limit
	sizes, err = db.GetApproximateSizes([]Range{{Start: nil, Limit: nil}})
	require.EqualValues(t, sizes, []uint64{0})
	require.NoError(t, err)

	// valid range
	sizes, err = db.GetApproximateSizes([]Range{{Start: []byte{0x00}, Limit: []byte{0xFF}}})
	require.EqualValues(t, sizes, []uint64{0})
	require.NoError(t, err)
}

func TestDBGetApproximateSizesCF(t *testing.T) {
	t.Parallel()

	db := newTestDB(t, nil)
	defer db.Close()

	o := NewDefaultOptions()

	cf, err := db.CreateColumnFamily(o, "other")
	require.Nil(t, err)

	// no ranges
	sizes, err := db.GetApproximateSizesCF(cf, nil)
	require.EqualValues(t, len(sizes), 0)
	require.NoError(t, err)

	// range will nil start and limit
	sizes, err = db.GetApproximateSizesCF(cf, []Range{{Start: nil, Limit: nil}})
	require.EqualValues(t, sizes, []uint64{0})
	require.NoError(t, err)

	// valid range
	sizes, err = db.GetApproximateSizesCF(cf, []Range{{Start: []byte{0x00}, Limit: []byte{0xFF}}})
	require.EqualValues(t, sizes, []uint64{0})
	require.NoError(t, err)
}

func TestCreateCFs(t *testing.T) {
	t.Parallel()

	db := newTestDB(t, nil)
	defer db.Close()

	o := NewDefaultOptions()

	cfs, err := db.CreateColumnFamilies(o, []string{"other1", "other2"})
	require.Nil(t, err)
	require.Len(t, cfs, 2)

	err = db.PutCF(NewDefaultWriteOptions(), cfs[0], []byte{1, 2, 3}, []byte{4, 5, 6})
	require.NoError(t, err)

	_, err = db.CreateColumnFamilies(o, nil)
	require.NoError(t, err)

	_, err = db.CreateColumnFamilies(o, []string{})
	require.NoError(t, err)
}
