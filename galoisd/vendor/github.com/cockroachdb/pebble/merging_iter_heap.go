// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

type mergingIterHeap struct {
	cmp     Compare
	reverse bool
	items   []*mergingIterLevel
}

func (h *mergingIterHeap) len() int {
	return len(h.items)
}

func (h *mergingIterHeap) clear() {
	h.items = h.items[:0]
}

func (h *mergingIterHeap) less(i, j int) bool {
	ikey, jkey := h.items[i].iterKey, h.items[j].iterKey
	if c := h.cmp(ikey.UserKey, jkey.UserKey); c != 0 {
		if h.reverse {
			return c > 0
		}
		return c < 0
	}
	if h.reverse {
		return ikey.Trailer < jkey.Trailer
	}
	return ikey.Trailer > jkey.Trailer
}

func (h *mergingIterHeap) swap(i, j int) {
	h.items[i], h.items[j] = h.items[j], h.items[i]
}

// init, fix, up and down are copied from the go stdlib.
func (h *mergingIterHeap) init() {
	// heapify
	n := h.len()
	for i := n/2 - 1; i >= 0; i-- {
		h.down(i, n)
	}
}

func (h *mergingIterHeap) fix(i int) {
	if !h.down(i, h.len()) {
		h.up(i)
	}
}

func (h *mergingIterHeap) pop() *mergingIterLevel {
	n := h.len() - 1
	h.swap(0, n)
	h.down(0, n)
	item := h.items[n]
	h.items = h.items[:n]
	return item
}

func (h *mergingIterHeap) up(j int) {
	for {
		i := (j - 1) / 2 // parent
		if i == j || !h.less(j, i) {
			break
		}
		h.swap(i, j)
		j = i
	}
}

func (h *mergingIterHeap) down(i0, n int) bool {
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
