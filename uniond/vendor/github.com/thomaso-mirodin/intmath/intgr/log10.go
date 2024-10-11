package intgr

// Log10 returns log base 10 of n. n <= 0 returns -1
func Log10(n int) int {
	if n <= 0 {
		return -1
	}
	// Since we have to check if int is 32 or 64 bits anyway,
	// we might as well turn it into an optimisation.
	if v := int64(n) & 0x7FFFFFFF00000000; v != 0 {
		switch {
		case v >= 1000000000000000000:
			return 18
		case v >= 100000000000000000:
			return 17
		case v >= 10000000000000000:
			return 16
		case v >= 1000000000000000:
			return 15
		case v >= 100000000000000:
			return 14
		case v >= 10000000000000:
			return 13
		case v >= 1000000000000:
			return 12
		case v >= 100000000000:
			return 11
		case v >= 10000000000:
			return 10
		}
	}
	switch {
	case n >= 1000000000:
		return 9
	case n >= 100000000:
		return 8
	case n >= 10000000:
		return 7
	case n >= 1000000:
		return 6
	case n >= 100000:
		return 5
	case n >= 10000:
		return 4
	case n >= 1000:
		return 3
	case n >= 100:
		return 2
	case n >= 10:
		return 1
	}
	return 0
}
