package gorocksdb

// #include "rocksdb/c.h"
import "C"

// FilterPolicy is a factory type that allows the RocksDB database to create a
// filter, such as a bloom filter, which will used to reduce reads.
type FilterPolicy interface {
	// keys contains a list of keys (potentially with duplicates)
	// that are ordered according to the user supplied comparator.
	CreateFilter(keys [][]byte) []byte

	// "filter" contains the data appended by a preceding call to
	// CreateFilter(). This method must return true if
	// the key was in the list of keys passed to CreateFilter().
	// This method may return true or false if the key was not on the
	// list, but it should aim to return false with a high probability.
	KeyMayMatch(key []byte, filter []byte) bool

	// Return the name of this policy.
	Name() string
}

// NewNativeFilterPolicy creates a FilterPolicy object.
func NewNativeFilterPolicy(c *C.rocksdb_filterpolicy_t) FilterPolicy {
	return nativeFilterPolicy{c}
}

type nativeFilterPolicy struct {
	c *C.rocksdb_filterpolicy_t
}

func (fp nativeFilterPolicy) CreateFilter(keys [][]byte) []byte          { return nil }
func (fp nativeFilterPolicy) KeyMayMatch(key []byte, filter []byte) bool { return false }
func (fp nativeFilterPolicy) Name() string                               { return "" }

// NewBloomFilter returns a new filter policy that uses a bloom filter with approximately
// the specified number of bits per key.  A good value for bits_per_key
// is 10, which yields a filter with ~1% false positive rate.
//
// Note: if you are using a custom comparator that ignores some parts
// of the keys being compared, you must not use NewBloomFilterPolicy()
// and must provide your own FilterPolicy that also ignores the
// corresponding parts of the keys.  For example, if the comparator
// ignores trailing spaces, it would be incorrect to use a
// FilterPolicy (like NewBloomFilterPolicy) that does not ignore
// trailing spaces in keys.
func NewBloomFilter(bitsPerKey int) FilterPolicy {
	return NewNativeFilterPolicy(C.rocksdb_filterpolicy_create_bloom(C.int(bitsPerKey)))
}

// NewBloomFilterFull returns a new filter policy created with use_block_based_builder=false
// (use full or partitioned filter).
func NewBloomFilterFull(bitsPerKey int) FilterPolicy {
	return NewNativeFilterPolicy(C.rocksdb_filterpolicy_create_bloom_full(C.int(bitsPerKey)))
}

// Hold references to filter policies.
var filterPolicies = NewCOWList()

type filterPolicyWrapper struct {
	name         *C.char
	filterPolicy FilterPolicy
}

func registerFilterPolicy(fp FilterPolicy) int {
	return filterPolicies.Append(filterPolicyWrapper{C.CString(fp.Name()), fp})
}

//export gorocksdb_filterpolicy_create_filter
func gorocksdb_filterpolicy_create_filter(idx int, cKeys **C.char, cKeysLen *C.size_t, cNumKeys C.int, cDstLen *C.size_t) *C.char {
	rawKeys := charSlice(cKeys, cNumKeys)
	keysLen := sizeSlice(cKeysLen, cNumKeys)
	keys := make([][]byte, int(cNumKeys))
	for i, len := range keysLen {
		keys[i] = charToByte(rawKeys[i], len)
	}

	dst := filterPolicies.Get(idx).(filterPolicyWrapper).filterPolicy.CreateFilter(keys)
	*cDstLen = C.size_t(len(dst))
	return cByteSlice(dst)
}

//export gorocksdb_filterpolicy_key_may_match
func gorocksdb_filterpolicy_key_may_match(idx int, cKey *C.char, cKeyLen C.size_t, cFilter *C.char, cFilterLen C.size_t) C.uchar {
	key := charToByte(cKey, cKeyLen)
	filter := charToByte(cFilter, cFilterLen)
	return boolToChar(filterPolicies.Get(idx).(filterPolicyWrapper).filterPolicy.KeyMayMatch(key, filter))
}

//export gorocksdb_filterpolicy_name
func gorocksdb_filterpolicy_name(idx int) *C.char {
	return filterPolicies.Get(idx).(filterPolicyWrapper).name
}
