package tree

import (
	"crypto/sha256"
	"hash"
	"math/bits"
)

var (
	leafPrefix  = []byte{0}
	innerPrefix = []byte{1}
)

// HashFromByteSlices computes a Merkle tree where the leaves are the byte slice,
// in the provided order. It follows RFC-6962.
func HashFromByteSlices(items [][]byte) []byte {
	return hashFromByteSlices(sha256.New(), items)
}

func hashFromByteSlices(sha hash.Hash, items [][]byte) []byte {
	switch len(items) {
	case 0:
		return emptyHash()
	case 1:
		return leafHashOpt(sha, items[0])
	default:
		k := getSplitPoint(int64(len(items)))
		left := hashFromByteSlices(sha, items[:k])
		right := hashFromByteSlices(sha, items[k:])
		return innerHashOpt(sha, left, right)
	}
}

// returns tmhash(0x00 || leaf)
func leafHashOpt(s hash.Hash, leaf []byte) []byte {
	s.Reset()
	s.Write(leafPrefix)
	s.Write(leaf)
	return s.Sum(nil)
}

func innerHashOpt(s hash.Hash, left, right []byte) []byte {
	s.Reset()
	s.Write(innerPrefix)
	s.Write(left)
	s.Write(right)
	return s.Sum(nil)
}

// returns tmhash(<empty>)
func emptyHash() []byte {
	h := sha256.Sum256([]byte{})
	return h[:]
}

// getSplitPoint returns the largest power of 2 less than length
func getSplitPoint(length int64) int64 {
	if length < 1 {
		panic("Trying to split a tree with size < 1")
	}
	uLength := uint(length)
	bitlen := bits.Len(uLength)
	k := int64(1 << uint(bitlen-1))
	if k == length {
		k >>= 1
	}
	return k
}
