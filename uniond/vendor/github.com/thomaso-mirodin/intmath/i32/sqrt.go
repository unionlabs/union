package i32

// Sqrt returns the square root of x. x < 0 returns -1.
// Based on code found in Hacker's Delight (Addison-Wesley, 2003):
// http://www.hackersdelight.org/
func Sqrt(x int32) int32 {
	if x < 0 {
		return -1
	}
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
	return int32(r)
}
