package u64

// Sqrt returns the square root of x. 
// Based on code found in Hacker's Delight (Addison-Wesley, 2003):
// http://www.hackersdelight.org/
func Sqrt(x uint64) (r uint64) {
	var b uint64
	//Fast way to make p highest power of 4 <= x
	p := x
	var n, v uint
	if p >= 1<<32 {
		v = uint(p >> 32)
		n = 32
	} else {
		v = uint(p)
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
	p = 1 << n

	for ; p != 0; p >>= 2 {
		b = r | p
		r >>= 1
		if x >= b {
			x -= b
			r |= p
		}
	}
	return
}
