package intgr

import (
	"github.com/thomaso-mirodin/intmath/constants/c32"
	"github.com/thomaso-mirodin/intmath/constants/c64"
)

//Bitcount returns the number of set bits in v
func BitCount(v int) int {
	if v<<32 != 0 {
		w := uint64(v)
		w -= w >> 1 & c64.BitCMask1
		w = w&c64.BitCMask3 + w>>2&c64.BitCMask3
		w = (w + w>>4) & c64.BitCMaskf
		return int(w * c64.BitCMaskp >> 56)
	}
	w := uint(v)
	w -= w >> 1 & c32.BitCMask1
	w = w&c32.BitCMask3 + w>>2&c32.BitCMask3
	w = (w + w>>4) & c32.BitCMaskf
	return int(w * c32.BitCMaskp >> 24)
}
