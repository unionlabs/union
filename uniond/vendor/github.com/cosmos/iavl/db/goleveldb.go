package db

import (
	"bytes"
	"errors"
	"fmt"
	"path/filepath"

	corestore "cosmossdk.io/core/store"
	"github.com/syndtr/goleveldb/leveldb"
	dberrors "github.com/syndtr/goleveldb/leveldb/errors"
	"github.com/syndtr/goleveldb/leveldb/filter"
	"github.com/syndtr/goleveldb/leveldb/iterator"
	"github.com/syndtr/goleveldb/leveldb/opt"
	"github.com/syndtr/goleveldb/leveldb/util"
)

const DBFileSuffix string = ".db"

// GoLevelDB implements corestore.KVStore using github.com/syndtr/goleveldb/leveldb.
// It is used for only store v2 migration, since some clients use goleveldb as
// the IAVL v0/v1 backend.
type GoLevelDB struct {
	db *leveldb.DB
}

func NewGoLevelDB(name, dir string) (*GoLevelDB, error) {
	defaultOpts := &opt.Options{
		Filter: filter.NewBloomFilter(10), // by default, goleveldb doesn't use a bloom filter.
	}

	return NewGoLevelDBWithOpts(name, dir, defaultOpts)
}

func NewGoLevelDBWithOpts(name, dir string, o *opt.Options) (*GoLevelDB, error) {
	dbPath := filepath.Join(dir, name+DBFileSuffix)
	db, err := leveldb.OpenFile(dbPath, o)
	if err != nil {
		return nil, err
	}
	return &GoLevelDB{db: db}, nil
}

// Get implements corestore.KVStore.
func (db *GoLevelDB) Get(key []byte) ([]byte, error) {
	if len(key) == 0 {
		return nil, errors.New("key is empty")
	}
	res, err := db.db.Get(key, nil)
	if err != nil {
		if errors.Is(err, dberrors.ErrNotFound) {
			return nil, nil
		}
		return nil, err
	}
	return res, nil
}

// Has implements corestore.KVStore.
func (db *GoLevelDB) Has(key []byte) (bool, error) {
	return db.db.Has(key, nil)
}

// Set implements corestore.KVStore.
func (db *GoLevelDB) Set(key, value []byte) error {
	if len(key) == 0 {
		return errors.New("key is empty")
	}
	if value == nil {
		return errors.New("value is nil")
	}
	return db.db.Put(key, value, nil)
}

// SetSync implements corestore.KVStore.
func (db *GoLevelDB) SetSync(key, value []byte) error {
	if len(key) == 0 {
		return errors.New("key is empty")
	}
	if value == nil {
		return errors.New("value is nil")
	}
	return db.db.Put(key, value, &opt.WriteOptions{Sync: true})
}

// Delete implements corestore.KVStore.
func (db *GoLevelDB) Delete(key []byte) error {
	if len(key) == 0 {
		return errors.New("key is empty")
	}
	return db.db.Delete(key, nil)
}

// DeleteSync implements corestore.KVStore.
func (db *GoLevelDB) DeleteSync(key []byte) error {
	if len(key) == 0 {
		return errors.New("key is empty")
	}
	return db.db.Delete(key, &opt.WriteOptions{Sync: true})
}

func (db *GoLevelDB) RawDB() *leveldb.DB {
	return db.db
}

// Close implements corestore.KVStore.
func (db *GoLevelDB) Close() error {
	return db.db.Close()
}

// Print implements corestore.KVStore.
func (db *GoLevelDB) Print() error {
	str, err := db.db.GetProperty("leveldb.stats")
	if err != nil {
		return err
	}
	fmt.Printf("%v\n", str)

	itr := db.db.NewIterator(nil, nil)
	for itr.Next() {
		key := itr.Key()
		value := itr.Value()
		fmt.Printf("[%X]:\t[%X]\n", key, value)
	}
	return nil
}

// Stats implements corestore.KVStore.
func (db *GoLevelDB) Stats() map[string]string {
	keys := []string{
		"leveldb.num-files-at-level{n}",
		"leveldb.stats",
		"leveldb.sstables",
		"leveldb.blockpool",
		"leveldb.cachedblock",
		"leveldb.openedtables",
		"leveldb.alivesnaps",
		"leveldb.aliveiters",
	}

	stats := make(map[string]string)
	for _, key := range keys {
		if str, err := db.db.GetProperty(key); err == nil {
			stats[key] = str
		}
	}
	return stats
}

func (db *GoLevelDB) ForceCompact(start, limit []byte) error {
	return db.db.CompactRange(util.Range{Start: start, Limit: limit})
}

// NewBatch implements corestore.BatchCreator.
func (db *GoLevelDB) NewBatch() corestore.Batch {
	return newGoLevelDBBatch(db)
}

// NewBatchWithSize implements corestore.BatchCreator.
func (db *GoLevelDB) NewBatchWithSize(size int) corestore.Batch {
	return newGoLevelDBBatchWithSize(db, size)
}

// Iterator implements KVStore.
func (db *GoLevelDB) Iterator(start, end []byte) (corestore.Iterator, error) {
	if (start != nil && len(start) == 0) || (end != nil && len(end) == 0) {
		return nil, errors.New("key is empty")
	}
	itr := db.db.NewIterator(&util.Range{Start: start, Limit: end}, nil)
	return newGoLevelDBIterator(itr, start, end, false), nil
}

// ReverseIterator implements corestore.KVStore.
func (db *GoLevelDB) ReverseIterator(start, end []byte) (corestore.Iterator, error) {
	if (start != nil && len(start) == 0) || (end != nil && len(end) == 0) {
		return nil, errors.New("key is empty")
	}
	itr := db.db.NewIterator(&util.Range{Start: start, Limit: end}, nil)
	return newGoLevelDBIterator(itr, start, end, true), nil
}

type goLevelDBIterator struct {
	source    iterator.Iterator
	start     []byte
	end       []byte
	isReverse bool
	isInvalid bool
}

var _ corestore.Iterator = (*goLevelDBIterator)(nil)

func newGoLevelDBIterator(source iterator.Iterator, start, end []byte, isReverse bool) *goLevelDBIterator {
	if isReverse {
		if end == nil {
			source.Last()
		} else {
			if source.Seek(end) {
				eoakey := source.Key() // end or after key
				if bytes.Compare(end, eoakey) <= 0 {
					source.Prev()
				}
			} else {
				source.Last()
			}
		}
	} else {
		if start == nil {
			source.First()
		} else {
			source.Seek(start)
		}
	}
	return &goLevelDBIterator{
		source:    source,
		start:     start,
		end:       end,
		isReverse: isReverse,
		isInvalid: false,
	}
}

// Domain implements Iterator.
func (itr *goLevelDBIterator) Domain() ([]byte, []byte) {
	return itr.start, itr.end
}

// Valid implements Iterator.
func (itr *goLevelDBIterator) Valid() bool {
	// Once invalid, forever invalid.
	if itr.isInvalid {
		return false
	}

	// If source errors, invalid.
	if err := itr.Error(); err != nil {
		itr.isInvalid = true
		return false
	}

	// If source is invalid, invalid.
	if !itr.source.Valid() {
		itr.isInvalid = true
		return false
	}

	// If key is end or past it, invalid.
	start := itr.start
	end := itr.end
	key := itr.source.Key()

	if itr.isReverse {
		if start != nil && bytes.Compare(key, start) < 0 {
			itr.isInvalid = true
			return false
		}
	} else {
		if end != nil && bytes.Compare(end, key) <= 0 {
			itr.isInvalid = true
			return false
		}
	}

	// Valid
	return true
}

// Key implements Iterator.
func (itr *goLevelDBIterator) Key() []byte {
	// Key returns a copy of the current key.
	// See https://github.com/syndtr/goleveldb/blob/52c212e6c196a1404ea59592d3f1c227c9f034b2/leveldb/iterator/iter.go#L88
	itr.assertIsValid()
	return cp(itr.source.Key())
}

// Value implements Iterator.
func (itr *goLevelDBIterator) Value() []byte {
	// Value returns a copy of the current value.
	// See https://github.com/syndtr/goleveldb/blob/52c212e6c196a1404ea59592d3f1c227c9f034b2/leveldb/iterator/iter.go#L88
	itr.assertIsValid()
	return cp(itr.source.Value())
}

// Next implements Iterator.
func (itr *goLevelDBIterator) Next() {
	itr.assertIsValid()
	if itr.isReverse {
		itr.source.Prev()
	} else {
		itr.source.Next()
	}
}

// Error implements Iterator.
func (itr *goLevelDBIterator) Error() error {
	return itr.source.Error()
}

// Close implements Iterator.
func (itr *goLevelDBIterator) Close() error {
	itr.source.Release()
	return nil
}

func (itr goLevelDBIterator) assertIsValid() {
	if !itr.Valid() {
		panic("iterator is invalid")
	}
}

type goLevelDBBatch struct {
	db    *GoLevelDB
	batch *leveldb.Batch
}

var _ corestore.Batch = (*goLevelDBBatch)(nil)

func newGoLevelDBBatch(db *GoLevelDB) *goLevelDBBatch {
	return &goLevelDBBatch{
		db:    db,
		batch: new(leveldb.Batch),
	}
}

func newGoLevelDBBatchWithSize(db *GoLevelDB, size int) *goLevelDBBatch {
	return &goLevelDBBatch{
		db:    db,
		batch: leveldb.MakeBatch(size),
	}
}

// Set implements corestore.Batch.
func (b *goLevelDBBatch) Set(key, value []byte) error {
	if len(key) == 0 {
		return errors.New("key is empty")
	}
	if value == nil {
		return errors.New("value is nil")
	}
	if b.batch == nil {
		return errors.New("batch is nil")
	}
	b.batch.Put(key, value)
	return nil
}

// Delete implements corestore.Batch.
func (b *goLevelDBBatch) Delete(key []byte) error {
	if len(key) == 0 {
		return errors.New("key is empty")
	}
	if b.batch == nil {
		return errors.New("batch is nil")
	}
	b.batch.Delete(key)
	return nil
}

// Write implements corestore.Batch.
func (b *goLevelDBBatch) Write() error {
	return b.write(false)
}

// WriteSync implements corestore.Batch.
func (b *goLevelDBBatch) WriteSync() error {
	return b.write(true)
}

func (b *goLevelDBBatch) write(sync bool) error {
	if b.batch == nil {
		return errors.New("batch is nil")
	}
	if err := b.db.db.Write(b.batch, &opt.WriteOptions{Sync: sync}); err != nil {
		return err
	}
	// Make sure batch cannot be used afterwards. Callers should still call Close(), for errors.
	return b.Close()
}

// Close implements corestore.Batch.
func (b *goLevelDBBatch) Close() error {
	if b.batch != nil {
		b.batch.Reset()
		b.batch = nil
	}
	return nil
}

// GetByteSize implements corestore.Batch
func (b *goLevelDBBatch) GetByteSize() (int, error) {
	if b.batch == nil {
		return 0, errors.New("batch is nil")
	}
	return len(b.batch.Dump()), nil
}

func cp(bz []byte) (ret []byte) {
	ret = make([]byte, len(bz))
	copy(ret, bz)
	return ret
}
