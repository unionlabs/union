package grocksdb

// #include "rocksdb/c.h"
import "C"
import "unsafe"

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

	// SetIgnoreSnapshots before release 6.0, if there is a snapshot taken later than
	// the key/value pair, RocksDB always try to prevent the key/value pair from being
	// filtered by compaction filter so that users can preserve the same view from a
	// snapshot, unless the compaction filter returns IgnoreSnapshots() = true. However,
	// this feature is deleted since 6.0, after realized that the feature has a bug which
	// can't be easily fixed. Since release 6.0, with compaction filter enabled, RocksDB
	// always invoke filtering for any key, even if it knows it will make a snapshot
	// not repeatable.
	SetIgnoreSnapshots(value bool)

	// Destroy underlying pointer/data.
	Destroy()
}

// NewNativeCompactionFilter creates a CompactionFilter object.
func NewNativeCompactionFilter(c unsafe.Pointer) CompactionFilter {
	return &nativeCompactionFilter{c: (*C.rocksdb_compactionfilter_t)(c)}
}

type nativeCompactionFilter struct {
	c *C.rocksdb_compactionfilter_t
}

func (c *nativeCompactionFilter) Filter(level int, key, val []byte) (remove bool, newVal []byte) {
	return false, nil
}
func (c *nativeCompactionFilter) Name() string { return "" }

func (c *nativeCompactionFilter) SetIgnoreSnapshots(value bool) {
	C.rocksdb_compactionfilter_set_ignore_snapshots(c.c, boolToChar(value))
}

func (c *nativeCompactionFilter) Destroy() {
	C.rocksdb_compactionfilter_destroy(c.c)
	c.c = nil
}

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
	key := refCBytes(cKey, cKeyLen)
	val := refCBytes(cVal, cValLen)

	remove, newVal := compactionFilters.Get(idx).(compactionFilterWrapper).filter.Filter(int(cLevel), key, val)
	if remove {
		return C.int(1)
	} else if newVal != nil {
		*cNewVal = refGoBytes(newVal)
		*cNewValLen = C.size_t(len(newVal))
		*cValChanged = C.uchar(1)
	}
	return C.int(0)
}

//export gorocksdb_compactionfilter_name
func gorocksdb_compactionfilter_name(idx int) *C.char {
	return compactionFilters.Get(idx).(compactionFilterWrapper).name
}
