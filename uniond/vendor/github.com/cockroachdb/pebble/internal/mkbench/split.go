package main

import "sort"

const increment = 50 // ops/sec

// findOptimalSplit computes and returns a value that separates the given pass
// and fail measurements optimally, such that the number of mis-classified
// passes (pass values that fall above the split) and fails (fail values that
// fall below the split) is minimized.
//
// The following gives a visual representation of the problem:
//
//		                     Optimal partition (=550) -----> |
//	                                                         |
//	  Passes:   o          o        o              o o o oo  |
//	  Fails:                         x             x         |x    x  x     x x        x
//	  |---------|---------|---------|---------|---------|----|----|---------|---------|---------|---> x
//	  0        100       200       300       400       500   |   600       700       800       900
//
// The algorithm works by computing the error (i.e. mis-classifications) at
// various points along the x-axis, starting from the origin and increasing by
// the given increment.
func findOptimalSplit(pass, fail []int) int {
	// Not enough data to compute a sensible score.
	if len(pass) == 0 || len(fail) == 0 {
		return -1
	}

	// Maintain counters for the number of incorrectly classified passes and
	// fails. All passes are initially incorrect, as we start at 0. Conversely,
	// no fails are incorrectly classified, as all scores are >= 0.
	pCount, fCount := len(pass), 0
	p, f := make([]int, len(pass)), make([]int, len(fail))
	copy(p, pass)
	copy(f, fail)

	// Sort the inputs.
	sort.Slice(p, func(i, j int) bool {
		return p[i] < p[j]
	})
	sort.Slice(f, func(i, j int) bool {
		return f[i] < f[j]
	})

	// Find the global min and max.
	min, max := p[0], f[len(fail)-1]

	// Iterate over the range in increments.
	var result [][]int
	for x := min; x <= max; x = x + increment {
		// Reduce the count of incorrect passes as x increases (i.e. fewer pass
		// values are incorrect as x increases).
		for len(p) > 0 && p[0] <= x {
			pCount--
			p = p[1:]
		}

		// Increase the count of incorrect fails as x increases (i.e. more fail
		// values are incorrect as x increases).
		for len(f) > 0 && f[0] < x {
			fCount++
			f = f[1:]
		}

		// Add a (x, score) tuple to result slice.
		result = append(result, []int{x, pCount + fCount})
	}

	// Sort the (x, score) result slice by score ascending. Tie-break by x
	// ascending.
	sort.Slice(result, func(i, j int) bool {
		if result[i][1] == result[j][1] {
			return result[i][0] < result[j][0]
		}
		return result[i][1] < result[j][1]
	})

	// If there is more than one interval, split the difference between the min
	// and the max.
	splitMin, splitMax := result[0][0], result[0][0]
	for i := 1; i < len(result); i++ {
		if result[i][1] != result[0][1] {
			break
		}
		splitMax = result[i][0]
	}

	return (splitMin + splitMax) / 2
}
