package u32

import (
	"github.com/thomaso-mirodin/intmath/constants/c32"
)

// TrailingZeros returns the number of trailing 0 bits in v. If v is 0, it returns 0.
func TrailingZeros(v uint32) uint32 {
	// Adapted from Sonia Keys' xmath library
	return uint32(c32.DeBruijnBits[v&-v*c32.DeBruijnMultiple>>c32.DeBruijnShift])
}
