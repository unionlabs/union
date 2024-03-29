package keyformat

import "encoding/binary"

// This file builds some dedicated key formatters for what appears in benchmarks.

// Prefixes a single byte before a 32 byte hash.
type FastPrefixFormatter struct {
	prefix      byte
	length      int
	prefixSlice []byte
}

func NewFastPrefixFormatter(prefix byte, length int) *FastPrefixFormatter {
	return &FastPrefixFormatter{prefix: prefix, length: length, prefixSlice: []byte{prefix}}
}

func (f *FastPrefixFormatter) Key(bz []byte) []byte {
	key := make([]byte, 1+f.length)
	key[0] = f.prefix
	copy(key[1:], bz)
	return key
}

func (f *FastPrefixFormatter) Scan(key []byte, a interface{}) {
	scan(a, key[1:])
}

func (f *FastPrefixFormatter) KeyInt64(bz int64) []byte {
	key := make([]byte, 1+f.length)
	key[0] = f.prefix
	binary.BigEndian.PutUint64(key[1:], uint64(bz))
	return key
}

func (f *FastPrefixFormatter) Prefix() []byte {
	return f.prefixSlice
}

func (f *FastPrefixFormatter) Length() int {
	return 1 + f.length
}
