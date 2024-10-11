package intgr

// GCD returns the greatest common divisor of a and b.
func GCD(a, b int) int {
	a = Abs(a)
	b = Abs(b)
	for b != 0 {
		a, b = b, a%b
	}
	return a
}
