package uintgr

// Cbrt returns the integer cube root of n. That is to say, the r such that:
//
// r * r * r <= n && n < (r + 1) * (r + 1) * (r + 1)
//
// Adapted from code found in Hacker's Delight, fixed by Fabian Giessen
// https://gist.github.com/729557
func Cbrt(n uint) (r uint) {
	var b uint
	s := uint(1)
	if s<<32 != 0 {
		s = 63
	} else {
		s = 30
	}
	for ; s > 0; s -= 3 {
		r <<= 1
		b = 3*r*(r+1) + 1
		if n>>s >= b {
			n -= b << s
			r++
		}
	}
	r <<= 1
	b = 3*r*(r+1) + 1
	if n >= b {
		r++
	}
	return
}
