package grocksdb

// #include "rocksdb/c.h"
// #include "grocksdb.h"
import "C"

// Comparing functor.
//
// Three-way comparison. Returns value:
//
//	< 0 iff "a" < "b",
//	== 0 iff "a" == "b",
//	> 0 iff "a" > "b"
//
// Note that Compare(a, b) also compares timestamp if timestamp size is
// non-zero. For the same user key with different timestamps, larger (newer)
// timestamp comes first.
type Comparing = func(a, b []byte) int

// ComparingWithoutTimestamp functor.
//
// Three-way comparison. Returns value:
//
//	< 0 if "a" < "b",
//	== 0 if "a" == "b",
//	> 0 if "a" > "b"
type ComparingWithoutTimestamp = func(a []byte, aHasTs bool, b []byte, bHasTs bool) int

// NewComparator creates a Comparator object which contains native c-comparator pointer.
func NewComparator(name string, compare Comparing) *Comparator {
	cmp := &Comparator{
		name:    name,
		compare: compare,
	}
	idx := registerComperator(cmp)
	cmp.c = C.gorocksdb_comparator_create(C.uintptr_t(idx))
	return cmp
}

// NewComparatorWithTimestamp creates a Timestamp Aware Comparator object which contains native c-comparator pointer.
func NewComparatorWithTimestamp(name string, tsSize uint64, compare, compareTs Comparing, compareWithoutTs ComparingWithoutTimestamp) *Comparator {
	cmp := &Comparator{
		name:             name,
		tsSize:           tsSize,
		compare:          compare,
		compareTs:        compareTs,
		compareWithoutTs: compareWithoutTs,
	}
	idx := registerComperator(cmp)
	cmp.c = C.gorocksdb_comparator_with_ts_create(C.uintptr_t(idx), C.size_t(tsSize))
	return cmp
}

// NativeComparator wraps c-comparator pointer.
type Comparator struct {
	c *C.rocksdb_comparator_t

	name   string
	tsSize uint64

	compare          Comparing
	compareTs        Comparing
	compareWithoutTs ComparingWithoutTimestamp
}

func (c *Comparator) Compare(a, b []byte) int          { return c.compare(a, b) }
func (c *Comparator) CompareTimestamp(a, b []byte) int { return c.compareTs(a, b) }
func (c *Comparator) CompareWithoutTimestamp(a []byte, aHasTs bool, b []byte, bHasTs bool) int {
	return c.compareWithoutTs(a, aHasTs, b, bHasTs)
}
func (c *Comparator) Name() string          { return c.name }
func (c *Comparator) TimestampSize() uint64 { return c.tsSize }
func (c *Comparator) Destroy() {
	C.rocksdb_comparator_destroy(c.c)
	c.c = nil
}

// Hold references to comperators.
var comperators = NewCOWList()

type comperatorWrapper struct {
	name       *C.char
	comparator *Comparator
}

func registerComperator(cmp *Comparator) int {
	return comperators.Append(comperatorWrapper{C.CString(cmp.Name()), cmp})
}

//export gorocksdb_comparator_compare
func gorocksdb_comparator_compare(idx int, cKeyA *C.char, cKeyALen C.size_t, cKeyB *C.char, cKeyBLen C.size_t) C.int {
	keyA := refCBytes(cKeyA, cKeyALen)
	keyB := refCBytes(cKeyB, cKeyBLen)
	return C.int(comperators.Get(idx).(comperatorWrapper).comparator.Compare(keyA, keyB))
}

//export gorocksdb_comparator_compare_ts
func gorocksdb_comparator_compare_ts(idx int, cTsA *C.char, cTsALen C.size_t, cTsB *C.char, cTsBLen C.size_t) C.int {
	tsA := refCBytes(cTsA, cTsALen)
	tsB := refCBytes(cTsB, cTsBLen)
	return C.int(comperators.Get(idx).(comperatorWrapper).comparator.CompareTimestamp(tsA, tsB))
}

//export gorocksdb_comparator_compare_without_ts
func gorocksdb_comparator_compare_without_ts(idx int, cKeyA *C.char, cKeyALen C.size_t, cAHasTs C.uchar, cKeyB *C.char, cKeyBLen C.size_t, cBHasTs C.uchar) C.int {
	keyA := refCBytes(cKeyA, cKeyALen)
	keyB := refCBytes(cKeyB, cKeyBLen)
	keyAHasTs := charToBool(cAHasTs)
	keyBHasTs := charToBool(cBHasTs)
	return C.int(comperators.Get(idx).(comperatorWrapper).comparator.CompareWithoutTimestamp(keyA, keyAHasTs, keyB, keyBHasTs))
}

//export gorocksdb_comparator_name
func gorocksdb_comparator_name(idx int) *C.char {
	return comperators.Get(idx).(comperatorWrapper).name
}
