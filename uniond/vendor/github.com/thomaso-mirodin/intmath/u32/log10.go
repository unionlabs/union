package u32

// Log10 returns log base 10 of n. n == 0 returns 0
func Log10(n uint32) uint32 {
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
