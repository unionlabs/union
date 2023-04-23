// Package heap implements a heap on specific types.
package heap

import "container/heap"

// MinInts is a min-heap of integers.
type MinInts struct {
	h *intheap
}

// NewMinInts builds an empty integer min-heap.
func NewMinInts() *MinInts {
	return &MinInts{
		h: &intheap{},
	}
}

// Empty returns whether the heap is empty.
func (h *MinInts) Empty() bool {
	return h.Len() == 0
}

// Len returns the number of elements in the heap.
func (h *MinInts) Len() int {
	return h.h.Len()
}

// Push x onto the heap.
func (h *MinInts) Push(x int) {
	heap.Push(h.h, x)
}

// Pop the min element from the heap.
func (h *MinInts) Pop() int {
	return heap.Pop(h.h).(int)
}

type intheap struct {
	x []int
}

func (h intheap) Len() int           { return len(h.x) }
func (h intheap) Less(i, j int) bool { return h.x[i] < h.x[j] }
func (h intheap) Swap(i, j int)      { h.x[i], h.x[j] = h.x[j], h.x[i] }

func (h *intheap) Push(x interface{}) {
	h.x = append(h.x, x.(int))
}

func (h *intheap) Pop() interface{} {
	n := len(h.x)
	x := h.x[n-1]
	h.x = h.x[:n-1]
	return x
}
