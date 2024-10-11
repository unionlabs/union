//Package c32 stores look-up tables and similar constants for 32bit 
//wide integers
package c32

// copied from Sonia Keys' xmath library, found at:
// https://github.com/soniakeys/integer/blob/master/xmath/xmath.go

// reference: http://graphics.stanford.edu/~seander/bithacks.html
const DeBruijnMultiple = 0x077CB531
const DeBruijnShift = 27

var DeBruijnBits = []byte{
	0, 1, 28, 2, 29, 14, 24, 3, 30, 22, 20, 15, 25, 17, 4, 8,
	31, 27, 13, 23, 21, 19, 16, 7, 26, 12, 18, 6, 11, 5, 10, 9,
}

const (
	ff        = 1<<32 - 1
	BitCMask1 = ff / 3
	BitCMask3 = ff / 5
	BitCMaskf = ff / 17
	BitCMaskp = ff / 255
)
