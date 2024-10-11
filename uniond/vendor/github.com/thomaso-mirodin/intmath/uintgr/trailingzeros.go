package uintgr

import (
	"github.com/thomaso-mirodin/intmath/constants/c32"
	"github.com/thomaso-mirodin/intmath/constants/c64"
)

// TrailingZeros returns the number of trailing 0 bits in v. If v is 0, it returns 0.
func TrailingZeros(v uint) uint {
	// Adapted from Sonia Keys' xmath library
	if v<<32 != 0 {
		w := uint64(v)
		return uint(c64.DeBruijnBits[w&-w*c64.DeBruijnMultiple>>c64.DeBruijnShift])
	}
	return uint(c32.DeBruijnBits[v&-v*c32.DeBruijnMultiple>>c32.DeBruijnShift])
}
