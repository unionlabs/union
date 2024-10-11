package u32

import (
	"github.com/thomaso-mirodin/intmath/constants/c32"
)

//Bitcount returns the number of set bits in v
func BitCount(v uint32) uint32 {
	v -= v >> 1 & c32.BitCMask1
	v = v&c32.BitCMask3 + v>>2&c32.BitCMask3
	v = (v + v>>4) & c32.BitCMaskf
	return v * c32.BitCMaskp >> 24
}
