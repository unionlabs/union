package intgr

// Cbrt returns the integer cube root of n. That is to say, the r such that:
//
// r * r * r <= n && n < (r + 1) * (r + 1) * (r + 1) //for positive n
// r * r * r >= n && n > (r + 1) * (r + 1) * (r + 1) //for negative n
//
// Adapted from code found in Hacker's Delight, fixed by Fabian Giessen
// https://gist.github.com/729557
func Cbrt(n int) int {
	if n < 0 {
		return -Cbrt(-n)
	}
	var b, r uint
	s := uint(1)
	if s<<32 != 0 {
		s = 63
	} else {
		s = 30
	}
	x := uint(n)
	for ; s > 0; s -= 3 {
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
	return int(r)
}
