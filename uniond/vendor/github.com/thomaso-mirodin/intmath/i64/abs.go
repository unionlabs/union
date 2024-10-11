package i64

// Abs returns the absolute value of x.
func Abs(x int64) int64 {
	return x ^ x>>63 - x>>63
}
