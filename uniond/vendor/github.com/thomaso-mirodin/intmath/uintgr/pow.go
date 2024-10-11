package uintgr

// Pow returns x**y, the base-x exponential of y.
func Pow(x, y uint) (r uint) {
	if x == r {
		return
	}
	r = 1
	if x == r {
		return
	}
	for y > 0 {
		if y&1 == 1 {
			r *= x
		}
		x *= x
		y >>= 1
	}
	return
}
