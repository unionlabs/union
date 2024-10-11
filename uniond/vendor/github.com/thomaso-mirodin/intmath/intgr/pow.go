package intgr

// Pow returns x**y, the base-x exponential of y.
func Pow(x, y int) (r int) {
	if x == r || y < r {
		return
	}
	r = 1
	if x == r {
		return
	}
	if x < 0 {
		x = -x
		if y&1 == 1 {
			r = -1
		}
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
