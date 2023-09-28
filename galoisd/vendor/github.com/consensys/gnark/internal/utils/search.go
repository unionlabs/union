package utils

import "sort"

// FindInSlice attempts to find the target in increasing slice x.
// If not found, returns false and the index where the target would be inserted.
func FindInSlice(x []int, target int) (int, bool) {
	return sort.Find(len(x), func(i int) int {
		return target - x[i]
	})
}

// MultiListSeeker looks up increasing integers in a list of increasing lists of integers.
type MultiListSeeker [][]int

// Seek returns the index of the earliest list where n is found, or -1 if not found.
func (s MultiListSeeker) Seek(n int) int {
	for i, l := range s {
		j, found := FindInSlice(l, n)
		s[i] = l[j:]
		if found {
			return i
		}
	}
	return -1
}
