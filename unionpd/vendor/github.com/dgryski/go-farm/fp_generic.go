// +build !amd64 purego

package farm

// Fingerprint64 is a 64-bit fingerprint function for byte-slices
func Fingerprint64(s []byte) uint64 {
	return naHash64(s)
}

// Fingerprint32 is a 32-bit fingerprint function for byte-slices
func Fingerprint32(s []byte) uint32 {
	return Hash32(s)
}
