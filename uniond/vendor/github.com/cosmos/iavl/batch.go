package iavl

import (
	"sync"

	dbm "github.com/cosmos/iavl/db"
)

// BatchWithFlusher is a wrapper
// around batch that flushes batch's data to disk
// as soon as the configurable limit is reached.
type BatchWithFlusher struct {
	mtx   sync.Mutex
	db    dbm.DB    // This is only used to create new batch
	batch dbm.Batch // Batched writing buffer.

	flushThreshold int // The threshold to flush the batch to disk.
}

var _ dbm.Batch = (*BatchWithFlusher)(nil)

// NewBatchWithFlusher returns new BatchWithFlusher wrapping the passed in batch
func NewBatchWithFlusher(db dbm.DB, flushThreshold int) *BatchWithFlusher {
	return &BatchWithFlusher{
		db:             db,
		batch:          db.NewBatchWithSize(flushThreshold),
		flushThreshold: flushThreshold,
	}
}

// estimateSizeAfterSetting estimates the batch's size after setting a key / value
func (b *BatchWithFlusher) estimateSizeAfterSetting(key []byte, value []byte) (int, error) {
	currentSize, err := b.batch.GetByteSize()
	if err != nil {
		return 0, err
	}
	// for some batch implementation, when adding a key / value,
	// the batch size could gain more than the total size of key and value,
	// https://github.com/syndtr/goleveldb/blob/64ee5596c38af10edb6d93e1327b3ed1739747c7/leveldb/batch.go#L98

	// we add 100 here just to over-account for that overhead
	// since estimateSizeAfterSetting is only used to check if we exceed the threshold when setting a key / value
	// this means we only over-account for the last key / value
	return currentSize + len(key) + len(value) + 100, nil
}

// Set sets value at the given key to the db.
// If the set causes the underlying batch size to exceed flushThreshold,
// the batch is flushed to disk, cleared, and a new one is created with buffer pre-allocated to threshold.
// The addition entry is then added to the batch.
func (b *BatchWithFlusher) Set(key, value []byte) error {
	b.mtx.Lock()
	defer b.mtx.Unlock()

	batchSizeAfter, err := b.estimateSizeAfterSetting(key, value)
	if err != nil {
		return err
	}
	if batchSizeAfter > b.flushThreshold {
		b.mtx.Unlock()
		if err := b.Write(); err != nil {
			return err
		}
		b.mtx.Lock()
	}
	return b.batch.Set(key, value)
}

// Delete delete value at the given key to the db.
// If the deletion causes the underlying batch size to exceed batchSizeFlushThreshold,
// the batch is flushed to disk, cleared, and a new one is created with buffer pre-allocated to threshold.
// The deletion entry is then added to the batch.
func (b *BatchWithFlusher) Delete(key []byte) error {
	b.mtx.Lock()
	defer b.mtx.Unlock()

	batchSizeAfter, err := b.estimateSizeAfterSetting(key, []byte{})
	if err != nil {
		return err
	}
	if batchSizeAfter > b.flushThreshold {
		b.mtx.Unlock()
		if err := b.Write(); err != nil {
			return err
		}
		b.mtx.Lock()
	}
	return b.batch.Delete(key)
}

func (b *BatchWithFlusher) Write() error {
	b.mtx.Lock()
	defer b.mtx.Unlock()

	if err := b.batch.Write(); err != nil {
		return err
	}
	if err := b.batch.Close(); err != nil {
		return err
	}
	b.batch = b.db.NewBatchWithSize(b.flushThreshold)
	return nil
}

func (b *BatchWithFlusher) WriteSync() error {
	b.mtx.Lock()
	defer b.mtx.Unlock()

	if err := b.batch.WriteSync(); err != nil {
		return err
	}
	if err := b.batch.Close(); err != nil {
		return err
	}
	b.batch = b.db.NewBatchWithSize(b.flushThreshold)
	return nil
}

func (b *BatchWithFlusher) Close() error {
	b.mtx.Lock()
	defer b.mtx.Unlock()

	return b.batch.Close()
}

func (b *BatchWithFlusher) GetByteSize() (int, error) {
	return b.batch.GetByteSize()
}
