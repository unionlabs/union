package u32

// Min(x,y) returns the smaller of x or y
func Min(x, y uint32) uint32 {
	if x < y {
		return x
	}
	return y
}
