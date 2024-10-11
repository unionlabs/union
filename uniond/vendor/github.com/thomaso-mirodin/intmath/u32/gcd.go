package u32

// GCD returns the greatest common divisor of a and b.
func GCD(a, b uint32) uint32 {
	for b != 0 {
		a, b = b, a%b
	}
	return a
}
