// Copyright 2020 Joshua J Baker. All rights reserved.
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file.
package btree

import "sync/atomic"

type ordered interface {
	~int | ~int8 | ~int16 | ~int32 | ~int64 |
		~uint | ~uint8 | ~uint16 | ~uint32 | ~uint64 | ~uintptr |
		~float32 | ~float64 | ~string
}

type copier[T any] interface {
	Copy() T
}

type isoCopier[T any] interface {
	IsoCopy() T
}

func degreeToMinMax(deg int) (min, max int) {
	if deg <= 0 {
		deg = 32
	} else if deg == 1 {
		deg = 2 // must have at least 2
	}
	max = deg*2 - 1 // max items per node. max children is +1
	min = max / 2
	return min, max
}

var gisoid uint64

func newIsoID() uint64 {
	return atomic.AddUint64(&gisoid, 1)
}

type mapPair[K ordered, V any] struct {
	// The `value` field should be before the `key` field because doing so
	// allows for the Go compiler to optimize away the `value` field when
	// it's a `struct{}`, which is the case for `btree.Set`.
	value V
	key   K
}

type Map[K ordered, V any] struct {
	isoid         uint64
	root          *mapNode[K, V]
	count         int
	empty         mapPair[K, V]
	min           int // min items
	max           int // max items
	copyValues    bool
	isoCopyValues bool
}

func NewMap[K ordered, V any](degree int) *Map[K, V] {
	m := new(Map[K, V])
	m.init(degree)
	return m
}

type mapNode[K ordered, V any] struct {
	isoid    uint64
	count    int
	items    []mapPair[K, V]
	children *[]*mapNode[K, V]
}

// Copy the node for safe isolation.
func (tr *Map[K, V]) copy(n *mapNode[K, V]) *mapNode[K, V] {
	n2 := new(mapNode[K, V])
	n2.isoid = tr.isoid
	n2.count = n.count
	n2.items = make([]mapPair[K, V], len(n.items), cap(n.items))
	copy(n2.items, n.items)
	if tr.copyValues {
		for i := 0; i < len(n2.items); i++ {
			n2.items[i].value =
				((interface{})(n2.items[i].value)).(copier[V]).Copy()
		}
	} else if tr.isoCopyValues {
		for i := 0; i < len(n2.items); i++ {
			n2.items[i].value =
				((interface{})(n2.items[i].value)).(isoCopier[V]).IsoCopy()
		}
	}
	if !n.leaf() {
		n2.children = new([]*mapNode[K, V])
		*n2.children = make([]*mapNode[K, V], len(*n.children), tr.max+1)
		copy(*n2.children, *n.children)
	}
	return n2
}

// isoLoad loads the provided node and, if needed, performs a copy-on-write.
func (tr *Map[K, V]) isoLoad(cn **mapNode[K, V], mut bool) *mapNode[K, V] {
	if mut && (*cn).isoid != tr.isoid {
		*cn = tr.copy(*cn)
	}
	return *cn
}

func (tr *Map[K, V]) Copy() *Map[K, V] {
	return tr.IsoCopy()
}

func (tr *Map[K, V]) IsoCopy() *Map[K, V] {
	tr2 := new(Map[K, V])
	*tr2 = *tr
	tr2.isoid = newIsoID()
	tr.isoid = newIsoID()
	return tr2
}

func (tr *Map[K, V]) newNode(leaf bool) *mapNode[K, V] {
	n := new(mapNode[K, V])
	n.isoid = tr.isoid
	if !leaf {
		n.children = new([]*mapNode[K, V])
	}
	return n
}

// leaf returns true if the node is a leaf.
func (n *mapNode[K, V]) leaf() bool {
	return n.children == nil
}

func (tr *Map[K, V]) search(n *mapNode[K, V], key K) (index int, found bool) {
	low, high := 0, len(n.items)
	for low < high {
		h := (low + high) / 2
		if !(key < n.items[h].key) {
			low = h + 1
		} else {
			high = h
		}
	}
	if low > 0 && !(n.items[low-1].key < key) {
		return low - 1, true
	}
	return low, false
}

func (tr *Map[K, V]) init(degree int) {
	if tr.min != 0 {
		return
	}
	tr.min, tr.max = degreeToMinMax(degree)
	_, tr.copyValues = ((interface{})(tr.empty.value)).(copier[V])
	if !tr.copyValues {
		_, tr.isoCopyValues = ((interface{})(tr.empty.value)).(isoCopier[V])
	}
}

// Set or replace a value for a key
func (tr *Map[K, V]) Set(key K, value V) (V, bool) {
	item := mapPair[K, V]{key: key, value: value}
	if tr.root == nil {
		tr.init(0)
		tr.root = tr.newNode(true)
		tr.root.items = append([]mapPair[K, V]{}, item)
		tr.root.count = 1
		tr.count = 1
		return tr.empty.value, false
	}
	prev, replaced, split := tr.nodeSet(&tr.root, item)
	if split {
		left := tr.root
		right, median := tr.nodeSplit(left)
		tr.root = tr.newNode(false)
		*tr.root.children = make([]*mapNode[K, V], 0, tr.max+1)
		*tr.root.children = append([]*mapNode[K, V]{}, left, right)
		tr.root.items = append([]mapPair[K, V]{}, median)
		tr.root.updateCount()
		return tr.Set(item.key, item.value)
	}
	if replaced {
		return prev, true
	}
	tr.count++
	return tr.empty.value, false
}

func (tr *Map[K, V]) nodeSplit(n *mapNode[K, V],
) (right *mapNode[K, V], median mapPair[K, V]) {
	i := tr.max / 2
	median = n.items[i]

	// right node
	right = tr.newNode(n.leaf())
	right.items = n.items[i+1:]
	if !n.leaf() {
		*right.children = (*n.children)[i+1:]
	}
	right.updateCount()

	// left node
	n.items[i] = tr.empty
	n.items = n.items[:i:i]
	if !n.leaf() {
		*n.children = (*n.children)[: i+1 : i+1]
	}
	n.updateCount()
	return right, median
}

func (n *mapNode[K, V]) updateCount() {
	n.count = len(n.items)
	if !n.leaf() {
		for i := 0; i < len(*n.children); i++ {
			n.count += (*n.children)[i].count
		}
	}
}

func (tr *Map[K, V]) nodeSet(pn **mapNode[K, V], item mapPair[K, V],
) (prev V, replaced bool, split bool) {
	n := tr.isoLoad(pn, true)
	i, found := tr.search(n, item.key)
	if found {
		prev = n.items[i].value
		n.items[i] = item
		return prev, true, false
	}
	if n.leaf() {
		if len(n.items) == tr.max {
			return tr.empty.value, false, true
		}
		n.items = append(n.items, tr.empty)
		copy(n.items[i+1:], n.items[i:])
		n.items[i] = item
		n.count++
		return tr.empty.value, false, false
	}
	prev, replaced, split = tr.nodeSet(&(*n.children)[i], item)
	if split {
		if len(n.items) == tr.max {
			return tr.empty.value, false, true
		}
		right, median := tr.nodeSplit((*n.children)[i])
		*n.children = append(*n.children, nil)
		copy((*n.children)[i+1:], (*n.children)[i:])
		(*n.children)[i+1] = right
		n.items = append(n.items, tr.empty)
		copy(n.items[i+1:], n.items[i:])
		n.items[i] = median
		return tr.nodeSet(&n, item)
	}
	if !replaced {
		n.count++
	}
	return prev, replaced, false
}

func (tr *Map[K, V]) Scan(iter func(key K, value V) bool) {
	tr.scan(iter, false)
}

func (tr *Map[K, V]) ScanMut(iter func(key K, value V) bool) {
	tr.scan(iter, true)
}

func (tr *Map[K, V]) scan(iter func(key K, value V) bool, mut bool) {
	if tr.root == nil {
		return
	}
	tr.nodeScan(&tr.root, iter, mut)
}

func (tr *Map[K, V]) nodeScan(cn **mapNode[K, V],
	iter func(key K, value V) bool, mut bool,
) bool {
	n := tr.isoLoad(cn, mut)
	if n.leaf() {
		for i := 0; i < len(n.items); i++ {
			if !iter(n.items[i].key, n.items[i].value) {
				return false
			}
		}
		return true
	}
	for i := 0; i < len(n.items); i++ {
		if !tr.nodeScan(&(*n.children)[i], iter, mut) {
			return false
		}
		if !iter(n.items[i].key, n.items[i].value) {
			return false
		}
	}
	return tr.nodeScan(&(*n.children)[len(*n.children)-1], iter, mut)
}

// Get a value for key.
func (tr *Map[K, V]) Get(key K) (V, bool) {
	return tr.get(key, false)
}

// GetMut gets a value for key.
// If needed, this may perform a copy the resulting value before returning.
//
// Mut methods are only useful when all of the following are true:
//   - The interior data of the value requires changes.
//   - The value is a pointer type.
//   - The BTree has been copied using `Copy()` or `IsoCopy()`.
//   - The value itself has a `Copy()` or `IsoCopy()` method.
//
// Mut methods may modify the tree structure and should have the same
// considerations as other mutable operations like Set, Delete, Clear, etc.
func (tr *Map[K, V]) GetMut(key K) (V, bool) {
	return tr.get(key, true)
}

func (tr *Map[K, V]) get(key K, mut bool) (V, bool) {
	if tr.root == nil {
		return tr.empty.value, false
	}
	n := tr.isoLoad(&tr.root, mut)
	for {
		i, found := tr.search(n, key)
		if found {
			return n.items[i].value, true
		}
		if n.leaf() {
			return tr.empty.value, false
		}
		n = tr.isoLoad(&(*n.children)[i], mut)
	}
}

// Len returns the number of items in the tree
func (tr *Map[K, V]) Len() int {
	return tr.count
}

// Delete a value for a key and returns the deleted value.
// Returns false if there was no value by that key found.
func (tr *Map[K, V]) Delete(key K) (V, bool) {
	if tr.root == nil {
		return tr.empty.value, false
	}
	prev, deleted := tr.delete(&tr.root, false, key)
	if !deleted {
		return tr.empty.value, false
	}
	if len(tr.root.items) == 0 && !tr.root.leaf() {
		tr.root = (*tr.root.children)[0]
	}
	tr.count--
	if tr.count == 0 {
		tr.root = nil
	}
	return prev.value, true
}

func (tr *Map[K, V]) delete(pn **mapNode[K, V], max bool, key K,
) (mapPair[K, V], bool) {
	n := tr.isoLoad(pn, true)
	var i int
	var found bool
	if max {
		i, found = len(n.items)-1, true
	} else {
		i, found = tr.search(n, key)
	}
	if n.leaf() {
		if found {
			// found the items at the leaf, remove it and return.
			prev := n.items[i]
			copy(n.items[i:], n.items[i+1:])
			n.items[len(n.items)-1] = tr.empty
			n.items = n.items[:len(n.items)-1]
			n.count--
			return prev, true
		}
		return tr.empty, false
	}

	var prev mapPair[K, V]
	var deleted bool
	if found {
		if max {
			i++
			prev, deleted = tr.delete(&(*n.children)[i], true, tr.empty.key)
		} else {
			prev = n.items[i]
			maxItem, _ := tr.delete(&(*n.children)[i], true, tr.empty.key)
			deleted = true
			n.items[i] = maxItem
		}
	} else {
		prev, deleted = tr.delete(&(*n.children)[i], max, key)
	}
	if !deleted {
		return tr.empty, false
	}
	n.count--
	if len((*n.children)[i].items) < tr.min {
		tr.nodeRebalance(n, i)
	}
	return prev, true
}

// nodeRebalance rebalances the child nodes following a delete operation.
// Provide the index of the child node with the number of items that fell
// below minItems.
func (tr *Map[K, V]) nodeRebalance(n *mapNode[K, V], i int) {
	if i == len(n.items) {
		i--
	}

	// ensure copy-on-write
	left := tr.isoLoad(&(*n.children)[i], true)
	right := tr.isoLoad(&(*n.children)[i+1], true)

	if len(left.items)+len(right.items) < tr.max {
		// Merges the left and right children nodes together as a single node
		// that includes (left,item,right), and places the contents into the
		// existing left node. Delete the right node altogether and move the
		// following items and child nodes to the left by one slot.

		// merge (left,item,right)
		left.items = append(left.items, n.items[i])
		left.items = append(left.items, right.items...)
		if !left.leaf() {
			*left.children = append(*left.children, *right.children...)
		}
		left.count += right.count + 1

		// move the items over one slot
		copy(n.items[i:], n.items[i+1:])
		n.items[len(n.items)-1] = tr.empty
		n.items = n.items[:len(n.items)-1]

		// move the children over one slot
		copy((*n.children)[i+1:], (*n.children)[i+2:])
		(*n.children)[len(*n.children)-1] = nil
		(*n.children) = (*n.children)[:len(*n.children)-1]
	} else if len(left.items) > len(right.items) {
		// move left -> right over one slot

		// Move the item of the parent node at index into the right-node first
		// slot, and move the left-node last item into the previously moved
		// parent item slot.
		right.items = append(right.items, tr.empty)
		copy(right.items[1:], right.items)
		right.items[0] = n.items[i]
		right.count++
		n.items[i] = left.items[len(left.items)-1]
		left.items[len(left.items)-1] = tr.empty
		left.items = left.items[:len(left.items)-1]
		left.count--

		if !left.leaf() {
			// move the left-node last child into the right-node first slot
			*right.children = append(*right.children, nil)
			copy((*right.children)[1:], *right.children)
			(*right.children)[0] = (*left.children)[len(*left.children)-1]
			(*left.children)[len(*left.children)-1] = nil
			(*left.children) = (*left.children)[:len(*left.children)-1]
			left.count -= (*right.children)[0].count
			right.count += (*right.children)[0].count
		}
	} else {
		// move left <- right over one slot

		// Same as above but the other direction
		left.items = append(left.items, n.items[i])
		left.count++
		n.items[i] = right.items[0]
		copy(right.items, right.items[1:])
		right.items[len(right.items)-1] = tr.empty
		right.items = right.items[:len(right.items)-1]
		right.count--

		if !left.leaf() {
			*left.children = append(*left.children, (*right.children)[0])
			copy(*right.children, (*right.children)[1:])
			(*right.children)[len(*right.children)-1] = nil
			*right.children = (*right.children)[:len(*right.children)-1]
			left.count += (*left.children)[len(*left.children)-1].count
			right.count -= (*left.children)[len(*left.children)-1].count
		}
	}
}

// Ascend the tree within the range [pivot, last]
// Pass nil for pivot to scan all item in ascending order
// Return false to stop iterating
func (tr *Map[K, V]) Ascend(pivot K, iter func(key K, value V) bool) {
	tr.ascend(pivot, iter, false)
}

func (tr *Map[K, V]) AscendMut(pivot K, iter func(key K, value V) bool) {
	tr.ascend(pivot, iter, true)
}

func (tr *Map[K, V]) ascend(pivot K, iter func(key K, value V) bool, mut bool) {
	if tr.root == nil {
		return
	}
	tr.nodeAscend(&tr.root, pivot, iter, mut)
}

// The return value of this function determines whether we should keep iterating
// upon this functions return.
func (tr *Map[K, V]) nodeAscend(cn **mapNode[K, V], pivot K,
	iter func(key K, value V) bool, mut bool,
) bool {
	n := tr.isoLoad(cn, mut)
	i, found := tr.search(n, pivot)
	if !found {
		if !n.leaf() {
			if !tr.nodeAscend(&(*n.children)[i], pivot, iter, mut) {
				return false
			}
		}
	}
	// We are either in the case that
	// - node is found, we should iterate through it starting at `i`,
	//   the index it was located at.
	// - node is not found, and TODO: fill in.
	for ; i < len(n.items); i++ {
		if !iter(n.items[i].key, n.items[i].value) {
			return false
		}
		if !n.leaf() {
			if !tr.nodeScan(&(*n.children)[i+1], iter, mut) {
				return false
			}
		}
	}
	return true
}

func (tr *Map[K, V]) Reverse(iter func(key K, value V) bool) {
	tr.reverse(iter, false)
}

func (tr *Map[K, V]) ReverseMut(iter func(key K, value V) bool) {
	tr.reverse(iter, true)
}

func (tr *Map[K, V]) reverse(iter func(key K, value V) bool, mut bool) {
	if tr.root == nil {
		return
	}
	tr.nodeReverse(&tr.root, iter, mut)
}

func (tr *Map[K, V]) nodeReverse(cn **mapNode[K, V],
	iter func(key K, value V) bool, mut bool,
) bool {
	n := tr.isoLoad(cn, mut)
	if n.leaf() {
		for i := len(n.items) - 1; i >= 0; i-- {
			if !iter(n.items[i].key, n.items[i].value) {
				return false
			}
		}
		return true
	}
	if !tr.nodeReverse(&(*n.children)[len(*n.children)-1], iter, mut) {
		return false
	}
	for i := len(n.items) - 1; i >= 0; i-- {
		if !iter(n.items[i].key, n.items[i].value) {
			return false
		}
		if !tr.nodeReverse(&(*n.children)[i], iter, mut) {
			return false
		}
	}
	return true
}

// Descend the tree within the range [pivot, first]
// Pass nil for pivot to scan all item in descending order
// Return false to stop iterating
func (tr *Map[K, V]) Descend(pivot K, iter func(key K, value V) bool) {
	tr.descend(pivot, iter, false)
}

func (tr *Map[K, V]) DescendMut(pivot K, iter func(key K, value V) bool) {
	tr.descend(pivot, iter, true)
}

func (tr *Map[K, V]) descend(
	pivot K,
	iter func(key K, value V) bool,
	mut bool,
) {
	if tr.root == nil {
		return
	}
	tr.nodeDescend(&tr.root, pivot, iter, mut)
}

func (tr *Map[K, V]) nodeDescend(cn **mapNode[K, V], pivot K,
	iter func(key K, value V) bool, mut bool,
) bool {
	n := tr.isoLoad(cn, mut)
	i, found := tr.search(n, pivot)
	if !found {
		if !n.leaf() {
			if !tr.nodeDescend(&(*n.children)[i], pivot, iter, mut) {
				return false
			}
		}
		i--
	}
	for ; i >= 0; i-- {
		if !iter(n.items[i].key, n.items[i].value) {
			return false
		}
		if !n.leaf() {
			if !tr.nodeReverse(&(*n.children)[i], iter, mut) {
				return false
			}
		}
	}
	return true
}

// Load is for bulk loading pre-sorted items
func (tr *Map[K, V]) Load(key K, value V) (V, bool) {
	item := mapPair[K, V]{key: key, value: value}
	if tr.root == nil {
		return tr.Set(item.key, item.value)
	}
	n := tr.isoLoad(&tr.root, true)
	for {
		n.count++ // optimistically update counts
		if n.leaf() {
			if len(n.items) < tr.max {
				if n.items[len(n.items)-1].key < item.key {
					n.items = append(n.items, item)
					tr.count++
					return tr.empty.value, false
				}
			}
			break
		}
		n = tr.isoLoad(&(*n.children)[len(*n.children)-1], true)
	}
	// revert the counts
	n = tr.root
	for {
		n.count--
		if n.leaf() {
			break
		}
		n = (*n.children)[len(*n.children)-1]
	}
	return tr.Set(item.key, item.value)
}

// Min returns the minimum item in tree.
// Returns nil if the treex has no items.
func (tr *Map[K, V]) Min() (K, V, bool) {
	return tr.minMut(false)
}

func (tr *Map[K, V]) MinMut() (K, V, bool) {
	return tr.minMut(true)
}

func (tr *Map[K, V]) minMut(mut bool) (key K, value V, ok bool) {
	if tr.root == nil {
		return key, value, false
	}
	n := tr.isoLoad(&tr.root, mut)
	for {
		if n.leaf() {
			item := n.items[0]
			return item.key, item.value, true
		}
		n = tr.isoLoad(&(*n.children)[0], mut)
	}
}

// Max returns the maximum item in tree.
// Returns nil if the tree has no items.
func (tr *Map[K, V]) Max() (K, V, bool) {
	return tr.maxMut(false)
}

func (tr *Map[K, V]) MaxMut() (K, V, bool) {
	return tr.maxMut(true)
}

func (tr *Map[K, V]) maxMut(mut bool) (K, V, bool) {
	if tr.root == nil {
		return tr.empty.key, tr.empty.value, false
	}
	n := tr.isoLoad(&tr.root, mut)
	for {
		if n.leaf() {
			item := n.items[len(n.items)-1]
			return item.key, item.value, true
		}
		n = tr.isoLoad(&(*n.children)[len(*n.children)-1], mut)
	}
}

// PopMin removes the minimum item in tree and returns it.
// Returns nil if the tree has no items.
func (tr *Map[K, V]) PopMin() (K, V, bool) {
	if tr.root == nil {
		return tr.empty.key, tr.empty.value, false
	}
	n := tr.isoLoad(&tr.root, true)
	var item mapPair[K, V]
	for {
		n.count-- // optimistically update counts
		if n.leaf() {
			item = n.items[0]
			if len(n.items) == tr.min {
				break
			}
			copy(n.items[:], n.items[1:])
			n.items[len(n.items)-1] = tr.empty
			n.items = n.items[:len(n.items)-1]
			tr.count--
			if tr.count == 0 {
				tr.root = nil
			}
			return item.key, item.value, true
		}
		n = tr.isoLoad(&(*n.children)[0], true)
	}
	// revert the counts
	n = tr.root
	for {
		n.count++
		if n.leaf() {
			break
		}
		n = (*n.children)[0]
	}
	value, deleted := tr.Delete(item.key)
	if deleted {
		return item.key, value, true
	}
	return tr.empty.key, tr.empty.value, false
}

// PopMax removes the maximum item in tree and returns it.
// Returns nil if the tree has no items.
func (tr *Map[K, V]) PopMax() (K, V, bool) {
	if tr.root == nil {
		return tr.empty.key, tr.empty.value, false
	}
	n := tr.isoLoad(&tr.root, true)
	var item mapPair[K, V]
	for {
		n.count-- // optimistically update counts
		if n.leaf() {
			item = n.items[len(n.items)-1]
			if len(n.items) == tr.min {
				break
			}
			n.items[len(n.items)-1] = tr.empty
			n.items = n.items[:len(n.items)-1]
			tr.count--
			if tr.count == 0 {
				tr.root = nil
			}
			return item.key, item.value, true
		}
		n = tr.isoLoad(&(*n.children)[len(*n.children)-1], true)
	}
	// revert the counts
	n = tr.root
	for {
		n.count++
		if n.leaf() {
			break
		}
		n = (*n.children)[len(*n.children)-1]
	}
	value, deleted := tr.Delete(item.key)
	if deleted {
		return item.key, value, true
	}
	return tr.empty.key, tr.empty.value, false
}

// GetAt returns the value at index.
// Return nil if the tree is empty or the index is out of bounds.
func (tr *Map[K, V]) GetAt(index int) (K, V, bool) {
	return tr.getAt(index, false)
}

func (tr *Map[K, V]) GetAtMut(index int) (K, V, bool) {
	return tr.getAt(index, true)
}

func (tr *Map[K, V]) getAt(index int, mut bool) (K, V, bool) {
	if tr.root == nil || index < 0 || index >= tr.count {
		return tr.empty.key, tr.empty.value, false
	}
	n := tr.isoLoad(&tr.root, mut)
	for {
		if n.leaf() {
			return n.items[index].key, n.items[index].value, true
		}
		i := 0
		for ; i < len(n.items); i++ {
			if index < (*n.children)[i].count {
				break
			} else if index == (*n.children)[i].count {
				return n.items[i].key, n.items[i].value, true
			}
			index -= (*n.children)[i].count + 1
		}
		n = tr.isoLoad(&(*n.children)[i], mut)
	}
}

// DeleteAt deletes the item at index.
// Return nil if the tree is empty or the index is out of bounds.
func (tr *Map[K, V]) DeleteAt(index int) (K, V, bool) {
	if tr.root == nil || index < 0 || index >= tr.count {
		return tr.empty.key, tr.empty.value, false
	}
	var pathbuf [8]uint8 // track the path
	path := pathbuf[:0]
	var item mapPair[K, V]
	n := tr.isoLoad(&tr.root, true)
outer:
	for {
		n.count-- // optimistically update counts
		if n.leaf() {
			// the index is the item position
			item = n.items[index]
			if len(n.items) == tr.min {
				path = append(path, uint8(index))
				break outer
			}
			copy(n.items[index:], n.items[index+1:])
			n.items[len(n.items)-1] = tr.empty
			n.items = n.items[:len(n.items)-1]
			tr.count--
			if tr.count == 0 {
				tr.root = nil
			}
			return item.key, item.value, true
		}
		i := 0
		for ; i < len(n.items); i++ {
			if index < (*n.children)[i].count {
				break
			} else if index == (*n.children)[i].count {
				item = n.items[i]
				path = append(path, uint8(i))
				break outer
			}
			index -= (*n.children)[i].count + 1
		}
		path = append(path, uint8(i))
		n = tr.isoLoad(&(*n.children)[i], true)
	}
	// revert the counts
	n = tr.root
	for i := 0; i < len(path); i++ {
		n.count++
		if !n.leaf() {
			n = (*n.children)[uint8(path[i])]
		}
	}
	value, deleted := tr.Delete(item.key)
	if deleted {
		return item.key, value, true
	}
	return tr.empty.key, tr.empty.value, false
}

// Height returns the height of the tree.
// Returns zero if tree has no items.
func (tr *Map[K, V]) Height() int {
	var height int
	if tr.root != nil {
		n := tr.root
		for {
			height++
			if n.leaf() {
				break
			}
			n = (*n.children)[0]
		}
	}
	return height
}

// MapIter represents an iterator for btree.Map
type MapIter[K ordered, V any] struct {
	tr      *Map[K, V]
	mut     bool
	seeked  bool
	atstart bool
	atend   bool
	stack   []mapIterStackItem[K, V]
	item    mapPair[K, V]
}

type mapIterStackItem[K ordered, V any] struct {
	n *mapNode[K, V]
	i int
}

// Iter returns a read-only iterator.
func (tr *Map[K, V]) Iter() MapIter[K, V] {
	return tr.iter(false)
}

func (tr *Map[K, V]) IterMut() MapIter[K, V] {
	return tr.iter(true)
}

func (tr *Map[K, V]) iter(mut bool) MapIter[K, V] {
	var iter MapIter[K, V]
	iter.tr = tr
	iter.mut = mut
	return iter
}

// Seek to item greater-or-equal-to key.
// Returns false if there was no item found.
func (iter *MapIter[K, V]) Seek(key K) bool {
	if iter.tr == nil {
		return false
	}
	iter.seeked = true
	iter.stack = iter.stack[:0]
	if iter.tr.root == nil {
		return false
	}
	n := iter.tr.isoLoad(&iter.tr.root, iter.mut)
	for {
		i, found := iter.tr.search(n, key)
		iter.stack = append(iter.stack, mapIterStackItem[K, V]{n, i})
		if found {
			iter.item = n.items[i]
			return true
		}
		if n.leaf() {
			iter.stack[len(iter.stack)-1].i--
			return iter.Next()
		}
		n = iter.tr.isoLoad(&(*n.children)[i], iter.mut)
	}
}

// First moves iterator to first item in tree.
// Returns false if the tree is empty.
func (iter *MapIter[K, V]) First() bool {
	if iter.tr == nil {
		return false
	}
	iter.atend = false
	iter.atstart = false
	iter.seeked = true
	iter.stack = iter.stack[:0]
	if iter.tr.root == nil {
		return false
	}
	n := iter.tr.isoLoad(&iter.tr.root, iter.mut)
	for {
		iter.stack = append(iter.stack, mapIterStackItem[K, V]{n, 0})
		if n.leaf() {
			break
		}
		n = iter.tr.isoLoad(&(*n.children)[0], iter.mut)
	}
	s := &iter.stack[len(iter.stack)-1]
	iter.item = s.n.items[s.i]
	return true
}

// Last moves iterator to last item in tree.
// Returns false if the tree is empty.
func (iter *MapIter[K, V]) Last() bool {
	if iter.tr == nil {
		return false
	}
	iter.seeked = true
	iter.stack = iter.stack[:0]
	if iter.tr.root == nil {
		return false
	}
	n := iter.tr.isoLoad(&iter.tr.root, iter.mut)
	for {
		iter.stack = append(iter.stack, mapIterStackItem[K, V]{n, len(n.items)})
		if n.leaf() {
			iter.stack[len(iter.stack)-1].i--
			break
		}
		n = iter.tr.isoLoad(&(*n.children)[len(n.items)], iter.mut)
	}
	s := &iter.stack[len(iter.stack)-1]
	iter.item = s.n.items[s.i]
	return true
}

// Next moves iterator to the next item in iterator.
// Returns false if the tree is empty or the iterator is at the end of
// the tree.
func (iter *MapIter[K, V]) Next() bool {
	if iter.tr == nil {
		return false
	}
	if !iter.seeked {
		return iter.First()
	}
	if len(iter.stack) == 0 {
		if iter.atstart {
			return iter.First() && iter.Next()
		}
		return false
	}
	s := &iter.stack[len(iter.stack)-1]
	s.i++
	if s.n.leaf() {
		if s.i == len(s.n.items) {
			for {
				iter.stack = iter.stack[:len(iter.stack)-1]
				if len(iter.stack) == 0 {
					iter.atend = true
					return false
				}
				s = &iter.stack[len(iter.stack)-1]
				if s.i < len(s.n.items) {
					break
				}
			}
		}
	} else {
		n := iter.tr.isoLoad(&(*s.n.children)[s.i], iter.mut)
		for {
			iter.stack = append(iter.stack, mapIterStackItem[K, V]{n, 0})
			if n.leaf() {
				break
			}
			n = iter.tr.isoLoad(&(*n.children)[0], iter.mut)
		}
	}
	s = &iter.stack[len(iter.stack)-1]
	iter.item = s.n.items[s.i]
	return true
}

// Prev moves iterator to the previous item in iterator.
// Returns false if the tree is empty or the iterator is at the beginning of
// the tree.
func (iter *MapIter[K, V]) Prev() bool {
	if iter.tr == nil {
		return false
	}
	if !iter.seeked {
		return false
	}
	if len(iter.stack) == 0 {
		if iter.atend {
			return iter.Last() && iter.Prev()
		}
		return false
	}
	s := &iter.stack[len(iter.stack)-1]
	if s.n.leaf() {
		s.i--
		if s.i == -1 {
			for {
				iter.stack = iter.stack[:len(iter.stack)-1]
				if len(iter.stack) == 0 {
					iter.atstart = true
					return false
				}
				s = &iter.stack[len(iter.stack)-1]
				s.i--
				if s.i > -1 {
					break
				}
			}
		}
	} else {
		n := iter.tr.isoLoad(&(*s.n.children)[s.i], iter.mut)
		for {
			iter.stack = append(iter.stack,
				mapIterStackItem[K, V]{n, len(n.items)})
			if n.leaf() {
				iter.stack[len(iter.stack)-1].i--
				break
			}
			n = iter.tr.isoLoad(&(*n.children)[len(n.items)], iter.mut)
		}
	}
	s = &iter.stack[len(iter.stack)-1]
	iter.item = s.n.items[s.i]
	return true
}

// Key returns the current iterator item key.
func (iter *MapIter[K, V]) Key() K {
	return iter.item.key
}

// Value returns the current iterator item value.
func (iter *MapIter[K, V]) Value() V {
	return iter.item.value
}

// Values returns all the values in order.
func (tr *Map[K, V]) Values() []V {
	return tr.values(false)
}

func (tr *Map[K, V]) ValuesMut() []V {
	return tr.values(true)
}

func (tr *Map[K, V]) values(mut bool) []V {
	values := make([]V, 0, tr.Len())
	if tr.root != nil {
		values = tr.nodeValues(&tr.root, values, mut)
	}
	return values
}

func (tr *Map[K, V]) nodeValues(cn **mapNode[K, V], values []V, mut bool) []V {
	n := tr.isoLoad(cn, mut)
	if n.leaf() {
		for i := 0; i < len(n.items); i++ {
			values = append(values, n.items[i].value)
		}
		return values
	}
	for i := 0; i < len(n.items); i++ {
		values = tr.nodeValues(&(*n.children)[i], values, mut)
		values = append(values, n.items[i].value)
	}
	return tr.nodeValues(&(*n.children)[len(*n.children)-1], values, mut)
}

// Keys returns all the keys in order.
func (tr *Map[K, V]) Keys() []K {
	keys := make([]K, 0, tr.Len())
	if tr.root != nil {
		keys = tr.root.keys(keys)
	}
	return keys
}

func (n *mapNode[K, V]) keys(keys []K) []K {
	if n.leaf() {
		for i := 0; i < len(n.items); i++ {
			keys = append(keys, n.items[i].key)
		}
		return keys
	}
	for i := 0; i < len(n.items); i++ {
		keys = (*n.children)[i].keys(keys)
		keys = append(keys, n.items[i].key)
	}
	return (*n.children)[len(*n.children)-1].keys(keys)
}

// KeyValues returns all the keys and values in order.
func (tr *Map[K, V]) KeyValues() ([]K, []V) {
	return tr.keyValues(false)
}

func (tr *Map[K, V]) KeyValuesMut() ([]K, []V) {
	return tr.keyValues(true)
}

func (tr *Map[K, V]) keyValues(mut bool) ([]K, []V) {
	keys := make([]K, 0, tr.Len())
	values := make([]V, 0, tr.Len())
	if tr.root != nil {
		keys, values = tr.nodeKeyValues(&tr.root, keys, values, mut)
	}
	return keys, values
}

func (tr *Map[K, V]) nodeKeyValues(cn **mapNode[K, V], keys []K, values []V,
	mut bool,
) ([]K, []V) {
	n := tr.isoLoad(cn, mut)
	if n.leaf() {
		for i := 0; i < len(n.items); i++ {
			keys = append(keys, n.items[i].key)
			values = append(values, n.items[i].value)
		}
		return keys, values
	}
	for i := 0; i < len(n.items); i++ {
		keys, values = tr.nodeKeyValues(&(*n.children)[i], keys, values, mut)
		keys = append(keys, n.items[i].key)
		values = append(values, n.items[i].value)
	}
	return tr.nodeKeyValues(&(*n.children)[len(*n.children)-1], keys, values,
		mut)
}

// Clear will delete all items.
func (tr *Map[K, V]) Clear() {
	tr.count = 0
	tr.root = nil
}
