package r1cs

// An minHeap is a min-heap of linear expressions. It facilitates merging k-linear expressions.
//
// The code is identical to https://pkg.go.dev/container/heap but replaces interfaces with concrete
// type to avoid memory overhead.
type minHeap []linMeta

func (h minHeap) less(i, j int) bool { return h[i].val < h[j].val }
func (h minHeap) swap(i, j int)      { h[i], h[j] = h[j], h[i] }

// heapify establishes the heap invariants required by the other routines in this package.
// heapify is idempotent with respect to the heap invariants
// and may be called whenever the heap invariants may have been invalidated.
// The complexity is O(n) where n = len(*h).
func (h *minHeap) heapify() {
	// heapify
	n := len(*h)
	for i := n/2 - 1; i >= 0; i-- {
		h.down(i, n)
	}
}

// push the element x onto the heap.
// The complexity is O(log n) where n = len(*h).
func (h *minHeap) push(x linMeta) {
	*h = append(*h, x)
	h.up(len(*h) - 1)
}

// Pop removes and returns the minimum element (according to Less) from the heap.
// The complexity is O(log n) where n = len(*h).
// Pop is equivalent to Remove(h, 0).
func (h *minHeap) popHead() {
	n := len(*h) - 1
	h.swap(0, n)
	h.down(0, n)
	*h = (*h)[0:n]
}

// fix re-establishes the heap ordering after the element at index i has changed its value.
// Changing the value of the element at index i and then calling fix is equivalent to,
// but less expensive than, calling Remove(h, i) followed by a Push of the new value.
// The complexity is O(log n) where n = len(*h).
func (h *minHeap) fix(i int) {
	if !h.down(i, len(*h)) {
		h.up(i)
	}
}

func (h *minHeap) up(j int) {
	for {
		i := (j - 1) / 2 // parent
		if i == j || !h.less(j, i) {
			break
		}
		h.swap(i, j)
		j = i
	}
}

func (h *minHeap) down(i0, n int) bool {
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

// linMeta stores metadata to iterate over a linear expression
type linMeta struct {
	lID int // argument ID to retrieve the position of the list in the argument
	tID int // termID current iteration position (starts at 0)
	val int // current linearExp[tID].VID value
}
