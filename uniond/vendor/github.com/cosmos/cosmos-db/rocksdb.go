//go:build rocksdb
// +build rocksdb

package db

import (
	"fmt"
	"path/filepath"
	"runtime"

	"github.com/linxGnu/grocksdb"
	"github.com/spf13/cast"
)

func init() {
	dbCreator := func(name string, dir string, opts Options) (DB, error) {
		return NewRocksDB(name, dir, opts)
	}
	registerDBCreator(RocksDBBackend, dbCreator, false)
}

// RocksDB is a RocksDB backend.
type RocksDB struct {
	db     *grocksdb.DB
	ro     *grocksdb.ReadOptions
	wo     *grocksdb.WriteOptions
	woSync *grocksdb.WriteOptions
}

var _ DB = (*RocksDB)(nil)

// defaultRocksdbOptions, good enough for most cases, including heavy workloads.
// 1GB table cache, 512MB write buffer(may use 50% more on heavy workloads).
// compression: snappy as default, need to -lsnappy to enable.
func defaultRocksdbOptions() *grocksdb.Options {
	bbto := grocksdb.NewDefaultBlockBasedTableOptions()
	bbto.SetBlockCache(grocksdb.NewLRUCache(1 << 30))
	bbto.SetFilterPolicy(grocksdb.NewBloomFilter(10))

	rocksdbOpts := grocksdb.NewDefaultOptions()
	rocksdbOpts.SetBlockBasedTableFactory(bbto)
	// SetMaxOpenFiles to 4096 seems to provide a reliable performance boost
	rocksdbOpts.SetMaxOpenFiles(4096)
	rocksdbOpts.SetCreateIfMissing(true)
	rocksdbOpts.IncreaseParallelism(runtime.NumCPU())
	// 1.5GB maximum memory use for writebuffer.
	rocksdbOpts.OptimizeLevelStyleCompaction(512 * 1024 * 1024)
	return rocksdbOpts
}

func NewRocksDB(name string, dir string, opts Options) (*RocksDB, error) {
	defaultOpts := defaultRocksdbOptions()

	if opts != nil {
		files := cast.ToInt(opts.Get("maxopenfiles"))
		if files > 0 {
			defaultOpts.SetMaxOpenFiles(files)
		}
	}

	return NewRocksDBWithOptions(name, dir, defaultOpts)
}

func NewRocksDBWithOptions(name string, dir string, opts *grocksdb.Options) (*RocksDB, error) {
	dbPath := filepath.Join(dir, name+DBFileSuffix)
	db, err := grocksdb.OpenDb(opts, dbPath)
	if err != nil {
		return nil, err
	}
	ro := grocksdb.NewDefaultReadOptions()
	wo := grocksdb.NewDefaultWriteOptions()
	woSync := grocksdb.NewDefaultWriteOptions()
	woSync.SetSync(true)
	return NewRocksDBWithRaw(db, ro, wo, woSync), nil
}

// NewRocksDBWithRaw is useful if user want to create the db in read-only or seconday-standby mode,
// or customize the default read/write options.
func NewRocksDBWithRaw(
	db *grocksdb.DB, ro *grocksdb.ReadOptions,
	wo *grocksdb.WriteOptions, woSync *grocksdb.WriteOptions,
) *RocksDB {
	return &RocksDB{
		db:     db,
		ro:     ro,
		wo:     wo,
		woSync: woSync,
	}
}

// Get implements DB.
func (db *RocksDB) Get(key []byte) ([]byte, error) {
	if len(key) == 0 {
		return nil, errKeyEmpty
	}
	res, err := db.db.Get(db.ro, key)
	if err != nil {
		return nil, err
	}
	return moveSliceToBytes(res), nil
}

// Has implements DB.
func (db *RocksDB) Has(key []byte) (bool, error) {
	bytes, err := db.Get(key)
	if err != nil {
		return false, err
	}
	return bytes != nil, nil
}

// Set implements DB.
func (db *RocksDB) Set(key []byte, value []byte) error {
	if len(key) == 0 {
		return errKeyEmpty
	}
	if value == nil {
		return errValueNil
	}
	return db.db.Put(db.wo, key, value)
}

// SetSync implements DB.
func (db *RocksDB) SetSync(key []byte, value []byte) error {
	if len(key) == 0 {
		return errKeyEmpty
	}
	if value == nil {
		return errValueNil
	}
	return db.db.Put(db.woSync, key, value)
}

// Delete implements DB.
func (db *RocksDB) Delete(key []byte) error {
	if len(key) == 0 {
		return errKeyEmpty
	}
	return db.db.Delete(db.wo, key)
}

// DeleteSync implements DB.
func (db *RocksDB) DeleteSync(key []byte) error {
	if len(key) == 0 {
		return errKeyEmpty
	}
	return db.db.Delete(db.woSync, key)
}

func (db *RocksDB) DB() *grocksdb.DB {
	return db.db
}

// Close implements DB.
func (db *RocksDB) Close() error {
	db.ro.Destroy()
	db.wo.Destroy()
	db.woSync.Destroy()
	db.db.Close()
	return nil
}

// Print implements DB.
func (db *RocksDB) Print() error {
	itr, err := db.Iterator(nil, nil)
	if err != nil {
		return err
	}
	defer itr.Close()
	for ; itr.Valid(); itr.Next() {
		key := itr.Key()
		value := itr.Value()
		fmt.Printf("[%X]:\t[%X]\n", key, value)
	}
	return nil
}

// Stats implements DB.
func (db *RocksDB) Stats() map[string]string {
	keys := []string{"rocksdb.stats"}
	stats := make(map[string]string, len(keys))
	for _, key := range keys {
		stats[key] = db.db.GetProperty(key)
	}
	return stats
}

// NewBatch implements DB.
func (db *RocksDB) NewBatch() Batch {
	return newRocksDBBatch(db)
}

// NewBatchWithSize implements DB.
// It does the same thing as NewBatch because we can't pre-allocate rocksDBBatch
func (db *RocksDB) NewBatchWithSize(_ int) Batch {
	return newRocksDBBatch(db)
}

// Iterator implements DB.
func (db *RocksDB) Iterator(start, end []byte) (Iterator, error) {
	if (start != nil && len(start) == 0) || (end != nil && len(end) == 0) {
		return nil, errKeyEmpty
	}
	itr := db.db.NewIterator(db.ro)
	return newRocksDBIterator(itr, start, end, false), nil
}

// ReverseIterator implements DB.
func (db *RocksDB) ReverseIterator(start, end []byte) (Iterator, error) {
	if (start != nil && len(start) == 0) || (end != nil && len(end) == 0) {
		return nil, errKeyEmpty
	}
	itr := db.db.NewIterator(db.ro)
	return newRocksDBIterator(itr, start, end, true), nil
}
