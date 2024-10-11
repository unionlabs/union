package u32

// Sqrt returns the square root of x. 
// Based on code found in Hacker's Delight (Addison-Wesley, 2003):
// http://www.hackersdelight.org/
func Sqrt(x uint32) uint32 {
	//Using uint guarantees native word width
	var b, r uint
	t := uint(x)
	//Fast way to make p highest power of 4 <= x
	p := t
	if p >= 1<<16 {
		p >>= 16
		b = 16
	}
	if p >= 1<<8 {
		p >>= 8
		b += 8
	}
	if p >= 1<<4 {
		p >>= 4
		b += 4
	}
	if p >= 1<<2 {
		b += 2
	}
	p = 1 << b
	for ; p != 0; p >>= 2 {
		b = r | p
		r >>= 1
		if t >= b {
			t -= b
			r |= p
		}
	}
	return uint32(r)
}
