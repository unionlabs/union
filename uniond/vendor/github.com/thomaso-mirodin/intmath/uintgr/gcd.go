package uintgr

// GCD returns the greatest common divisor of a and b.
func GCD(a, b uint) uint {
	for b != 0 {
		a, b = b, a%b
	}
	return a
}
