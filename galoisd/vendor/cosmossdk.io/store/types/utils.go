package types

import (
	"encoding/binary"
	"fmt"
	"sort"
	"strings"
)

// KVStorePrefixIterator iterates over all the keys with a certain prefix in ascending order
func KVStorePrefixIterator(kvs KVStore, prefix []byte) Iterator {
	return kvs.Iterator(prefix, PrefixEndBytes(prefix))
}

// KVStoreReversePrefixIterator iterates over all the keys with a certain prefix in descending order.
func KVStoreReversePrefixIterator(kvs KVStore, prefix []byte) Iterator {
	return kvs.ReverseIterator(prefix, PrefixEndBytes(prefix))
}

// PrefixEndBytes returns the []byte that would end a
// range query for all []byte with a certain prefix
// Deals with last byte of prefix being FF without overflowing
func PrefixEndBytes(prefix []byte) []byte {
	if len(prefix) == 0 {
		return nil
	}

	end := make([]byte, len(prefix))
	copy(end, prefix)

	for {
		if end[len(end)-1] != byte(255) {
			end[len(end)-1]++
			break
		}

		end = end[:len(end)-1]

		if len(end) == 0 {
			end = nil
			break
		}
	}

	return end
}

// InclusiveEndBytes returns the []byte that would end a
// range query such that the input would be included
func InclusiveEndBytes(inclusiveBytes []byte) []byte {
	return append(inclusiveBytes, byte(0x00))
}

// assertNoCommonPrefix will panic if there are two keys: k1 and k2 in keys, such that
// k1 is a prefix of k2
func assertNoCommonPrefix(keys []string) {
	sorted := make([]string, len(keys))
	copy(sorted, keys)
	sort.Strings(sorted)
	for i := 1; i < len(sorted); i++ {
		if strings.HasPrefix(sorted[i], sorted[i-1]) {
			panic(fmt.Sprint("Potential key collision between KVStores:", sorted[i], " - ", sorted[i-1]))
		}
	}
}

// Uint64ToBigEndian - marshals uint64 to a bigendian byte slice so it can be sorted
func Uint64ToBigEndian(i uint64) []byte {
	b := make([]byte, 8)
	binary.BigEndian.PutUint64(b, i)
	return b
}

// BigEndianToUint64 returns an uint64 from big endian encoded bytes. If encoding
// is empty, zero is returned.
func BigEndianToUint64(bz []byte) uint64 {
	if len(bz) == 0 {
		return 0
	}

	return binary.BigEndian.Uint64(bz)
}

// SliceContains implements a generic function for checking if a slice contains
// a certain value.
func SliceContains[T comparable](elements []T, v T) bool {
	for _, s := range elements {
		if v == s {
			return true
		}
	}

	return false
}
