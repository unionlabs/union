package btree

type Set[K ordered] struct {
	base Map[K, struct{}]
}

// Copy
func (tr *Set[K]) Copy() *Set[K] {
	tr2 := new(Set[K])
	tr2.base = *tr.base.Copy()
	return tr2
}

func (tr *Set[K]) IsoCopy() *Set[K] {
	tr2 := new(Set[K])
	tr2.base = *tr.base.IsoCopy()
	return tr2
}

// Insert an item
func (tr *Set[K]) Insert(key K) {
	tr.base.Set(key, struct{}{})
}

func (tr *Set[K]) Scan(iter func(key K) bool) {
	tr.base.Scan(func(key K, value struct{}) bool {
		return iter(key)
	})
}

// Get a value for key
func (tr *Set[K]) Contains(key K) bool {
	_, ok := tr.base.Get(key)
	return ok
}

// Len returns the number of items in the tree
func (tr *Set[K]) Len() int {
	return tr.base.Len()
}

// Delete an item
func (tr *Set[K]) Delete(key K) {
	tr.base.Delete(key)
}

// Ascend the tree within the range [pivot, last]
// Pass nil for pivot to scan all item in ascending order
// Return false to stop iterating
func (tr *Set[K]) Ascend(pivot K, iter func(key K) bool) {
	tr.base.Ascend(pivot, func(key K, value struct{}) bool {
		return iter(key)
	})
}

func (tr *Set[K]) Reverse(iter func(key K) bool) {
	tr.base.Reverse(func(key K, value struct{}) bool {
		return iter(key)
	})
}

// Descend the tree within the range [pivot, first]
// Pass nil for pivot to scan all item in descending order
// Return false to stop iterating
func (tr *Set[K]) Descend(pivot K, iter func(key K) bool) {
	tr.base.Descend(pivot, func(key K, value struct{}) bool {
		return iter(key)
	})
}

// Load is for bulk loading pre-sorted items
func (tr *Set[K]) Load(key K) {
	tr.base.Load(key, struct{}{})
}

// Min returns the minimum item in tree.
// Returns nil if the treex has no items.
func (tr *Set[K]) Min() (K, bool) {
	key, _, ok := tr.base.Min()
	return key, ok
}

// Max returns the maximum item in tree.
// Returns nil if the tree has no items.
func (tr *Set[K]) Max() (K, bool) {
	key, _, ok := tr.base.Max()
	return key, ok
}

// PopMin removes the minimum item in tree and returns it.
// Returns nil if the tree has no items.
func (tr *Set[K]) PopMin() (K, bool) {
	key, _, ok := tr.base.PopMin()
	return key, ok
}

// PopMax removes the maximum item in tree and returns it.
// Returns nil if the tree has no items.
func (tr *Set[K]) PopMax() (K, bool) {
	key, _, ok := tr.base.PopMax()
	return key, ok
}

// GetAt returns the value at index.
// Return nil if the tree is empty or the index is out of bounds.
func (tr *Set[K]) GetAt(index int) (K, bool) {
	key, _, ok := tr.base.GetAt(index)
	return key, ok
}

// DeleteAt deletes the item at index.
// Return nil if the tree is empty or the index is out of bounds.
func (tr *Set[K]) DeleteAt(index int) (K, bool) {
	key, _, ok := tr.base.DeleteAt(index)
	return key, ok
}

// Height returns the height of the tree.
// Returns zero if tree has no items.
func (tr *Set[K]) Height() int {
	return tr.base.Height()
}

// SetIter represents an iterator for btree.Set
type SetIter[K ordered] struct {
	base MapIter[K, struct{}]
}

// Iter returns a read-only iterator.
func (tr *Set[K]) Iter() SetIter[K] {
	return SetIter[K]{tr.base.Iter()}
}

// Seek to item greater-or-equal-to key.
// Returns false if there was no item found.
func (iter *SetIter[K]) Seek(key K) bool {
	return iter.base.Seek(key)
}

// First moves iterator to first item in tree.
// Returns false if the tree is empty.
func (iter *SetIter[K]) First() bool {
	return iter.base.First()
}

// Last moves iterator to last item in tree.
// Returns false if the tree is empty.
func (iter *SetIter[K]) Last() bool {
	return iter.base.Last()
}

// Next moves iterator to the next item in iterator.
// Returns false if the tree is empty or the iterator is at the end of
// the tree.
func (iter *SetIter[K]) Next() bool {
	return iter.base.Next()
}

// Prev moves iterator to the previous item in iterator.
// Returns false if the tree is empty or the iterator is at the beginning of
// the tree.
func (iter *SetIter[K]) Prev() bool {
	return iter.base.Prev()
}

// Key returns the current iterator item key.
func (iter *SetIter[K]) Key() K {
	return iter.base.Key()
}

// Keys returns all the keys in order.
func (tr *Set[K]) Keys() []K {
	return tr.base.Keys()
}

// Clear will delete all items.
func (tr *Set[K]) Clear() {
	tr.base.Clear()
}
