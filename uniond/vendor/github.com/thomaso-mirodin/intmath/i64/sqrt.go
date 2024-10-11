package i64

// Sqrt returns the square root of x. x < 0 returns -1. Based on a C implementation of 
// Newton's Method using bitshifting, originally found here:
// http://www.codecodex.com/wiki/Calculate_an_integer_square_root#C
func Sqrt(x int64) int64 {
	if x < 0 {
		return -1
	}
	// p starts at the highest power of four less or equal to x
	//Fast way to make p highest power of 4 <= x
	var v, n uint
	if x > 1<<32 {
		v = uint(x >> 32)
		n = 32
	} else {
		v = uint(x)
	}
	if v >= 1<<16 {
		v >>= 16
		n += 16
	}
	if v >= 1<<8 {
		v >>= 8
		n += 8
	}
	if v >= 1<<4 {
		v >>= 4
		n += 4
	}
	if v >= 1<<2 {
		n += 2
	}
	var r, b int64
	for p := int64(1 << n); p != 0; p >>= 2 {
		b = r | p
		r >>= 1
		if x >= b {
			x -= b
			r |= p
		}
	}
	return r
}
