// Copyright (c) 2021 Hirotsuna Mizuno. All rights reserved.
// Use of this source code is governed by the MIT license that can be found in
// the LICENSE file.

package bitarray

import (
	"sort"
)

// Slice attaches the methods of sort.Interface to []*BitArray, sorting in
// increasing order.
type Slice []*BitArray

func (s Slice) Len() int           { return len(s) }
func (s Slice) Less(i, j int) bool { return Compare(s[i], s[j]) < 0 }
func (s Slice) Swap(i, j int)      { s[i], s[j] = s[j], s[i] }

// Sort is a convenience method: s.Sort() calls sort.Sort(s).
func (s Slice) Sort() { sort.Sort(s) }

// Search searches for x in the sorted slice s using binary search and returns
// the index. The return value is the index to insert x if x is not present (it
// could be s.Len()). The slice must be sorted in ascending order.
func (s Slice) Search(x BitArrayer) int {
	return sort.Search(len(s), func(i int) bool { return 0 <= Compare(s[i], x) })
}
