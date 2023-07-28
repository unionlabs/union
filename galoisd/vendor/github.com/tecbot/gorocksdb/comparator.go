package gorocksdb

// #include "rocksdb/c.h"
import "C"

// A Comparator object provides a total order across slices that are
// used as keys in an sstable or a database.
type Comparator interface {
	// Three-way comparison. Returns value:
	//   < 0 iff "a" < "b",
	//   == 0 iff "a" == "b",
	//   > 0 iff "a" > "b"
	Compare(a, b []byte) int

	// The name of the comparator.
	Name() string
}

// NewNativeComparator creates a Comparator object.
func NewNativeComparator(c *C.rocksdb_comparator_t) Comparator {
	return nativeComparator{c}
}

type nativeComparator struct {
	c *C.rocksdb_comparator_t
}

func (c nativeComparator) Compare(a, b []byte) int { return 0 }
func (c nativeComparator) Name() string            { return "" }

// Hold references to comperators.
var comperators = NewCOWList()

type comperatorWrapper struct {
	name       *C.char
	comparator Comparator
}

func registerComperator(cmp Comparator) int {
	return comperators.Append(comperatorWrapper{C.CString(cmp.Name()), cmp})
}

//export gorocksdb_comparator_compare
func gorocksdb_comparator_compare(idx int, cKeyA *C.char, cKeyALen C.size_t, cKeyB *C.char, cKeyBLen C.size_t) C.int {
	keyA := charToByte(cKeyA, cKeyALen)
	keyB := charToByte(cKeyB, cKeyBLen)
	return C.int(comperators.Get(idx).(comperatorWrapper).comparator.Compare(keyA, keyB))
}

//export gorocksdb_comparator_name
func gorocksdb_comparator_name(idx int) *C.char {
	return comperators.Get(idx).(comperatorWrapper).name
}
