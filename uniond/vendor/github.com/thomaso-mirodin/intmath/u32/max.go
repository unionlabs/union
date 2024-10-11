package u32

// Max(x,y) returns the larger of x or y
func Max(x, y uint32) uint32 {
	if x > y {
		return x
	}
	return y
}
