package gorocksdb

// #include <stdlib.h>
// #include "rocksdb/c.h"
import "C"
import (
	"errors"
	"unsafe"
)

type WalIterator struct {
	c *C.rocksdb_wal_iterator_t
}

func NewNativeWalIterator(c unsafe.Pointer) *WalIterator {
	return &WalIterator{(*C.rocksdb_wal_iterator_t)(c)}
}

func (iter *WalIterator) Valid() bool {
	return C.rocksdb_wal_iter_valid(iter.c) != 0
}

func (iter *WalIterator) Next() {
	C.rocksdb_wal_iter_next(iter.c)
}

func (iter *WalIterator) Err() error {
	var cErr *C.char
	C.rocksdb_wal_iter_status(iter.c, &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return errors.New(C.GoString(cErr))
	}
	return nil
}

func (iter *WalIterator) Destroy() {
	C.rocksdb_wal_iter_destroy(iter.c)
	iter.c = nil
}

// C.rocksdb_wal_iter_get_batch in the official rocksdb c wrapper has memory leak
// see https://github.com/facebook/rocksdb/pull/5515
//     https://github.com/facebook/rocksdb/issues/5536
func (iter *WalIterator) GetBatch() (*WriteBatch, uint64) {
	var cSeq C.uint64_t
	cB := C.rocksdb_wal_iter_get_batch(iter.c, &cSeq)
	return NewNativeWriteBatch(cB), uint64(cSeq)
}
