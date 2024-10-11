package uintgr

//Is64bits returns true if integer is 64 bits.
func Is64bits() bool {
	v := uint(1)
	return v<<32 != 0
}
