package internal

func ConcatAll(slices ...[]int) []int { // copyright note: written by GitHub Copilot
	totalLen := 0
	for _, s := range slices {
		totalLen += len(s)
	}
	res := make([]int, totalLen)
	i := 0
	for _, s := range slices {
		i += copy(res[i:], s)
	}
	return res
}

func NbElements(slices [][]int) int { // copyright note: written by GitHub Copilot
	totalLen := 0
	for _, s := range slices {
		totalLen += len(s)
	}
	return totalLen
}
