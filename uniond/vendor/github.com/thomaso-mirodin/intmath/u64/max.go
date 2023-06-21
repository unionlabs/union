package u64

// Max(x,y) returns the larger of x or y
func Max(x, y uint64) uint64 {
	if x > y {
		return x
	}
	return y
}
