package i32

import (
	"github.com/thomaso-mirodin/intmath/constants/c32"
)

// TrailingZeros returns a byte with the number of trailing 0 bits in v.
// Remember that Go represents negative integers in two's complement.
// If v is 0, it returns 0.
func TrailingZeros(v int32) int32 {
	// Adapted from Sonia Keys' xmath library
	w := uint32(v)
	return int32(c32.DeBruijnBits[w&-w*c32.DeBruijnMultiple>>c32.DeBruijnShift])
}
