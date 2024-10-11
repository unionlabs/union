package uintgr

import (
	"github.com/thomaso-mirodin/intmath/constants/c32"
	"github.com/thomaso-mirodin/intmath/constants/c64"
)

//Bitcount returns the number of set bits in v
func BitCount(v uint) uint {
	if v<<32 != 0 {
		w := uint64(v)
		w -= w >> 1 & c64.BitCMask1
		w = w&c64.BitCMask3 + w>>2&c64.BitCMask3
		w = (w + w>>4) & c64.BitCMaskf
		return uint(w * c64.BitCMaskp >> 56)
	}
	v -= v >> 1 & c32.BitCMask1
	v = v&c32.BitCMask3 + v>>2&c32.BitCMask3
	v = (v + v>>4) & c32.BitCMaskf
	return v * c32.BitCMaskp >> 24
}
