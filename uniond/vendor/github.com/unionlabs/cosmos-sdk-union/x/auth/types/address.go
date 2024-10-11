package types

import "crypto/sha256"

const TruncatedSize = 20

// AddressHash generates a hash of the given byte slice.
//
// bz: the byte slice to be hashed.
// []byte: the hashed result.
func AddressHash(bz []byte) []byte {
	return SumTruncated(bz)
}

// SumTruncated calculates the SHA256 hash of the given byte slice and returns the first TruncatedSize bytes.
//
// Parameters:
// - bz: the byte slice to calculate the hash for.
//
// Returns:
// - []byte: the first TruncatedSize bytes of the calculated hash.
func SumTruncated(bz []byte) []byte {
	hash := sha256.Sum256(bz)
	return hash[:TruncatedSize]
}
