//Package c64 stores look-up tables and similar constants for 64bit 
//wide integers
package c64

// copied from Sonia Keys' xmath library, found at:
// https://github.com/soniakeys/integer/blob/master/xmath/xmath.go

// reference: http://graphics.stanford.edu/~seander/bithacks.html
const DeBruijnMultiple = 0x03f79d71b4ca8b09
const DeBruijnShift = 58

var DeBruijnBits = []byte{
	0, 1, 56, 2, 57, 49, 28, 3, 61, 58, 42, 50, 38, 29, 17, 4,
	62, 47, 59, 36, 45, 43, 51, 22, 53, 39, 33, 30, 24, 18, 12, 5,
	63, 55, 48, 27, 60, 41, 37, 16, 46, 35, 44, 21, 52, 32, 23, 11,
	54, 26, 40, 15, 34, 20, 31, 10, 25, 14, 19, 9, 13, 8, 7, 6,
}

const (
	ff        = 1<<64 - 1
	BitCMask1 = ff / 3
	BitCMask3 = ff / 5
	BitCMaskf = ff / 17
	BitCMaskp = BitCMaskf >> 3 & BitCMaskf
)
