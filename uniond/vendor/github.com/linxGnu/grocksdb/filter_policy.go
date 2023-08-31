package grocksdb

// #include "rocksdb/c.h"
import "C"

// NativeFilterPolicy wraps over rocksdb filter policy.
type NativeFilterPolicy struct {
	c *C.rocksdb_filterpolicy_t
}

func (fp *NativeFilterPolicy) Destroy() {
	C.rocksdb_filterpolicy_destroy(fp.c)
	fp.c = nil
}

// creates a FilterPolicy object.
func newNativeFilterPolicy(c *C.rocksdb_filterpolicy_t) *NativeFilterPolicy {
	return &NativeFilterPolicy{c: c}
}

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
func NewBloomFilter(bitsPerKey float64) *NativeFilterPolicy {
	cFilter := C.rocksdb_filterpolicy_create_bloom(C.double(bitsPerKey))
	return newNativeFilterPolicy(cFilter)
}

// NewBloomFilterFull returns a new filter policy that uses a full bloom filter
// with approximately the specified number of bits per key. A good value for
// bits_per_key is 10, which yields a filter with ~1% false positive rate.
//
// Note: if you are using a custom comparator that ignores some parts
// of the keys being compared, you must not use NewBloomFilterPolicy()
// and must provide your own FilterPolicy that also ignores the
// corresponding parts of the keys.  For example, if the comparator
// ignores trailing spaces, it would be incorrect to use a
// FilterPolicy (like NewBloomFilterPolicy) that does not ignore
// trailing spaces in keys.
func NewBloomFilterFull(bitsPerKey float64) *NativeFilterPolicy {
	cFilter := C.rocksdb_filterpolicy_create_bloom_full(C.double(bitsPerKey))
	return newNativeFilterPolicy(cFilter)
}

// NewRibbonFilterPolicy creates a new Bloom alternative that saves about
// 30% space compared to Bloom filters, with similar query times but
// roughly 3-4x CPU time and 3x temporary space usage during construction.
//
// For example:
// if you pass in 10 for bloom_equivalent_bits_per_key, you'll get the same
// 0.95% FP rate as Bloom filter but only using about 7 bits per key.
//
// The space savings of Ribbon filters makes sense for lower (higher
// numbered; larger; longer-lived) levels of LSM, whereas the speed of
// Bloom filters make sense for highest levels of LSM.
//
// Ribbon filters are compatible with RocksDB >= 6.15.0. Earlier
// versions reading the data will behave as if no filter was used
// (degraded performance until compaction rebuilds filters). All
// built-in FilterPolicies (Bloom or Ribbon) are able to read other
// kinds of built-in filters.
//
// Note: the current Ribbon filter schema uses some extra resources
// when constructing very large filters. For example, for 100 million
// keys in a single filter (one SST file without partitioned filters),
// 3GB of temporary, untracked memory is used, vs. 1GB for Bloom.
// However, the savings in filter space from just ~60 open SST files
// makes up for the additional temporary memory use.
//
// Also consider using optimize_filters_for_memory to save filter
// memory.
func NewRibbonFilterPolicy(bloomEquivalentBitsPerKey float64) *NativeFilterPolicy {
	cFilter := C.rocksdb_filterpolicy_create_ribbon(C.double(bloomEquivalentBitsPerKey))
	return newNativeFilterPolicy(cFilter)
}

// NewRibbonHybridFilterPolicy similar to Ribbon.
//
// Setting bloom_before_level allows for this design with Level and Universal
// compaction styles. For example, bloom_before_level=1 means that Bloom
// filters will be used in level 0, including flushes, and Ribbon
// filters elsewhere, including FIFO compaction and external SST files.
// For this option, memtable flushes are considered level -1 (so that
// flushes can be distinguished from intra-L0 compaction).
// bloom_before_level=0 (default) -> Generate Bloom filters only for
// flushes under Level and Universal compaction styles.
// bloom_before_level=-1 -> Always generate Ribbon filters (except in
// some extreme or exceptional cases).
func NewRibbonHybridFilterPolicy(bloomEquivalentBitsPerKey float64, bloomBeforeLevel int) *NativeFilterPolicy {
	cFilter := C.rocksdb_filterpolicy_create_ribbon_hybrid(C.double(bloomEquivalentBitsPerKey), C.int(bloomBeforeLevel))
	return newNativeFilterPolicy(cFilter)
}
