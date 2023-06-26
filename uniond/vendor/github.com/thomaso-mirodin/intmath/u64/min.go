package u64

// Min(x,y) returns the smaller of x or y
func Min(x, y uint64) uint64 {
	if x < y {
		return x
	}
	return y
}
