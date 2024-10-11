package i32

import (
	"github.com/thomaso-mirodin/intmath/constants/c32"
)

//Bitcount returns the number of set bits in v.
//Recall that Go represent integers in two's complement.
func BitCount(v int32) int32 {
	w := uint32(v)
	w -= w >> 1 & c32.BitCMask1
	w = w&c32.BitCMask3 + w>>2&c32.BitCMask3
	w = (w + w>>4) & c32.BitCMaskf
	return int32(w * c32.BitCMaskp >> 24)
}
