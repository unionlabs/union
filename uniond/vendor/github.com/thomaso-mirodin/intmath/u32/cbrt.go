package u32

// Cbrt returns the integer cube root of n. That is to say, the r such that:
//
// r * r * r <= n && n < (r + 1) * (r + 1) * (r + 1)
//
// Adapted from code found in Hacker's Delight, fixed by Fabian Giessen
// https://gist.github.com/729557
func Cbrt(n uint32) uint32 {
	var b, r uint
	x := uint(n)
	for s := uint(30); s > 0; s -= 3 {
		r <<= 1
		b = 3*r*(r+1) + 1
		if x>>s >= b {
			x -= b << s
			r++
		}
	}
	r <<= 1
	b = 3*r*(r+1) + 1
	if x >= b {
		r++
	}
	return uint32(r)
}
