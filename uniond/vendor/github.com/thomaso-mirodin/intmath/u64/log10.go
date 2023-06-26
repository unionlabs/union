package u64

// Log10 returns log base 10 of n. n == 0 returns 0
func Log10(n uint64) uint64 {
	// Slightly counterintuitively, the optimisation used when checking if uintgr
	// is 32 or 64 bits does not apply to uniformly distributed uint64 numbers,
	// as half of them will have the highest bit set on average, half of the 
	// remaining the second-highest, and so on.
	// See also: http://graphics.stanford.edu/~seander/bithacks.html#IntegerLog10
	switch {
	case n >= 10000000000000000000:
		return 19
	case n >= 1000000000000000000:
		return 18
	case n >= 100000000000000000:
		return 17
	case n >= 10000000000000000:
		return 16
	case n >= 1000000000000000:
		return 15
	case n >= 100000000000000:
		return 14
	case n >= 10000000000000:
		return 13
	case n >= 1000000000000:
		return 12
	case n >= 100000000000:
		return 11
	case n >= 10000000000:
		return 10
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
