package intgr

import (
	"github.com/thomaso-mirodin/intmath/constants/c32"
	"github.com/thomaso-mirodin/intmath/constants/c64"
)

// TrailingZeros returns a byte with the number of trailing 0 bits in v.
// Remember that Go represents negative integers in two's complement.
// If v is 0, it returns 0.
func TrailingZeros(v int) int {
	// Adapted from Sonia Keys' xmath library
	if v<<32 != 0 {
		w := uint64(v)
		return int(c64.DeBruijnBits[w&-w*c64.DeBruijnMultiple>>c64.DeBruijnShift])
	}
	w := uint(v)
	return int(c32.DeBruijnBits[w&-w*c32.DeBruijnMultiple>>c32.DeBruijnShift])
}
