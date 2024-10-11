package i64

import (
	"github.com/thomaso-mirodin/intmath/constants/c64"
)

//Bitcount returns the number of set bits in v.
//Recall that Go represent integers in two's complement.
func BitCount(v int64) int64 {
	w := uint64(v)
	w -= w >> 1 & c64.BitCMask1
	w = w&c64.BitCMask3 + w>>2&c64.BitCMask3
	w = (w + w>>4) & c64.BitCMaskf
	return int64(w * c64.BitCMaskp >> 56)
}
