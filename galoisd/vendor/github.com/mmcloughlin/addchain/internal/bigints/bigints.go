// Package bigints provides helpers for slices of multi-precision integers.
package bigints

import (
	"math/big"
	"sort"

	"github.com/mmcloughlin/addchain/internal/bigint"
)

// Int64s converts a list of int64s into a slice of big integers.
func Int64s(xs ...int64) []*big.Int {
	bs := make([]*big.Int, len(xs))
	for i, x := range xs {
		bs[i] = big.NewInt(x)
	}
	return bs
}

// ascending sorts integers in ascending order.
type ascending []*big.Int

func (a ascending) Len() int           { return len(a) }
func (a ascending) Swap(i, j int)      { a[i], a[j] = a[j], a[i] }
func (a ascending) Less(i, j int) bool { return a[i].Cmp(a[j]) < 0 }

// Sort in ascending order.
func Sort(xs []*big.Int) {
	sort.Sort(ascending(xs))
}

// Index returns the index of the first occurrence of n in xs, or -1 if it does not appear.
func Index(n *big.Int, xs []*big.Int) int {
	for i, x := range xs {
		if bigint.Equal(n, x) {
			return i
		}
	}
	return -1
}

// Contains reports whether n is in xs.
func Contains(n *big.Int, xs []*big.Int) bool {
	return Index(n, xs) >= 0
}

// ContainsSorted reports whether n is in xs, which is assumed to be sorted.
func ContainsSorted(n *big.Int, xs []*big.Int) bool {
	i := sort.Search(len(xs), func(i int) bool { return xs[i].Cmp(n) >= 0 })
	return i < len(xs) && bigint.Equal(xs[i], n)
}

// Clone a list of integers.
func Clone(xs []*big.Int) []*big.Int {
	return append([]*big.Int{}, xs...)
}

// Unique removes consecutive duplicates.
func Unique(xs []*big.Int) []*big.Int {
	if len(xs) == 0 {
		return []*big.Int{}
	}
	u := make([]*big.Int, 1, len(xs))
	u[0] = xs[0]
	for _, x := range xs[1:] {
		last := u[len(u)-1]
		if !bigint.Equal(x, last) {
			u = append(u, x)
		}
	}
	return u
}

// InsertSortedUnique inserts an integer into a slice of sorted distinct
// integers.
func InsertSortedUnique(xs []*big.Int, x *big.Int) []*big.Int {
	return MergeUnique([]*big.Int{x}, xs)
}

// MergeUnique merges two slices of sorted distinct integers. Elements in both
// slices are deduplicated.
func MergeUnique(xs, ys []*big.Int) []*big.Int {
	r := make([]*big.Int, 0, len(xs)+len(ys))

	for len(xs) > 0 && len(ys) > 0 {
		switch xs[0].Cmp(ys[0]) {
		case -1:
			r = append(r, xs[0])
			xs = xs[1:]
		case 0:
			r = append(r, xs[0])
			xs = xs[1:]
			ys = ys[1:]
		case 1:
			r = append(r, ys[0])
			ys = ys[1:]
		}
	}

	r = append(r, xs...)
	r = append(r, ys...)

	return r
}
