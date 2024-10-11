package i64

import (
	"github.com/thomaso-mirodin/intmath/constants/c64"
)

// TrailingZeros returns a byte with the number of trailing 0 bits in v.
// Remember that Go represents negative integers in two's complement.
// If v is 0, it returns 0.
func TrailingZeros(v int64) int64 {
	// Adapted from Sonia Keys' xmath library
	w := uint64(v)
	return int64(c64.DeBruijnBits[w&-w*c64.DeBruijnMultiple>>c64.DeBruijnShift])
}
