package i64

// GCD returns the greatest common divisor of a and b.
func GCD(a, b int64) int64 {
	//Set a and b the absolute value of a and b.
	a = a ^ a>>63 - a>>63
	b = b ^ b>>63 - b>>63
	for b != 0 {
		a, b = b, a%b
	}
	return a
}
