package i32

// GCD returns the greatest common divisor of a and b.
func GCD(a, b int32) int32 {
	//Set a and b the absolute value of a and b.
	a = a ^ a>>31 - a>>31
	b = b ^ b>>31 - b>>31
	for b != 0 {
		a, b = b, a%b
	}
	return a
}
