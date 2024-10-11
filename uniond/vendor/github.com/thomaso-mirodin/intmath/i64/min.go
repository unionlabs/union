package i64

// Min(x,y) returns the smaller of x or y
func Min(x, y int64) int64 {
	if x < y {
		return x
	}
	return y
}
