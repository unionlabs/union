package levigo

// #cgo LDFLAGS: -lleveldb
// #include "leveldb/c.h"
import "C"

import (
	"unsafe"
)

// WriteBatch is a batching of Puts, and Deletes to be written atomically to a
// database. A WriteBatch is written when passed to DB.Write.
//
// To prevent memory leaks, call Close when the program no longer needs the
// WriteBatch object.
type WriteBatch struct {
	wbatch *C.leveldb_writebatch_t
}

// NewWriteBatch creates a fully allocated WriteBatch.
func NewWriteBatch() *WriteBatch {
	wb := C.leveldb_writebatch_create()
	return &WriteBatch{wb}
}

// Close releases the underlying memory of a WriteBatch.
func (w *WriteBatch) Close() {
	C.leveldb_writebatch_destroy(w.wbatch)
}

// Put places a key-value pair into the WriteBatch for writing later.
//
// Both the key and value byte slices may be reused as WriteBatch takes a copy
// of them before returning.
//
func (w *WriteBatch) Put(key, value []byte) {
	// leveldb_writebatch_put, and _delete call memcpy() (by way of
	// Memtable::Add) when called, so we do not need to worry about these
	// []byte being reclaimed by GC.
	var k, v *C.char
	if len(key) != 0 {
		k = (*C.char)(unsafe.Pointer(&key[0]))
	}
	if len(value) != 0 {
		v = (*C.char)(unsafe.Pointer(&value[0]))
	}

	lenk := len(key)
	lenv := len(value)

	C.leveldb_writebatch_put(w.wbatch, k, C.size_t(lenk), v, C.size_t(lenv))
}

// Delete queues a deletion of the data at key to be deleted later.
//
// The key byte slice may be reused safely. Delete takes a copy of
// them before returning.
func (w *WriteBatch) Delete(key []byte) {
	C.leveldb_writebatch_delete(w.wbatch,
		(*C.char)(unsafe.Pointer(&key[0])), C.size_t(len(key)))
}

// Clear removes all the enqueued Put and Deletes in the WriteBatch.
func (w *WriteBatch) Clear() {
	C.leveldb_writebatch_clear(w.wbatch)
}
