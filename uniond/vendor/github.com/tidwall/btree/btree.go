// Copyright 2020 Joshua J Baker. All rights reserved.
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file.
package btree

type BTree struct {
	base *BTreeG[any]
}

// New returns a new BTree
func New(less func(a, b any) bool) *BTree {
	if less == nil {
		panic("nil less")
	}
	return &BTree{base: NewBTreeG(less)}
}

// NewNonConcurrent returns a new BTree which is not safe for concurrent
// write operations by multiple goroutines.
//
// This is useful for when you do not need the BTree to manage the locking,
// but would rather do it yourself.
//
// Deprecated: use NewOptions
func NewNonConcurrent(less func(a, b any) bool) *BTree {
	if less == nil {
		panic("nil less")
	}
	return &BTree{base: NewBTreeGOptions(less, Options{NoLocks: true})}
}

// NewOptions returns a new BTree
func NewOptions(less func(a, b any) bool, opts Options) *BTree {
	if less == nil {
		panic("nil less")
	}
	return &BTree{base: NewBTreeGOptions(less, opts)}
}

// Less is a convenience function that performs a comparison of two items
// using the same "less" function provided to New.
func (tr *BTree) Less(a, b any) bool {
	return tr.base.Less(a, b)
}

// Set or replace a value for a key
// Returns the value for the replaced item or nil if the key was not found.
func (tr *BTree) Set(item any) (prev any) {
	return tr.SetHint(item, nil)
}

// SetHint sets or replace a value for a key using a path hint
// Returns the value for the replaced item or nil if the key was not found.
func (tr *BTree) SetHint(item any, hint *PathHint) (prev any) {
	if item == nil {
		panic("nil item")
	}
	v, ok := tr.base.SetHint(item, hint)
	if !ok {
		return nil
	}
	return v
}

// Get a value for key.
// Returns nil if the key was not found.
func (tr *BTree) Get(key any) any {
	return tr.getHintMut(key, nil, false)
}

func (tr *BTree) GetMut(key any) any {
	return tr.getHintMut(key, nil, true)
}

func (tr *BTree) GetHint(key any, hint *PathHint) any {
	return tr.getHintMut(key, hint, false)
}

func (tr *BTree) GetHintMut(key any, hint *PathHint) any {
	return tr.getHintMut(key, hint, true)
}

// GetHint gets a value for key using a path hint.
// Returns nil if the item was not found.
func (tr *BTree) getHintMut(key any, hint *PathHint, mut bool) (value any) {
	if key == nil {
		return nil
	}
	var v any
	var ok bool
	if mut {
		v, ok = tr.base.GetHintMut(key, hint)
	} else {
		v, ok = tr.base.GetHint(key, hint)
	}
	if !ok {
		return nil
	}
	return v
}

// Len returns the number of items in the tree
func (tr *BTree) Len() int {
	return tr.base.Len()
}

// Delete an item for a key.
// Returns the deleted value or nil if the key was not found.
func (tr *BTree) Delete(key any) (prev any) {
	return tr.DeleteHint(key, nil)
}

// DeleteHint deletes a value for a key using a path hint
// Returns the deleted value or nil if the key was not found.
func (tr *BTree) DeleteHint(key any, hint *PathHint) (prev any) {
	if key == nil {
		return nil
	}
	v, ok := tr.base.DeleteHint(key, nil)
	if !ok {
		return nil
	}
	return v
}

// Ascend the tree within the range [pivot, last]
// Pass nil for pivot to scan all item in ascending order
// Return false to stop iterating
func (tr *BTree) Ascend(pivot any, iter func(item any) bool) {
	if pivot == nil {
		tr.base.Scan(iter)
	} else {
		tr.base.Ascend(pivot, iter)
	}
}

func (tr *BTree) AscendMut(pivot any, iter func(item any) bool) {
	if pivot == nil {
		tr.base.ScanMut(iter)
	} else {
		tr.base.AscendMut(pivot, iter)
	}
}

func (tr *BTree) AscendHint(pivot any, iter func(item any) bool,
	hint *PathHint,
) {
	if pivot == nil {
		tr.base.Scan(iter)
	} else {
		tr.base.AscendHint(pivot, iter, hint)
	}
}

func (tr *BTree) AscendHintMut(pivot any, iter func(item any) bool,
	hint *PathHint,
) {
	if pivot == nil {
		tr.base.ScanMut(iter)
	} else {
		tr.base.AscendHintMut(pivot, iter, hint)
	}
}

// Descend the tree within the range [pivot, first]
// Pass nil for pivot to scan all item in descending order
// Return false to stop iterating
func (tr *BTree) Descend(pivot any, iter func(item any) bool) {
	if pivot == nil {
		tr.base.Reverse(iter)
	} else {
		tr.base.Descend(pivot, iter)
	}
}

func (tr *BTree) DescendMut(pivot any, iter func(item any) bool) {
	if pivot == nil {
		tr.base.ReverseMut(iter)
	} else {
		tr.base.DescendMut(pivot, iter)
	}
}

func (tr *BTree) DescendHint(pivot any, iter func(item any) bool,
	hint *PathHint,
) {
	if pivot == nil {
		tr.base.Reverse(iter)
	} else {
		tr.base.DescendHint(pivot, iter, hint)
	}
}

func (tr *BTree) DescendHintMut(pivot any, iter func(item any) bool,
	hint *PathHint,
) {
	if pivot == nil {
		tr.base.ReverseMut(iter)
	} else {
		tr.base.DescendHintMut(pivot, iter, hint)
	}
}

// Load is for bulk loading pre-sorted items
// If the load replaces and existing item then the value for the replaced item
// is returned.
func (tr *BTree) Load(item any) (prev any) {
	if item == nil {
		panic("nil item")
	}
	v, ok := tr.base.Load(item)
	if !ok {
		return nil
	}
	return v
}

// Min returns the minimum item in tree.
// Returns nil if the tree has no items.
func (tr *BTree) Min() any {
	v, ok := tr.base.Min()
	if !ok {
		return nil
	}
	return v
}

func (tr *BTree) MinMut() any {
	v, ok := tr.base.MinMut()
	if !ok {
		return nil
	}
	return v
}

// Max returns the maximum item in tree.
// Returns nil if the tree has no items.
func (tr *BTree) Max() any {
	v, ok := tr.base.Max()
	if !ok {
		return nil
	}
	return v
}

func (tr *BTree) MaxMut() any {
	v, ok := tr.base.Max()
	if !ok {
		return nil
	}
	return v
}

// PopMin removes the minimum item in tree and returns it.
// Returns nil if the tree has no items.
func (tr *BTree) PopMin() any {
	v, ok := tr.base.PopMin()
	if !ok {
		return nil
	}
	return v
}

// PopMax removes the maximum item in tree and returns it.
// Returns nil if the tree has no items.
func (tr *BTree) PopMax() any {
	v, ok := tr.base.PopMax()
	if !ok {
		return nil
	}
	return v
}

// GetAt returns the value at index.
// Return nil if the tree is empty or the index is out of bounds.
func (tr *BTree) GetAt(index int) any {
	v, ok := tr.base.GetAt(index)
	if !ok {
		return nil
	}
	return v
}

func (tr *BTree) GetAtMut(index int) any {
	v, ok := tr.base.GetAtMut(index)
	if !ok {
		return nil
	}
	return v
}

// DeleteAt deletes the item at index.
// Return nil if the tree is empty or the index is out of bounds.
func (tr *BTree) DeleteAt(index int) any {
	v, ok := tr.base.DeleteAt(index)
	if !ok {
		return nil
	}
	return v
}

// Height returns the height of the tree.
// Returns zero if tree has no items.
func (tr *BTree) Height() int {
	return tr.base.Height()
}

// Walk iterates over all items in tree, in order.
// The items param will contain one or more items.
func (tr *BTree) Walk(iter func(items []any)) {
	tr.base.Walk(func(items []any) bool {
		iter(items)
		return true
	})
}

func (tr *BTree) WalkMut(iter func(items []any)) {
	tr.base.WalkMut(func(items []any) bool {
		iter(items)
		return true
	})
}

// Copy the tree. This is a copy-on-write operation and is very fast because
// it only performs a shadowed copy.
func (tr *BTree) Copy() *BTree {
	return &BTree{base: tr.base.Copy()}
}

func (tr *BTree) IsoCopy() *BTree {
	return &BTree{base: tr.base.IsoCopy()}
}

// Clear will delete all items.
func (tr *BTree) Clear() {
	tr.base.Clear()
}

// Iter is an iterator for
type Iter struct {
	base IterG[any]
}

// Iter returns a read-only iterator.
// The Release method must be called finished with iterator.
func (tr *BTree) Iter() Iter {
	return Iter{tr.base.Iter()}
}

func (tr *BTree) IterMut() Iter {
	return Iter{tr.base.IterMut()}
}

// Seek to item greater-or-equal-to key.
// Returns false if there was no item found.
func (iter *Iter) Seek(key any) bool {
	return iter.base.Seek(key)
}

func (iter *Iter) SeekHint(key any, hint *PathHint) bool {
	return iter.base.SeekHint(key, hint)
}

// First moves iterator to first item in tree.
// Returns false if the tree is empty.
func (iter *Iter) First() bool {
	return iter.base.First()
}

// Last moves iterator to last item in tree.
// Returns false if the tree is empty.
func (iter *Iter) Last() bool {
	return iter.base.Last()
}

// First moves iterator to first item in tree.
// Returns false if the tree is empty.
func (iter *Iter) Release() {
	iter.base.Release()
}

// Next moves iterator to the next item in iterator.
// Returns false if the tree is empty or the iterator is at the end of
// the tree.
func (iter *Iter) Next() bool {
	return iter.base.Next()
}

// Prev moves iterator to the previous item in iterator.
// Returns false if the tree is empty or the iterator is at the beginning of
// the tree.
func (iter *Iter) Prev() bool {
	return iter.base.Prev()
}

// Item returns the current iterator item.
func (iter *Iter) Item() any {
	return iter.base.Item()
}
