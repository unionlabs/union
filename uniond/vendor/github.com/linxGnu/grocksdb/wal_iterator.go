package grocksdb

// #include <stdlib.h>
// #include "rocksdb/c.h"
import "C"
import "unsafe"

// WalIterator is iterator for WAL Files.
type WalIterator struct {
	c *C.rocksdb_wal_iterator_t
}

// NewNativeWalIterator returns new WalIterator.
func newNativeWalIterator(c unsafe.Pointer) *WalIterator {
	return &WalIterator{c: (*C.rocksdb_wal_iterator_t)(c)}
}

// Valid check if current WAL is valid.
func (iter *WalIterator) Valid() bool {
	return C.rocksdb_wal_iter_valid(iter.c) != 0
}

// Next moves next.
func (iter *WalIterator) Next() {
	C.rocksdb_wal_iter_next(iter.c)
}

// Err returns error happened during iteration.
func (iter *WalIterator) Err() (err error) {
	var cErr *C.char
	C.rocksdb_wal_iter_status(iter.c, &cErr)
	err = fromCError(cErr)
	return
}

// Destroy free iterator.
func (iter *WalIterator) Destroy() {
	C.rocksdb_wal_iter_destroy(iter.c)
	iter.c = nil
}

// GetBatch returns the current write_batch and the sequence number of the
// earliest transaction contained in the batch.
func (iter *WalIterator) GetBatch() (*WriteBatch, uint64) {
	var cSeq C.uint64_t
	cB := C.rocksdb_wal_iter_get_batch(iter.c, &cSeq)
	return newNativeWriteBatch(cB), uint64(cSeq)
}
