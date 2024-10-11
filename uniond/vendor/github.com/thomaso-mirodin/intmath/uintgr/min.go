package uintgr

// Min(x,y) returns the smaller of x or y
func Min(x, y uint) uint {
	if x < y {
		return x
	}
	return y
}
