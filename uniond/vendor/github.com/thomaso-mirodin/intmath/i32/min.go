package i32

// Min(x,y) returns the smaller of x or y
func Min(x, y int32) int32 {
	if x < y {
		return x
	}
	return y
}
