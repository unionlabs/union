//go:build rocksdb
// +build rocksdb

package db

import "github.com/linxGnu/grocksdb"

type rocksDBBatch struct {
	db    *RocksDB
	batch *grocksdb.WriteBatch
}

var _ Batch = (*rocksDBBatch)(nil)

func newRocksDBBatch(db *RocksDB) *rocksDBBatch {
	return &rocksDBBatch{
		db:    db,
		batch: grocksdb.NewWriteBatch(),
	}
}

// Set implements Batch.
func (b *rocksDBBatch) Set(key, value []byte) error {
	if len(key) == 0 {
		return errKeyEmpty
	}
	if value == nil {
		return errValueNil
	}
	if b.batch == nil {
		return errBatchClosed
	}
	b.batch.Put(key, value)
	return nil
}

// Delete implements Batch.
func (b *rocksDBBatch) Delete(key []byte) error {
	if len(key) == 0 {
		return errKeyEmpty
	}
	if b.batch == nil {
		return errBatchClosed
	}
	b.batch.Delete(key)
	return nil
}

// Write implements Batch.
func (b *rocksDBBatch) Write() error {
	if b.batch == nil {
		return errBatchClosed
	}
	err := b.db.db.Write(b.db.wo, b.batch)
	if err != nil {
		return err
	}
	// Make sure batch cannot be used afterwards. Callers should still call Close(), for errors.
	b.Close()
	return nil
}

// WriteSync implements Batch.
func (b *rocksDBBatch) WriteSync() error {
	if b.batch == nil {
		return errBatchClosed
	}
	err := b.db.db.Write(b.db.woSync, b.batch)
	if err != nil {
		return err
	}
	// Make sure batch cannot be used afterwards. Callers should still call Close(), for errors.
	return b.Close()
}

// Close implements Batch.
func (b *rocksDBBatch) Close() error {
	if b.batch != nil {
		b.batch.Destroy()
		b.batch = nil
	}
	return nil
}
