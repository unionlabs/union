package iavl

// NOTE: This file favors int64 as opposed to int for size/counts.
// The Tree on the other hand favors int.  This is intentional.

import (
	"bytes"
	"errors"

	dbm "github.com/cosmos/cosmos-db"
)

type traversal struct {
	tree         *ImmutableTree
	start, end   []byte        // iteration domain
	ascending    bool          // ascending traversal
	inclusive    bool          // end key inclusiveness
	post         bool          // postorder traversal
	delayedNodes *delayedNodes // delayed nodes to be traversed
}

var errIteratorNilTreeGiven = errors.New("iterator must be created with an immutable tree but the tree was nil")

func (node *Node) newTraversal(tree *ImmutableTree, start, end []byte, ascending bool, inclusive bool, post bool) *traversal {
	return &traversal{
		tree:         tree,
		start:        start,
		end:          end,
		ascending:    ascending,
		inclusive:    inclusive,
		post:         post,
		delayedNodes: &delayedNodes{{node, true}}, // set initial traverse to the node
	}
}

// delayedNode represents the delayed iteration on the nodes.
// When delayed is set to true, the delayedNode should be expanded, and their
// children should be traversed. When delayed is set to false, the delayedNode is
// already have expanded, and it could be immediately returned.
type delayedNode struct {
	node    *Node
	delayed bool
}

type delayedNodes []delayedNode

func (nodes *delayedNodes) pop() (*Node, bool) {
	node := (*nodes)[len(*nodes)-1]
	*nodes = (*nodes)[:len(*nodes)-1]
	return node.node, node.delayed
}

func (nodes *delayedNodes) push(node *Node, delayed bool) {
	*nodes = append(*nodes, delayedNode{node, delayed})
}

func (nodes *delayedNodes) length() int {
	return len(*nodes)
}

// `traversal` returns the delayed execution of recursive traversal on a tree.
//
// `traversal` will traverse the tree in a depth-first manner. To handle locating
// the next element, and to handle unwinding, the traversal maintains its future
// iteration under `delayedNodes`. At each call of `next()`, it will retrieve the
// next element from the `delayedNodes` and acts accordingly. The `next()` itself
// defines how to unwind the delayed nodes stack. The caller can either call the
// next traversal to proceed, or simply discard the `traversal` struct to stop iteration.
//
// At the each step of `next`, the `delayedNodes` can have one of the three states:
// 1. It has length of 0, meaning that their is no more traversable nodes.
// 2. It has length of 1, meaning that the traverse is being started from the initial node.
// 3. It has length of 2>=, meaning that there are delayed nodes to be traversed.
//
// When the `delayedNodes` are not empty, `next` retrieves the first `delayedNode` and initially check:
// 1. If it is not an delayed node (node.delayed == false) it immediately returns it.
//
// A. If the `node` is a branch node:
//  1. If the traversal is postorder, then append the current node to the t.delayedNodes,
//     with `delayed` set to false. This makes the current node returned *after* all the children
//     are traversed, without being expanded.
//  2. Append the traversable children nodes into the `delayedNodes`, with `delayed` set to true. This
//     makes the children nodes to be traversed, and expanded with their respective children.
//  3. If the traversal is preorder, (with the children to be traversed already pushed to the
//     `delayedNodes`), returns the current node.
//  4. Call `traversal.next()` to further traverse through the `delayedNodes`.
//
// B. If the `node` is a leaf node, it will be returned without expand, by the following process:
//  1. If the traversal is postorder, the current node will be append to the `delayedNodes` with `delayed`
//     set to false, and immediately returned at the subsequent call of `traversal.next()` at the last line.
//  2. If the traversal is preorder, the current node will be returned.
func (t *traversal) next() (*Node, error) {
	// End of traversal.
	if t.delayedNodes.length() == 0 {
		return nil, nil
	}

	node, delayed := t.delayedNodes.pop()

	// Already expanded, immediately return.
	if !delayed || node == nil {
		return node, nil
	}

	afterStart := t.start == nil || bytes.Compare(t.start, node.key) < 0
	startOrAfter := afterStart || bytes.Equal(t.start, node.key)
	beforeEnd := t.end == nil || bytes.Compare(node.key, t.end) < 0
	if t.inclusive {
		beforeEnd = beforeEnd || bytes.Equal(node.key, t.end)
	}

	// case of postorder. A-1 and B-1
	// Recursively process left sub-tree, then right-subtree, then node itself.
	if t.post && (!node.isLeaf() || (startOrAfter && beforeEnd)) {
		t.delayedNodes.push(node, false)
	}

	// case of branch node, traversing children. A-2.
	if !node.isLeaf() {
		// if node is a branch node and the order is ascending,
		// We traverse through the left subtree, then the right subtree.
		if t.ascending {
			if beforeEnd {
				// push the delayed traversal for the right nodes,
				rightNode, err := node.getRightNode(t.tree)
				if err != nil {
					return nil, err
				}
				t.delayedNodes.push(rightNode, true)
			}
			if afterStart {
				// push the delayed traversal for the left nodes,
				leftNode, err := node.getLeftNode(t.tree)
				if err != nil {
					return nil, err
				}
				t.delayedNodes.push(leftNode, true)
			}
		} else {
			// if node is a branch node and the order is not ascending
			// We traverse through the right subtree, then the left subtree.
			if afterStart {
				// push the delayed traversal for the left nodes,
				leftNode, err := node.getLeftNode(t.tree)
				if err != nil {
					return nil, err
				}
				t.delayedNodes.push(leftNode, true)
			}
			if beforeEnd {
				// push the delayed traversal for the right nodes,
				rightNode, err := node.getRightNode(t.tree)
				if err != nil {
					return nil, err
				}
				t.delayedNodes.push(rightNode, true)
			}
		}
	}

	// case of preorder traversal. A-3 and B-2.
	// Process root then (recursively) processing left child, then process right child
	if !t.post && (!node.isLeaf() || (startOrAfter && beforeEnd)) {
		return node, nil
	}

	// Keep traversing and expanding the remaning delayed nodes. A-4.
	return t.next()
}

// Iterator is a dbm.Iterator for ImmutableTree
type Iterator struct {
	start, end []byte

	key, value []byte

	valid bool

	err error

	t *traversal
}

var _ dbm.Iterator = (*Iterator)(nil)

// Returns a new iterator over the immutable tree. If the tree is nil, the iterator will be invalid.
func NewIterator(start, end []byte, ascending bool, tree *ImmutableTree) dbm.Iterator {
	iter := &Iterator{
		start: start,
		end:   end,
	}

	if tree == nil {
		iter.err = errIteratorNilTreeGiven
	} else {
		iter.valid = true
		iter.t = tree.root.newTraversal(tree, start, end, ascending, false, false)
		// Move iterator before the first element
		iter.Next()
	}
	return iter
}

// Domain implements dbm.Iterator.
func (iter *Iterator) Domain() ([]byte, []byte) {
	return iter.start, iter.end
}

// Valid implements dbm.Iterator.
func (iter *Iterator) Valid() bool {
	return iter.valid
}

// Key implements dbm.Iterator
func (iter *Iterator) Key() []byte {
	return iter.key
}

// Value implements dbm.Iterator
func (iter *Iterator) Value() []byte {
	return iter.value
}

// Next implements dbm.Iterator
func (iter *Iterator) Next() {
	if iter.t == nil {
		return
	}

	node, err := iter.t.next()
	// TODO: double-check if this error is correctly handled.
	if node == nil || err != nil {
		iter.t = nil
		iter.valid = false
		return
	}

	if node.subtreeHeight == 0 {
		iter.key, iter.value = node.key, node.value
		return
	}

	iter.Next()
}

// Close implements dbm.Iterator
func (iter *Iterator) Close() error {
	iter.t = nil
	iter.valid = false
	return iter.err
}

// Error implements dbm.Iterator
func (iter *Iterator) Error() error {
	return iter.err
}

// IsFast returnts true if iterator uses fast strategy
func (iter *Iterator) IsFast() bool {
	return false
}

// NodeIterator is an iterator for nodeDB to traverse a tree in depth-first, preorder manner.
type NodeIterator struct {
	nodesToVisit []*Node
	ndb          *nodeDB
	err          error
}

// NewNodeIterator returns a new NodeIterator to traverse the tree of the root node.
func NewNodeIterator(rootKey []byte, ndb *nodeDB) (*NodeIterator, error) {
	if len(rootKey) == 0 {
		return &NodeIterator{
			nodesToVisit: []*Node{},
			ndb:          ndb,
		}, nil
	}

	node, err := ndb.GetNode(rootKey)
	if err != nil {
		return nil, err
	}

	return &NodeIterator{
		nodesToVisit: []*Node{node},
		ndb:          ndb,
	}, nil
}

// GetNode returns the current visiting node.
func (iter *NodeIterator) GetNode() *Node {
	return iter.nodesToVisit[len(iter.nodesToVisit)-1]
}

// Valid checks if the validator is valid.
func (iter *NodeIterator) Valid() bool {
	return iter.err == nil && len(iter.nodesToVisit) > 0
}

// Error returns an error if any errors.
func (iter *NodeIterator) Error() error {
	return iter.err
}

// Next moves forward the traversal.
// if isSkipped is true, the subtree under the current node is skipped.
func (iter *NodeIterator) Next(isSkipped bool) {
	if !iter.Valid() {
		return
	}
	node := iter.GetNode()
	iter.nodesToVisit = iter.nodesToVisit[:len(iter.nodesToVisit)-1]

	if isSkipped {
		return
	}

	if node.isLeaf() {
		return
	}

	rightNode, err := iter.ndb.GetNode(node.rightNodeKey)
	if err != nil {
		iter.err = err
		return
	}
	iter.nodesToVisit = append(iter.nodesToVisit, rightNode)

	leftNode, err := iter.ndb.GetNode(node.leftNodeKey)
	if err != nil {
		iter.err = err
		return
	}
	iter.nodesToVisit = append(iter.nodesToVisit, leftNode)
}
