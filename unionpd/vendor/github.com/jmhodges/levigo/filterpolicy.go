package levigo

// #cgo LDFLAGS: -lleveldb
// #include <stdlib.h>
// #include "leveldb/c.h"
import "C"

// FilterPolicy is a factory type that allows the LevelDB database to create a
// filter, such as a bloom filter, that is stored in the sstables and used by
// DB.Get to reduce reads.
//
// An instance of this struct may be supplied to Options when opening a
// DB. Typical usage is to call NewBloomFilter to get an instance.
//
// To prevent memory leaks, a FilterPolicy must have Close called on it when
// it is no longer needed by the program.
type FilterPolicy struct {
	Policy *C.leveldb_filterpolicy_t
}

// NewBloomFilter creates a filter policy that will create a bloom filter when
// necessary with the given number of bits per key.
//
// See the FilterPolicy documentation for more.
func NewBloomFilter(bitsPerKey int) *FilterPolicy {
	policy := C.leveldb_filterpolicy_create_bloom(C.int(bitsPerKey))
	return &FilterPolicy{policy}
}

func (fp *FilterPolicy) Close() {
	C.leveldb_filterpolicy_destroy(fp.Policy)
}
