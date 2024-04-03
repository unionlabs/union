// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package manifest

import (
	"bytes"
	"fmt"
	"strings"
	"sync/atomic"
	"unsafe"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/invariants"
)

// The Annotator type defined below is used by other packages to lazily
// compute a value over a B-Tree. Each node of the B-Tree stores one
// `annotation` per annotator, containing the result of the computation over
// the node's subtree.
//
// An annotation is marked as valid if it's current with the current subtree
// state. Annotations are marked as invalid whenever a node will be mutated
// (in mut).  Annotators may also return `false` from `Accumulate` to signal
// that a computation for a file is not stable and may change in the future.
// Annotations that include these unstable values are also marked as invalid
// on the node, ensuring that future queries for the annotation will recompute
// the value.

// An Annotator defines a computation over a level's FileMetadata. If the
// computation is stable and uses inputs that are fixed for the lifetime of
// a FileMetadata, the LevelMetadata's internal data structures are annotated
// with the intermediary computations. This allows the computation to be
// computed incrementally as edits are applied to a level.
type Annotator interface {
	// Zero returns the zero value of an annotation. This value is returned
	// when a LevelMetadata is empty. The dst argument, if non-nil, is an
	// obsolete value previously returned by this Annotator and may be
	// overwritten and reused to avoid a memory allocation.
	Zero(dst interface{}) (v interface{})

	// Accumulate computes the annotation for a single file in a level's
	// metadata. It merges the file's value into dst and returns a bool flag
	// indicating whether or not the value is stable and okay to cache as an
	// annotation. If the file's value may change over the life of the file,
	// the annotator must return false.
	//
	// Implementations may modify dst and return it to avoid an allocation.
	Accumulate(m *FileMetadata, dst interface{}) (v interface{}, cacheOK bool)

	// Merge combines two values src and dst, returning the result.
	// Implementations may modify dst and return it to avoid an allocation.
	Merge(src interface{}, dst interface{}) interface{}
}

type btreeCmp func(*FileMetadata, *FileMetadata) int

func btreeCmpSeqNum(a, b *FileMetadata) int {
	return a.cmpSeqNum(b)
}

func btreeCmpSmallestKey(cmp Compare) btreeCmp {
	return func(a, b *FileMetadata) int {
		return a.cmpSmallestKey(b, cmp)
	}
}

// btreeCmpSpecificOrder is used in tests to construct a B-Tree with a
// specific ordering of FileMetadata within the tree. It's typically used to
// test consistency checking code that needs to construct a malformed B-Tree.
func btreeCmpSpecificOrder(files []*FileMetadata) btreeCmp {
	m := map[*FileMetadata]int{}
	for i, f := range files {
		m[f] = i
	}
	return func(a, b *FileMetadata) int {
		ai, aok := m[a]
		bi, bok := m[b]
		if !aok || !bok {
			panic("btreeCmpSliceOrder called with unknown files")
		}
		switch {
		case ai < bi:
			return -1
		case ai > bi:
			return +1
		default:
			return 0
		}
	}
}

const (
	degree   = 16
	maxItems = 2*degree - 1
	minItems = degree - 1
)

type annotation struct {
	annotator Annotator
	// v is an annotation value, the output of either
	// annotator.Value or annotator.Merge.
	v interface{}
	// valid indicates whether future reads of the annotation may use v as-is.
	// If false, v will be zeroed and recalculated.
	valid bool
}

type leafNode struct {
	ref   atomic.Int32
	count int16
	leaf  bool
	// subtreeCount holds the count of files in the entire subtree formed by
	// this node. For leaf nodes, subtreeCount is always equal to count. For
	// non-leaf nodes, it's the sum of count plus all the children's
	// subtreeCounts.
	//
	// NB: We could move this field to the end of the node struct, since leaf =>
	// count=subtreeCount, however the unsafe casting [leafToNode] performs make
	// it risky and cumbersome.
	subtreeCount int
	items        [maxItems]*FileMetadata
	// annot contains one annotation per annotator, merged over the entire
	// node's files (and all descendants for non-leaf nodes).
	annot []annotation
}

type node struct {
	leafNode
	children [maxItems + 1]*node
}

//go:nocheckptr casts a ptr to a smaller struct to a ptr to a larger struct.
func leafToNode(ln *leafNode) *node {
	return (*node)(unsafe.Pointer(ln))
}

func newLeafNode() *node {
	n := leafToNode(new(leafNode))
	n.leaf = true
	n.ref.Store(1)
	return n
}

func newNode() *node {
	n := new(node)
	n.ref.Store(1)
	return n
}

// mut creates and returns a mutable node reference. If the node is not shared
// with any other trees then it can be modified in place. Otherwise, it must be
// cloned to ensure unique ownership. In this way, we enforce a copy-on-write
// policy which transparently incorporates the idea of local mutations, like
// Clojure's transients or Haskell's ST monad, where nodes are only copied
// during the first time that they are modified between Clone operations.
//
// When a node is cloned, the provided pointer will be redirected to the new
// mutable node.
func mut(n **node) *node {
	if (*n).ref.Load() == 1 {
		// Exclusive ownership. Can mutate in place.

		// Whenever a node will be mutated, reset its annotations to be marked
		// as uncached. This ensures any future calls to (*node).annotation
		// will recompute annotations on the modified subtree.
		for i := range (*n).annot {
			(*n).annot[i].valid = false
		}
		return *n
	}
	// If we do not have unique ownership over the node then we
	// clone it to gain unique ownership. After doing so, we can
	// release our reference to the old node. We pass recursive
	// as true because even though we just observed the node's
	// reference count to be greater than 1, we might be racing
	// with another call to decRef on this node.
	c := (*n).clone()
	(*n).decRef(true /* contentsToo */, nil)
	*n = c
	// NB: We don't need to clear annotations, because (*node).clone does not
	// copy them.
	return *n
}

// incRef acquires a reference to the node.
func (n *node) incRef() {
	n.ref.Add(1)
}

// decRef releases a reference to the node. If requested, the method will unref
// its items and recurse into child nodes and decrease their refcounts as well.
// Some internal codepaths that manually copy the node's items or children to
// new nodes pass contentsToo=false to preserve existing reference counts during
// operations that should yield a net-zero change to descendant refcounts.
// When a node is released, its contained files are dereferenced.
func (n *node) decRef(contentsToo bool, obsolete *[]*FileBacking) {
	if n.ref.Add(-1) > 0 {
		// Other references remain. Can't free.
		return
	}

	// Dereference the node's metadata and release child references if
	// requested. Some internal callers may not want to propagate the deref
	// because they're manually copying the filemetadata and children to other
	// nodes, and they want to preserve the existing reference count.
	if contentsToo {
		for _, f := range n.items[:n.count] {
			if f.Unref() == 0 {
				// There are two sources of node dereferences: tree mutations
				// and Version dereferences. Files should only be made obsolete
				// during Version dereferences, during which `obsolete` will be
				// non-nil.
				if obsolete == nil {
					panic(fmt.Sprintf("file metadata %s dereferenced to zero during tree mutation", f.FileNum))
				}
				// Reference counting is performed on the FileBacking. In the case
				// of a virtual sstable, this reference counting is performed on
				// a FileBacking which is shared by every single virtual sstable
				// with the same backing sstable. If the reference count hits 0,
				// then we know that the FileBacking won't be required by any
				// sstable in Pebble, and that the backing sstable can be deleted.
				*obsolete = append(*obsolete, f.FileBacking)
			}
		}
		if !n.leaf {
			for i := int16(0); i <= n.count; i++ {
				n.children[i].decRef(true /* contentsToo */, obsolete)
			}
		}
	}
}

// clone creates a clone of the receiver with a single reference count.
func (n *node) clone() *node {
	var c *node
	if n.leaf {
		c = newLeafNode()
	} else {
		c = newNode()
	}
	// NB: copy field-by-field without touching n.ref to avoid
	// triggering the race detector and looking like a data race.
	c.count = n.count
	c.items = n.items
	c.subtreeCount = n.subtreeCount
	// Increase the refcount of each contained item.
	for _, f := range n.items[:n.count] {
		f.Ref()
	}
	if !c.leaf {
		// Copy children and increase each refcount.
		c.children = n.children
		for i := int16(0); i <= c.count; i++ {
			c.children[i].incRef()
		}
	}
	return c
}

// insertAt inserts the provided file and node at the provided index. This
// function is for use only as a helper function for internal B-Tree code.
// Clients should not invoke it directly.
func (n *node) insertAt(index int, item *FileMetadata, nd *node) {
	if index < int(n.count) {
		copy(n.items[index+1:n.count+1], n.items[index:n.count])
		if !n.leaf {
			copy(n.children[index+2:n.count+2], n.children[index+1:n.count+1])
		}
	}
	n.items[index] = item
	if !n.leaf {
		n.children[index+1] = nd
	}
	n.count++
}

// pushBack inserts the provided file and node at the tail of the node's items.
// This function is for use only as a helper function for internal B-Tree code.
// Clients should not invoke it directly.
func (n *node) pushBack(item *FileMetadata, nd *node) {
	n.items[n.count] = item
	if !n.leaf {
		n.children[n.count+1] = nd
	}
	n.count++
}

// pushFront inserts the provided file and node at the head of the
// node's items. This function is for use only as a helper function for internal B-Tree
// code. Clients should not invoke it directly.
func (n *node) pushFront(item *FileMetadata, nd *node) {
	if !n.leaf {
		copy(n.children[1:n.count+2], n.children[:n.count+1])
		n.children[0] = nd
	}
	copy(n.items[1:n.count+1], n.items[:n.count])
	n.items[0] = item
	n.count++
}

// removeAt removes a value at a given index, pulling all subsequent values
// back. This function is for use only as a helper function for internal B-Tree
// code. Clients should not invoke it directly.
func (n *node) removeAt(index int) (*FileMetadata, *node) {
	var child *node
	if !n.leaf {
		child = n.children[index+1]
		copy(n.children[index+1:n.count], n.children[index+2:n.count+1])
		n.children[n.count] = nil
	}
	n.count--
	out := n.items[index]
	copy(n.items[index:n.count], n.items[index+1:n.count+1])
	n.items[n.count] = nil
	return out, child
}

// popBack removes and returns the last element in the list. This function is
// for use only as a helper function for internal B-Tree code. Clients should
// not invoke it directly.
func (n *node) popBack() (*FileMetadata, *node) {
	n.count--
	out := n.items[n.count]
	n.items[n.count] = nil
	if n.leaf {
		return out, nil
	}
	child := n.children[n.count+1]
	n.children[n.count+1] = nil
	return out, child
}

// popFront removes and returns the first element in the list. This function is
// for use only as a helper function for internal B-Tree code. Clients should
// not invoke it directly.
func (n *node) popFront() (*FileMetadata, *node) {
	n.count--
	var child *node
	if !n.leaf {
		child = n.children[0]
		copy(n.children[:n.count+1], n.children[1:n.count+2])
		n.children[n.count+1] = nil
	}
	out := n.items[0]
	copy(n.items[:n.count], n.items[1:n.count+1])
	n.items[n.count] = nil
	return out, child
}

// find returns the index where the given item should be inserted into this
// list. 'found' is true if the item already exists in the list at the given
// index.
//
// This function is for use only as a helper function for internal B-Tree code.
// Clients should not invoke it directly.
func (n *node) find(cmp btreeCmp, item *FileMetadata) (index int, found bool) {
	// Logic copied from sort.Search. Inlining this gave
	// an 11% speedup on BenchmarkBTreeDeleteInsert.
	i, j := 0, int(n.count)
	for i < j {
		h := int(uint(i+j) >> 1) // avoid overflow when computing h
		// i ≤ h < j
		v := cmp(item, n.items[h])
		if v == 0 {
			return h, true
		} else if v > 0 {
			i = h + 1
		} else {
			j = h
		}
	}
	return i, false
}

// split splits the given node at the given index. The current node shrinks,
// and this function returns the item that existed at that index and a new
// node containing all items/children after it.
//
// split is called when we want to perform a transformation like the one
// depicted in the following diagram.
//
//	Before:
//	                       +-----------+
//	             n *node   |   x y z   |
//	                       +--/-/-\-\--+
//
//	After:
//	                       +-----------+
//	                       |     y     |  n's parent
//	                       +----/-\----+
//	                           /   \
//	                          v     v
//	              +-----------+     +-----------+
//	      n *node |         x |     | z         | next *node
//	              +-----------+     +-----------+
//
// split does not perform the complete transformation; the caller is responsible
// for updating the parent appropriately. split splits `n` into two nodes, `n`
// and `next`, returning `next` and the file that separates them. In the diagram
// above, `n.split` removes y and z from `n`, returning y in the first return
// value and `next` in the second return value. The caller is responsible for
// updating n's parent to now contain `y` as the separator between nodes `n` and
// `next`.
//
// This function is for use only as a helper function for internal B-Tree code.
// Clients should not invoke it directly.
func (n *node) split(i int) (*FileMetadata, *node) {
	out := n.items[i]
	var next *node
	if n.leaf {
		next = newLeafNode()
	} else {
		next = newNode()
	}
	next.count = n.count - int16(i+1)
	copy(next.items[:], n.items[i+1:n.count])
	for j := int16(i); j < n.count; j++ {
		n.items[j] = nil
	}
	if !n.leaf {
		copy(next.children[:], n.children[i+1:n.count+1])
		descendantsMoved := 0
		for j := int16(i + 1); j <= n.count; j++ {
			descendantsMoved += n.children[j].subtreeCount
			n.children[j] = nil
		}
		n.subtreeCount -= descendantsMoved
		next.subtreeCount += descendantsMoved
	}
	n.count = int16(i)
	// NB: We subtract one more than `next.count` from n's subtreeCount because
	// the item at index `i` was removed from `n.items`. We'll return the item
	// at index `i`, and the caller is responsible for updating the subtree
	// count of whichever node adopts it.
	n.subtreeCount -= int(next.count) + 1
	next.subtreeCount += int(next.count)
	return out, next
}

// Insert inserts a item into the subtree rooted at this node, making sure no
// nodes in the subtree exceed maxItems items.
func (n *node) Insert(cmp btreeCmp, item *FileMetadata) error {
	i, found := n.find(cmp, item)
	if found {
		// cmp provides a total ordering of the files within a level.
		// If we're inserting a metadata that's equal to an existing item
		// in the tree, we're inserting a file into a level twice.
		return errors.Errorf("files %s and %s collided on sort keys",
			errors.Safe(item.FileNum), errors.Safe(n.items[i].FileNum))
	}
	if n.leaf {
		n.insertAt(i, item, nil)
		n.subtreeCount++
		return nil
	}
	if n.children[i].count >= maxItems {
		splitLa, splitNode := mut(&n.children[i]).split(maxItems / 2)
		n.insertAt(i, splitLa, splitNode)

		switch cmp := cmp(item, n.items[i]); {
		case cmp < 0:
			// no change, we want first split node
		case cmp > 0:
			i++ // we want second split node
		default:
			// cmp provides a total ordering of the files within a level.
			// If we're inserting a metadata that's equal to an existing item
			// in the tree, we're inserting a file into a level twice.
			return errors.Errorf("files %s and %s collided on sort keys",
				errors.Safe(item.FileNum), errors.Safe(n.items[i].FileNum))
		}
	}

	err := mut(&n.children[i]).Insert(cmp, item)
	if err == nil {
		n.subtreeCount++
	}
	return err
}

// removeMax removes and returns the maximum item from the subtree rooted at
// this node. This function is for use only as a helper function for internal
// B-Tree code. Clients should not invoke it directly.
func (n *node) removeMax() *FileMetadata {
	if n.leaf {
		n.count--
		n.subtreeCount--
		out := n.items[n.count]
		n.items[n.count] = nil
		return out
	}
	child := mut(&n.children[n.count])
	if child.count <= minItems {
		n.rebalanceOrMerge(int(n.count))
		return n.removeMax()
	}
	n.subtreeCount--
	return child.removeMax()
}

// Remove removes a item from the subtree rooted at this node. Returns
// the item that was removed or nil if no matching item was found.
func (n *node) Remove(cmp btreeCmp, item *FileMetadata) (out *FileMetadata) {
	i, found := n.find(cmp, item)
	if n.leaf {
		if found {
			out, _ = n.removeAt(i)
			n.subtreeCount--
			return out
		}
		return nil
	}
	if n.children[i].count <= minItems {
		// Child not large enough to remove from.
		n.rebalanceOrMerge(i)
		return n.Remove(cmp, item)
	}
	child := mut(&n.children[i])
	if found {
		// Replace the item being removed with the max item in our left child.
		out = n.items[i]
		n.items[i] = child.removeMax()
		n.subtreeCount--
		return out
	}
	// File is not in this node and child is large enough to remove from.
	out = child.Remove(cmp, item)
	if out != nil {
		n.subtreeCount--
	}
	return out
}

// rebalanceOrMerge grows child 'i' to ensure it has sufficient room to remove a
// item from it while keeping it at or above minItems. This function is for use
// only as a helper function for internal B-Tree code. Clients should not invoke
// it directly.
func (n *node) rebalanceOrMerge(i int) {
	switch {
	case i > 0 && n.children[i-1].count > minItems:
		// Rebalance from left sibling.
		//
		//          +-----------+
		//          |     y     |
		//          +----/-\----+
		//              /   \
		//             v     v
		// +-----------+     +-----------+
		// |         x |     |           |
		// +----------\+     +-----------+
		//             \
		//              v
		//              a
		//
		// After:
		//
		//          +-----------+
		//          |     x     |
		//          +----/-\----+
		//              /   \
		//             v     v
		// +-----------+     +-----------+
		// |           |     | y         |
		// +-----------+     +/----------+
		//                   /
		//                  v
		//                  a
		//
		left := mut(&n.children[i-1])
		child := mut(&n.children[i])
		xLa, grandChild := left.popBack()
		yLa := n.items[i-1]
		child.pushFront(yLa, grandChild)
		n.items[i-1] = xLa
		child.subtreeCount++
		left.subtreeCount--
		if grandChild != nil {
			child.subtreeCount += grandChild.subtreeCount
			left.subtreeCount -= grandChild.subtreeCount
		}

	case i < int(n.count) && n.children[i+1].count > minItems:
		// Rebalance from right sibling.
		//
		//          +-----------+
		//          |     y     |
		//          +----/-\----+
		//              /   \
		//             v     v
		// +-----------+     +-----------+
		// |           |     | x         |
		// +-----------+     +/----------+
		//                   /
		//                  v
		//                  a
		//
		// After:
		//
		//          +-----------+
		//          |     x     |
		//          +----/-\----+
		//              /   \
		//             v     v
		// +-----------+     +-----------+
		// |         y |     |           |
		// +----------\+     +-----------+
		//             \
		//              v
		//              a
		//
		right := mut(&n.children[i+1])
		child := mut(&n.children[i])
		xLa, grandChild := right.popFront()
		yLa := n.items[i]
		child.pushBack(yLa, grandChild)
		child.subtreeCount++
		right.subtreeCount--
		if grandChild != nil {
			child.subtreeCount += grandChild.subtreeCount
			right.subtreeCount -= grandChild.subtreeCount
		}
		n.items[i] = xLa

	default:
		// Merge with either the left or right sibling.
		//
		//          +-----------+
		//          |   u y v   |
		//          +----/-\----+
		//              /   \
		//             v     v
		// +-----------+     +-----------+
		// |         x |     | z         |
		// +-----------+     +-----------+
		//
		// After:
		//
		//          +-----------+
		//          |    u v    |
		//          +-----|-----+
		//                |
		//                v
		//          +-----------+
		//          |   x y z   |
		//          +-----------+
		//
		if i >= int(n.count) {
			i = int(n.count - 1)
		}
		child := mut(&n.children[i])
		// Make mergeChild mutable, bumping the refcounts on its children if necessary.
		_ = mut(&n.children[i+1])
		mergeLa, mergeChild := n.removeAt(i)
		child.items[child.count] = mergeLa
		copy(child.items[child.count+1:], mergeChild.items[:mergeChild.count])
		if !child.leaf {
			copy(child.children[child.count+1:], mergeChild.children[:mergeChild.count+1])
		}
		child.count += mergeChild.count + 1
		child.subtreeCount += mergeChild.subtreeCount + 1

		mergeChild.decRef(false /* contentsToo */, nil)
	}
}

// InvalidateAnnotation removes any existing cached annotations for the provided
// annotator from this node's subtree.
func (n *node) InvalidateAnnotation(a Annotator) {
	// Find this annotator's annotation on this node.
	var annot *annotation
	for i := range n.annot {
		if n.annot[i].annotator == a {
			annot = &n.annot[i]
		}
	}

	if annot != nil && annot.valid {
		annot.valid = false
		annot.v = a.Zero(annot.v)
	}
	if !n.leaf {
		for i := int16(0); i <= n.count; i++ {
			n.children[i].InvalidateAnnotation(a)
		}
	}
}

// Annotation retrieves, computing if not already computed, the provided
// annotator's annotation of this node. The second return value indicates
// whether the future reads of this annotation may use the first return value
// as-is. If false, the annotation is not stable and may change on a subsequent
// computation.
func (n *node) Annotation(a Annotator) (interface{}, bool) {
	// Find this annotator's annotation on this node.
	var annot *annotation
	for i := range n.annot {
		if n.annot[i].annotator == a {
			annot = &n.annot[i]
		}
	}

	// If it exists and is marked as valid, we can return it without
	// recomputing anything.
	if annot != nil && annot.valid {
		return annot.v, true
	}

	if annot == nil {
		// This is n's first time being annotated by a.
		// Create a new zeroed annotation.
		n.annot = append(n.annot, annotation{
			annotator: a,
			v:         a.Zero(nil),
		})
		annot = &n.annot[len(n.annot)-1]
	} else {
		// There's an existing annotation that must be recomputed.
		// Zero its value.
		annot.v = a.Zero(annot.v)
	}

	annot.valid = true
	for i := int16(0); i <= n.count; i++ {
		if !n.leaf {
			v, ok := n.children[i].Annotation(a)
			annot.v = a.Merge(v, annot.v)
			annot.valid = annot.valid && ok
		}
		if i < n.count {
			v, ok := a.Accumulate(n.items[i], annot.v)
			annot.v = v
			annot.valid = annot.valid && ok
		}
	}
	return annot.v, annot.valid
}

func (n *node) verifyInvariants() {
	recomputedSubtreeCount := int(n.count)
	if !n.leaf {
		for i := int16(0); i <= n.count; i++ {
			n.children[i].verifyInvariants()
			recomputedSubtreeCount += n.children[i].subtreeCount
		}
	}
	if recomputedSubtreeCount != n.subtreeCount {
		panic(fmt.Sprintf("recomputed subtree count (%d) ≠ n.subtreeCount (%d)",
			recomputedSubtreeCount, n.subtreeCount))
	}
}

// btree is an implementation of a B-Tree.
//
// btree stores FileMetadata in an ordered structure, allowing easy insertion,
// removal, and iteration. The B-Tree stores items in order based on cmp. The
// first level of the LSM uses a cmp function that compares sequence numbers.
// All other levels compare using the FileMetadata.Smallest.
//
// Write operations are not safe for concurrent mutation by multiple
// goroutines, but Read operations are.
type btree struct {
	root *node
	cmp  btreeCmp
}

// Release dereferences and clears the root node of the btree, removing all
// items from the btree. In doing so, it decrements contained file counts.
// It returns a slice of newly obsolete backing files, if any.
func (t *btree) Release() (obsolete []*FileBacking) {
	if t.root != nil {
		t.root.decRef(true /* contentsToo */, &obsolete)
		t.root = nil
	}
	return obsolete
}

// Clone clones the btree, lazily. It does so in constant time.
func (t *btree) Clone() btree {
	c := *t
	if c.root != nil {
		// Incrementing the reference count on the root node is sufficient to
		// ensure that no node in the cloned tree can be mutated by an actor
		// holding a reference to the original tree and vice versa. This
		// property is upheld because the root node in the receiver btree and
		// the returned btree will both necessarily have a reference count of at
		// least 2 when this method returns. All tree mutations recursively
		// acquire mutable node references (see mut) as they traverse down the
		// tree. The act of acquiring a mutable node reference performs a clone
		// if a node's reference count is greater than one. Cloning a node (see
		// clone) increases the reference count on each of its children,
		// ensuring that they have a reference count of at least 2. This, in
		// turn, ensures that any of the child nodes that are modified will also
		// be copied-on-write, recursively ensuring the immutability property
		// over the entire tree.
		c.root.incRef()
	}
	return c
}

// Delete removes the provided file from the tree.
// It returns true if the file now has a zero reference count.
func (t *btree) Delete(item *FileMetadata) (obsolete bool) {
	if t.root == nil || t.root.count == 0 {
		return false
	}
	if out := mut(&t.root).Remove(t.cmp, item); out != nil {
		obsolete = out.Unref() == 0
	}
	if invariants.Enabled {
		t.root.verifyInvariants()
	}
	if t.root.count == 0 {
		old := t.root
		if t.root.leaf {
			t.root = nil
		} else {
			t.root = t.root.children[0]
		}
		old.decRef(false /* contentsToo */, nil)
	}
	return obsolete
}

// Insert adds the given item to the tree. If a item in the tree already
// equals the given one, Insert panics.
func (t *btree) Insert(item *FileMetadata) error {
	if t.root == nil {
		t.root = newLeafNode()
	} else if t.root.count >= maxItems {
		splitLa, splitNode := mut(&t.root).split(maxItems / 2)
		newRoot := newNode()
		newRoot.count = 1
		newRoot.items[0] = splitLa
		newRoot.children[0] = t.root
		newRoot.children[1] = splitNode
		newRoot.subtreeCount = t.root.subtreeCount + splitNode.subtreeCount + 1
		t.root = newRoot
	}
	item.Ref()
	err := mut(&t.root).Insert(t.cmp, item)
	if invariants.Enabled {
		t.root.verifyInvariants()
	}
	return err
}

// Iter returns a new iterator object. It is not safe to continue using an
// iterator after modifications are made to the tree. If modifications are made,
// create a new iterator.
func (t *btree) Iter() iterator {
	return iterator{r: t.root, pos: -1, cmp: t.cmp}
}

// Count returns the number of files contained within the B-Tree.
func (t *btree) Count() int {
	if t.root == nil {
		return 0
	}
	return t.root.subtreeCount
}

// String returns a string description of the tree. The format is
// similar to the https://en.wikipedia.org/wiki/Newick_format.
func (t *btree) String() string {
	if t.Count() == 0 {
		return ";"
	}
	var b strings.Builder
	t.root.writeString(&b)
	return b.String()
}

func (n *node) writeString(b *strings.Builder) {
	if n.leaf {
		for i := int16(0); i < n.count; i++ {
			if i != 0 {
				b.WriteString(",")
			}
			b.WriteString(n.items[i].String())
		}
		return
	}
	for i := int16(0); i <= n.count; i++ {
		b.WriteString("(")
		n.children[i].writeString(b)
		b.WriteString(")")
		if i < n.count {
			b.WriteString(n.items[i].String())
		}
	}
}

// iterStack represents a stack of (node, pos) tuples, which captures
// iteration state as an iterator descends a btree.
type iterStack struct {
	// a contains aLen stack frames when an iterator stack is short enough.
	// If the iterator stack overflows the capacity of iterStackArr, the stack
	// is moved to s and aLen is set to -1.
	a    iterStackArr
	aLen int16 // -1 when using s
	s    []iterFrame
}

// Used to avoid allocations for stacks below a certain size.
type iterStackArr [3]iterFrame

type iterFrame struct {
	n   *node
	pos int16
}

func (is *iterStack) push(f iterFrame) {
	if is.aLen == -1 {
		is.s = append(is.s, f)
	} else if int(is.aLen) == len(is.a) {
		is.s = make([]iterFrame, int(is.aLen)+1, 2*int(is.aLen))
		copy(is.s, is.a[:])
		is.s[int(is.aLen)] = f
		is.aLen = -1
	} else {
		is.a[is.aLen] = f
		is.aLen++
	}
}

func (is *iterStack) pop() iterFrame {
	if is.aLen == -1 {
		f := is.s[len(is.s)-1]
		is.s = is.s[:len(is.s)-1]
		return f
	}
	is.aLen--
	return is.a[is.aLen]
}

func (is *iterStack) len() int {
	if is.aLen == -1 {
		return len(is.s)
	}
	return int(is.aLen)
}

func (is *iterStack) clone() iterStack {
	// If the iterator is using the embedded iterStackArr, we only need to
	// copy the struct itself.
	if is.s == nil {
		return *is
	}
	clone := *is
	clone.s = make([]iterFrame, len(is.s))
	copy(clone.s, is.s)
	return clone
}

func (is *iterStack) nth(n int) (f iterFrame, ok bool) {
	if is.aLen == -1 {
		if n >= len(is.s) {
			return f, false
		}
		return is.s[n], true
	}
	if int16(n) >= is.aLen {
		return f, false
	}
	return is.a[n], true
}

func (is *iterStack) reset() {
	if is.aLen == -1 {
		is.s = is.s[:0]
	} else {
		is.aLen = 0
	}
}

// iterator is responsible for search and traversal within a btree.
type iterator struct {
	// the root node of the B-Tree.
	r *node
	// n and pos make up the current position of the iterator.
	// If valid, n.items[pos] is the current value of the iterator.
	//
	// n may be nil iff i.r is nil.
	n   *node
	pos int16
	// cmp dictates the ordering of the FileMetadata.
	cmp func(*FileMetadata, *FileMetadata) int
	// a stack of n's ancestors within the B-Tree, alongside the position
	// taken to arrive at n. If non-empty, the bottommost frame of the stack
	// will always contain the B-Tree root.
	s iterStack
}

// countLeft returns the count of files that are to the left of the current
// iterator position.
func (i *iterator) countLeft() int {
	if i.r == nil {
		return 0
	}

	// Each iterator has a stack of frames marking the path from the root node
	// to the current iterator position. All files (n.items) and all subtrees
	// (n.children) with indexes less than [pos] are to the left of the current
	// iterator position.
	//
	//     +------------------------+  -
	//     |  Root            pos:5 |   |
	//     +------------------------+   | stack
	//     |  Root/5          pos:3 |   | frames
	//     +------------------------+   | [i.s]
	//     |  Root/5/3        pos:9 |   |
	//     +========================+  -
	//     |                        |
	//     | i.n: Root/5/3/9 i.pos:2|
	//     +------------------------+
	//
	var count int
	// Walk all the ancestors in the iterator stack [i.s], tallying up all the
	// files and subtrees to the left of the stack frame's position.
	f, ok := i.s.nth(0)
	for fi := 0; ok; fi++ {
		// There are [f.pos] files contained within [f.n.items] that sort to the
		// left of the subtree the iterator has descended.
		count += int(f.pos)
		// Any subtrees that fall before the stack frame's position are entirely
		// to the left of the iterator's current position.
		for j := int16(0); j < f.pos; j++ {
			count += f.n.children[j].subtreeCount
		}
		f, ok = i.s.nth(fi + 1)
	}

	// The bottommost stack frame is inlined within the iterator struct. Again,
	// [i.pos] files fall to the left of the current iterator position.
	count += int(i.pos)
	if !i.n.leaf {
		// NB: Unlike above, we use a `<= i.pos` comparison. The iterator is
		// positioned at item `i.n.items[i.pos]`, which sorts after everything
		// in the subtree at `i.n.children[i.pos]`.
		for j := int16(0); j <= i.pos; j++ {
			count += i.n.children[j].subtreeCount
		}
	}
	return count
}

func (i *iterator) clone() iterator {
	c := *i
	c.s = i.s.clone()
	return c
}

func (i *iterator) reset() {
	i.n = i.r
	i.pos = -1
	i.s.reset()
}

func (i iterator) String() string {
	var buf bytes.Buffer
	for n := 0; ; n++ {
		f, ok := i.s.nth(n)
		if !ok {
			break
		}
		fmt.Fprintf(&buf, "%p: %02d/%02d\n", f.n, f.pos, f.n.count)
	}
	if i.r == nil {
		fmt.Fprintf(&buf, "<nil>: %02d", i.pos)
	} else {
		fmt.Fprintf(&buf, "%p: %02d/%02d", i.n, i.pos, i.n.count)
	}
	return buf.String()
}

func cmpIter(a, b iterator) int {
	if a.r != b.r {
		panic("compared iterators from different btrees")
	}

	// Each iterator has a stack of frames marking the path from the root node
	// to the current iterator position. We walk both paths formed by the
	// iterators' stacks simultaneously, descending from the shared root node,
	// always comparing nodes at the same level in the tree.
	//
	// If the iterators' paths ever diverge and point to different nodes, the
	// iterators are not equal and we use the node positions to evaluate the
	// comparison.
	//
	// If an iterator's stack ends, we stop descending and use its current
	// node and position for the final comparison. One iterator's stack may
	// end before another's if one iterator is positioned deeper in the tree.
	//
	// a                                b
	// +------------------------+      +--------------------------+ -
	// |  Root            pos:5 |   =  |  Root              pos:5 |  |
	// +------------------------+      +--------------------------+  | stack
	// |  Root/5          pos:3 |   =  |  Root/5            pos:3 |  | frames
	// +------------------------+      +--------------------------+  |
	// |  Root/5/3        pos:9 |   >  |  Root/5/3          pos:1 |  |
	// +========================+      +==========================+ -
	// |                        |      |                          |
	// | a.n: Root/5/3/9 a.pos:2|      | b.n: Root/5/3/1, b.pos:5 |
	// +------------------------+      +--------------------------+

	// Initialize with the iterator's current node and position. These are
	// conceptually the most-recent/current frame of the iterator stack.
	an, apos := a.n, a.pos
	bn, bpos := b.n, b.pos

	// aok, bok are set while traversing the iterator's path down the B-Tree.
	// They're declared in the outer scope because they help distinguish the
	// sentinel case when both iterators' first frame points to the last child
	// of the root. If an iterator has no other frames in its stack, it's the
	// end sentinel state which sorts after everything else.
	var aok, bok bool
	for i := 0; ; i++ {
		var af, bf iterFrame
		af, aok = a.s.nth(i)
		bf, bok = b.s.nth(i)
		if !aok || !bok {
			if aok {
				// Iterator a, unlike iterator b, still has a frame. Set an,
				// apos so we compare using the frame from the stack.
				an, apos = af.n, af.pos
			}
			if bok {
				// Iterator b, unlike iterator a, still has a frame. Set bn,
				// bpos so we compare using the frame from the stack.
				bn, bpos = bf.n, bf.pos
			}
			break
		}

		// aok && bok
		if af.n != bf.n {
			panic("nonmatching nodes during btree iterator comparison")
		}
		switch {
		case af.pos < bf.pos:
			return -1
		case af.pos > bf.pos:
			return +1
		default:
			// Continue up both iterators' stacks (equivalently, down the
			// B-Tree away from the root).
		}
	}

	if aok && bok {
		panic("expected one or more stacks to have been exhausted")
	}
	if an != bn {
		panic("nonmatching nodes during btree iterator comparison")
	}
	switch {
	case apos < bpos:
		return -1
	case apos > bpos:
		return +1
	default:
		switch {
		case aok:
			// a is positioned at a leaf child at this position and b is at an
			// end sentinel state.
			return -1
		case bok:
			// b is positioned at a leaf child at this position and a is at an
			// end sentinel state.
			return +1
		default:
			return 0
		}
	}
}

func (i *iterator) descend(n *node, pos int16) {
	i.s.push(iterFrame{n: n, pos: pos})
	i.n = n.children[pos]
	i.pos = 0
}

// ascend ascends up to the current node's parent and resets the position
// to the one previously set for this parent node.
func (i *iterator) ascend() {
	f := i.s.pop()
	i.n = f.n
	i.pos = f.pos
}

// seek repositions the iterator over the first file for which fn returns
// true, mirroring the semantics of the standard library's sort.Search
// function.  Like sort.Search, seek requires the iterator's B-Tree to be
// ordered such that fn returns false for some (possibly empty) prefix of the
// tree's files, and then true for the (possibly empty) remainder.
func (i *iterator) seek(fn func(*FileMetadata) bool) {
	i.reset()
	if i.r == nil {
		return
	}

	for {
		// Logic copied from sort.Search.
		j, k := 0, int(i.n.count)
		for j < k {
			h := int(uint(j+k) >> 1) // avoid overflow when computing h

			// j ≤ h < k
			if !fn(i.n.items[h]) {
				j = h + 1 // preserves f(j-1) == false
			} else {
				k = h // preserves f(k) == true
			}
		}

		i.pos = int16(j)
		if i.n.leaf {
			if i.pos == i.n.count {
				i.next()
			}
			return
		}
		i.descend(i.n, i.pos)
	}
}

// first seeks to the first item in the btree.
func (i *iterator) first() {
	i.reset()
	if i.r == nil {
		return
	}
	for !i.n.leaf {
		i.descend(i.n, 0)
	}
	i.pos = 0
}

// last seeks to the last item in the btree.
func (i *iterator) last() {
	i.reset()
	if i.r == nil {
		return
	}
	for !i.n.leaf {
		i.descend(i.n, i.n.count)
	}
	i.pos = i.n.count - 1
}

// next positions the iterator to the item immediately following
// its current position.
func (i *iterator) next() {
	if i.r == nil {
		return
	}

	if i.n.leaf {
		if i.pos < i.n.count {
			i.pos++
		}
		if i.pos < i.n.count {
			return
		}
		for i.s.len() > 0 && i.pos >= i.n.count {
			i.ascend()
		}
		return
	}

	i.descend(i.n, i.pos+1)
	for !i.n.leaf {
		i.descend(i.n, 0)
	}
	i.pos = 0
}

// prev positions the iterator to the item immediately preceding
// its current position.
func (i *iterator) prev() {
	if i.r == nil {
		return
	}

	if i.n.leaf {
		i.pos--
		if i.pos >= 0 {
			return
		}
		for i.s.len() > 0 && i.pos < 0 {
			i.ascend()
			i.pos--
		}
		return
	}

	i.descend(i.n, i.pos)
	for !i.n.leaf {
		i.descend(i.n, i.n.count)
	}
	i.pos = i.n.count - 1
}

// valid returns whether the iterator is positioned at a valid position.
func (i *iterator) valid() bool {
	return i.r != nil && i.pos >= 0 && i.pos < i.n.count
}

// cur returns the item at the iterator's current position. It is illegal
// to call cur if the iterator is not valid.
func (i *iterator) cur() *FileMetadata {
	if invariants.Enabled && !i.valid() {
		panic("btree iterator.cur invoked on invalid iterator")
	}
	return i.n.items[i.pos]
}
