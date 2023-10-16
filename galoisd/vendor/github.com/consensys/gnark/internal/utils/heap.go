package utils

// An IntHeap is a min-heap of linear expressions. It facilitates merging k-linear expressions.
//
// The code is identical to https://pkg.go.dev/container/heap but replaces interfaces with concrete
// type to avoid memory overhead.
type IntHeap []int

func (h *IntHeap) less(i, j int) bool { return (*h)[i] < (*h)[j] }
func (h *IntHeap) swap(i, j int)      { (*h)[i], (*h)[j] = (*h)[j], (*h)[i] }

// Heapify establishes the heap invariants required by the other routines in this package.
// Heapify is idempotent with respect to the heap invariants
// and may be called whenever the heap invariants may have been invalidated.
// The complexity is O(n) where n = len(*h).
func (h *IntHeap) Heapify() {
	// heapify
	n := len(*h)
	for i := n/2 - 1; i >= 0; i-- {
		h.down(i, n)
	}
}

// Pop removes and returns the minimum element (according to Less) from the heap.
// The complexity is O(log n) where n = len(*h).
// Pop is equivalent to Remove(h, 0).
func (h *IntHeap) Pop() {
	n := len(*h) - 1
	h.swap(0, n)
	h.down(0, n)
	*h = (*h)[0:n]
}

func (h *IntHeap) up(j int) {
	for {
		i := (j - 1) / 2 // parent
		if i == j || !h.less(j, i) {
			break
		}
		h.swap(i, j)
		j = i
	}
}

func (h *IntHeap) down(i0, n int) bool {
	i := i0
	for {
		j1 := 2*i + 1
		if j1 >= n || j1 < 0 { // j1 < 0 after int overflow
			break
		}
		j := j1 // left child
		if j2 := j1 + 1; j2 < n && h.less(j2, j1) {
			j = j2 // = 2*i + 2  // right child
		}
		if !h.less(j, i) {
			break
		}
		h.swap(i, j)
		i = j
	}
	return i > i0
}
