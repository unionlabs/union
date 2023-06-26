package u64

// Cbrt returns the integer cube root of n. That is to say, the r such that:
//
// r * r * r <= n && n < (r + 1) * (r + 1) * (r + 1)
//
// Adapted from code found in Hacker's Delight, fixed by Fabian Giessen
// https://gist.github.com/729557
func Cbrt(n uint64) (r uint64) {
	var b uint64
	for s := uint(63); s > 0; s -= 3 {
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
