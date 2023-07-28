package gorocksdb

// #include "rocksdb/c.h"
import "C"

// A CompactionFilter can be used to filter keys during compaction time.
type CompactionFilter interface {
	// If the Filter function returns false, it indicates
	// that the kv should be preserved, while a return value of true
	// indicates that this key-value should be removed from the
	// output of the compaction. The application can inspect
	// the existing value of the key and make decision based on it.
	//
	// When the value is to be preserved, the application has the option
	// to modify the existing value and pass it back through a new value.
	// To retain the previous value, simply return nil
	//
	// If multithreaded compaction is being used *and* a single CompactionFilter
	// instance was supplied via SetCompactionFilter, this the Filter function may be
	// called from different threads concurrently. The application must ensure
	// that the call is thread-safe.
	Filter(level int, key, val []byte) (remove bool, newVal []byte)

	// The name of the compaction filter, for logging
	Name() string
}

// NewNativeCompactionFilter creates a CompactionFilter object.
func NewNativeCompactionFilter(c *C.rocksdb_compactionfilter_t) CompactionFilter {
	return nativeCompactionFilter{c}
}

type nativeCompactionFilter struct {
	c *C.rocksdb_compactionfilter_t
}

func (c nativeCompactionFilter) Filter(level int, key, val []byte) (remove bool, newVal []byte) {
	return false, nil
}
func (c nativeCompactionFilter) Name() string { return "" }

// Hold references to compaction filters.
var compactionFilters = NewCOWList()

type compactionFilterWrapper struct {
	name   *C.char
	filter CompactionFilter
}

func registerCompactionFilter(filter CompactionFilter) int {
	return compactionFilters.Append(compactionFilterWrapper{C.CString(filter.Name()), filter})
}

//export gorocksdb_compactionfilter_filter
func gorocksdb_compactionfilter_filter(idx int, cLevel C.int, cKey *C.char, cKeyLen C.size_t, cVal *C.char, cValLen C.size_t, cNewVal **C.char, cNewValLen *C.size_t, cValChanged *C.uchar) C.int {
	key := charToByte(cKey, cKeyLen)
	val := charToByte(cVal, cValLen)

	remove, newVal := compactionFilters.Get(idx).(compactionFilterWrapper).filter.Filter(int(cLevel), key, val)
	if remove {
		return C.int(1)
	} else if newVal != nil {
		*cNewVal = byteToChar(newVal)
		*cNewValLen = C.size_t(len(newVal))
		*cValChanged = C.uchar(1)
	}
	return C.int(0)
}

//export gorocksdb_compactionfilter_name
func gorocksdb_compactionfilter_name(idx int) *C.char {
	return compactionFilters.Get(idx).(compactionFilterWrapper).name
}
