package merkle

// MimcHashFromByteSlices computes a Merkle tree (mimc hash) where the leaves are the byte slice,
// in the provided order. It follows RFC-6962.
func MimcHashFromByteSlices(items [][]byte) []byte {
	switch len(items) {
	case 0:
		return emptyMimcHash()
	case 1:
		return leafMimcHash(items[0])
	default:
		k := getSplitPoint(int64(len(items)))
		left := MimcHashFromByteSlices(items[:k])
		right := MimcHashFromByteSlices(items[k:])
		return innerMimcHash(left, right)
	}
}
