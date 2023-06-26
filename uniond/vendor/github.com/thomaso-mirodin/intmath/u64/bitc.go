package u64

import (
	"github.com/thomaso-mirodin/intmath/constants/c64"
)

//Bitcount returns the number of set bits in v
func BitCount(v uint64) uint64 {
	v -= v >> 1 & c64.BitCMask1
	v = v&c64.BitCMask3 + v>>2&c64.BitCMask3
	v = (v + v>>4) & c64.BitCMaskf
	return v * c64.BitCMaskp >> 56
}
