package u64

// GCD returns the greatest common divisor of a and b.
func GCD(a, b uint64) uint64 {
	for b != 0 {
		a, b = b, a%b
	}
	return a
}
