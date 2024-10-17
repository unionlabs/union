package crypto

import (
	crand "crypto/rand"
	"encoding/hex"
	"io"
)

// This only uses the OS's randomness.
func randBytes(numBytes int) []byte {
	b := make([]byte, numBytes)
	_, err := crand.Read(b)
	if err != nil {
		panic(err)
	}
	return b
}

// This only uses the OS's randomness.
func CRandBytes(numBytes int) []byte {
	return randBytes(numBytes)
}

// CRandHex returns a hex encoded string that's floor(numDigits/2) * 2 long.
//
// Note: CRandHex(24) gives 96 bits of randomness that
// are usually strong enough for most purposes.
func CRandHex(numDigits int) string {
	return hex.EncodeToString(CRandBytes(numDigits / 2))
}

// Returns a crand.Reader.
func CReader() io.Reader {
	return crand.Reader
}
