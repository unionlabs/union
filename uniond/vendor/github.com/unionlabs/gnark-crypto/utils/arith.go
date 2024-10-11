package utils

// DivCeiling (a, b) = ⌈a/b⌉
func DivCeiling(a, b uint) uint {
	q := a / b
	if q*b == a {
		return q
	}
	return q + 1
}

func MinU(a, b uint) uint {
	if a < b {
		return a
	}
	return b
}

func Min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

func Max(a, b int) int {
	if a > b {
		return a
	}
	return b
}
