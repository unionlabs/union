package levigo

// #cgo LDFLAGS: -lleveldb
// #include <stdlib.h>
// #include "leveldb/c.h"
import "C"

import (
	"unsafe"
)

type IteratorError string

func (e IteratorError) Error() string {
	return string(e)
}

// Iterator is a read-only iterator through a LevelDB database. It provides a
// way to seek to specific keys and iterate through the keyspace from that
// point, as well as access the values of those keys.
//
// Care must be taken when using an Iterator. If the method Valid returns
// false, calls to Key, Value, Next, and Prev will result in panics. However,
// Seek, SeekToFirst, SeekToLast, GetError, Valid, and Close will still be
// safe to call.
//
// GetError will only return an error in the event of a LevelDB error. It will
// return a nil on iterators that are simply invalid. Given that behavior,
// GetError is not a replacement for a Valid.
//
// A typical use looks like:
//
// 	db := levigo.Open(...)
//
// 	it := db.NewIterator(readOpts)
// 	defer it.Close()
// 	for it.Seek(mykey); it.Valid(); it.Next() {
// 		useKeyAndValue(it.Key(), it.Value())
// 	}
// 	if err := it.GetError() {
// 		...
// 	}
//
// To prevent memory leaks, an Iterator must have Close called on it when it
// is no longer needed by the program.
type Iterator struct {
	Iter *C.leveldb_iterator_t
}

// Valid returns false only when an Iterator has iterated past either the
// first or the last key in the database.
func (it *Iterator) Valid() bool {
	return ucharToBool(C.leveldb_iter_valid(it.Iter))
}

// Key returns a copy the key in the database the iterator currently holds.
//
// If Valid returns false, this method will panic.
func (it *Iterator) Key() []byte {
	var klen C.size_t
	kdata := C.leveldb_iter_key(it.Iter, &klen)
	if kdata == nil {
		return nil
	}
	// Unlike DB.Get, the key, kdata, returned is not meant to be freed by the
	// client. It's a direct reference to data managed by the iterator_t
	// instead of a copy.  So, we must not free it here but simply copy it
	// with GoBytes.
	return C.GoBytes(unsafe.Pointer(kdata), C.int(klen))
}

// Value returns a copy of the value in the database the iterator currently
// holds.
//
// If Valid returns false, this method will panic.
func (it *Iterator) Value() []byte {
	var vlen C.size_t
	vdata := C.leveldb_iter_value(it.Iter, &vlen)
	if vdata == nil {
		return nil
	}
	// Unlike DB.Get, the value, vdata, returned is not meant to be freed by
	// the client. It's a direct reference to data managed by the iterator_t
	// instead of a copy. So, we must not free it here but simply copy it with
	// GoBytes.
	return C.GoBytes(unsafe.Pointer(vdata), C.int(vlen))
}

// Next moves the iterator to the next sequential key in the database, as
// defined by the Comparator in the ReadOptions used to create this Iterator.
//
// If Valid returns false, this method will panic.
func (it *Iterator) Next() {
	C.leveldb_iter_next(it.Iter)
}

// Prev moves the iterator to the previous sequential key in the database, as
// defined by the Comparator in the ReadOptions used to create this Iterator.
//
// If Valid returns false, this method will panic.
func (it *Iterator) Prev() {
	C.leveldb_iter_prev(it.Iter)
}

// SeekToFirst moves the iterator to the first key in the database, as defined
// by the Comparator in the ReadOptions used to create this Iterator.
//
// This method is safe to call when Valid returns false.
func (it *Iterator) SeekToFirst() {
	C.leveldb_iter_seek_to_first(it.Iter)
}

// SeekToLast moves the iterator to the last key in the database, as defined
// by the Comparator in the ReadOptions used to create this Iterator.
//
// This method is safe to call when Valid returns false.
func (it *Iterator) SeekToLast() {
	C.leveldb_iter_seek_to_last(it.Iter)
}

// Seek moves the iterator the position of the key given or, if the key
// doesn't exist, the next key that does exist in the database. If the key
// doesn't exist, and there is no next key, the Iterator becomes invalid.
//
// This method is safe to call when Valid returns false.
func (it *Iterator) Seek(key []byte) {
	C.leveldb_iter_seek(it.Iter, (*C.char)(unsafe.Pointer(&key[0])), C.size_t(len(key)))
}

// GetError returns an IteratorError from LevelDB if it had one during
// iteration.
//
// This method is safe to call when Valid returns false.
func (it *Iterator) GetError() error {
	var errStr *C.char
	C.leveldb_iter_get_error(it.Iter, &errStr)
	if errStr != nil {
		gs := C.GoString(errStr)
		C.leveldb_free(unsafe.Pointer(errStr))
		return IteratorError(gs)
	}
	return nil
}

// Close deallocates the given Iterator, freeing the underlying C struct.
func (it *Iterator) Close() {
	C.leveldb_iter_destroy(it.Iter)
	it.Iter = nil
}
