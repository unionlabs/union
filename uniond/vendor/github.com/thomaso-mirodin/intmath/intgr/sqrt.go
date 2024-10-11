package intgr

// Sqrt returns the square root of x. 
// Based on code found in Hacker's Delight (Addison-Wesley, 2003):
// http://www.hackersdelight.org/
func Sqrt(x int) (r int) {
	if x < 0 {
		return -1
	}

	//Fast way to make p highest power of 4 <= x
	var n uint
	p := x
	if int64(p) >= 1<<32 {
		p >>= 32
		n = 32
	}
	if p >= 1<<16 {
		p >>= 16
		n += 16
	}
	if p >= 1<<8 {
		p >>= 8
		n += 8
	}
	if p >= 1<<4 {
		p >>= 4
		n += 4
	}
	if p >= 1<<2 {
		n += 2
	}
	p = 1 << n
	var b int
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
