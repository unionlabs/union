package ssz

import (
	"encoding/binary"
	"errors"
)

// Proof represents a merkle proof against a general index.
type Proof struct {
	Index  int
	Leaf   []byte
	Hashes [][]byte
}

// Multiproof represents a merkle proof of several leaves.
type Multiproof struct {
	Indices []int
	Leaves  [][]byte
	Hashes  [][]byte
}

// Compress returns a new proof with zero hashes omitted.
// See `CompressedMultiproof` for more info.
func (p *Multiproof) Compress() *CompressedMultiproof {
	compressed := &CompressedMultiproof{
		Indices:    p.Indices,
		Leaves:     p.Leaves,
		Hashes:     make([][]byte, 0, len(p.Hashes)),
		ZeroLevels: make([]int, 0, len(p.Hashes)),
	}

	for _, h := range p.Hashes {
		if l, ok := zeroHashLevels[string(h)]; ok {
			compressed.ZeroLevels = append(compressed.ZeroLevels, l)
			compressed.Hashes = append(compressed.Hashes, nil)
		} else {
			compressed.Hashes = append(compressed.Hashes, h)
		}
	}

	return compressed
}

// CompressedMultiproof represents a compressed merkle proof of several leaves.
// Compression is achieved by omitting zero hashes (and their hashes). `ZeroLevels`
// contains information which helps the verifier fill in those hashes.
type CompressedMultiproof struct {
	Indices    []int
	Leaves     [][]byte
	Hashes     [][]byte
	ZeroLevels []int // Stores the level for every omitted zero hash in the proof
}

// Decompress returns a new multiproof, filling in the omitted
// zero hashes. See `CompressedMultiProof` for more info.
func (c *CompressedMultiproof) Decompress() *Multiproof {
	p := &Multiproof{
		Indices: c.Indices,
		Leaves:  c.Leaves,
		Hashes:  make([][]byte, len(c.Hashes)),
	}

	zc := 0
	for i, h := range c.Hashes {
		if h == nil {
			p.Hashes[i] = zeroHashes[c.ZeroLevels[zc]][:]
			zc++
		} else {
			p.Hashes[i] = c.Hashes[i]
		}
	}

	return p
}

// Node represents a node in the tree
// backing of a SSZ object.
type Node struct {
	left  *Node
	right *Node

	value []byte
}

// NewNodeWithValue initializes a leaf node.
func NewNodeWithValue(value []byte) *Node {
	return &Node{left: nil, right: nil, value: value}
}

// NewNodeWithLR initializes a branch node.
func NewNodeWithLR(left, right *Node) *Node {
	return &Node{left: left, right: right, value: nil}
}

// TreeFromChunks constructs a tree from leaf values.
// The number of leaves should be a power of 2.
func TreeFromChunks(chunks [][]byte) (*Node, error) {
	numLeaves := len(chunks)
	if !isPowerOfTwo(numLeaves) {
		return nil, errors.New("Number of leaves should be a power of 2")
	}

	leaves := make([]*Node, numLeaves)
	for i, c := range chunks {
		leaves[i] = &Node{left: nil, right: nil, value: c}
	}
	return TreeFromNodes(leaves)
}

// TreeFromNodes constructs a tree from leaf nodes.
// This is useful for merging subtrees.
// The number of leaves should be a power of 2.
func TreeFromNodes(leaves []*Node) (*Node, error) {
	numLeaves := len(leaves)

	if numLeaves == 1 {
		return leaves[0], nil
	}
	if numLeaves == 2 {
		return NewNodeWithLR(leaves[0], leaves[1]), nil
	}

	if !isPowerOfTwo(numLeaves) {
		return nil, errors.New("Number of leaves should be a power of 2")
	}

	numNodes := numLeaves*2 - 1
	nodes := make([]*Node, numNodes)
	for i := numNodes; i > 0; i-- {
		// Is a leaf
		if i > numNodes-numLeaves {
			nodes[i-1] = leaves[i-numLeaves]
		} else {
			// Is a branch node
			nodes[i-1] = &Node{left: nodes[(i*2)-1], right: nodes[(i*2+1)-1], value: nil}
		}
	}

	return nodes[0], nil
}

func TreeFromNodesWithMixin(leaves []*Node, num, limit int) (*Node, error) {
	numLeaves := len(leaves)
	if !isPowerOfTwo(limit) {
		return nil, errors.New("Size of tree should be a power of 2")
	}

	allLeaves := make([]*Node, limit)
	emptyLeaf := NewNodeWithValue(make([]byte, 32))
	for i := 0; i < limit; i++ {
		if i < numLeaves {
			allLeaves[i] = leaves[i]
		} else {
			allLeaves[i] = emptyLeaf
		}
	}

	mainTree, err := TreeFromNodes(allLeaves)
	if err != nil {
		return nil, err
	}

	// Mixin len
	countLeaf := LeafFromUint64(uint64(num))
	return NewNodeWithLR(mainTree, countLeaf), nil
}

// Get fetches a node with the given general index.
func (n *Node) Get(index int) (*Node, error) {
	pathLen := getPathLength(index)
	cur := n
	for i := pathLen - 1; i >= 0; i-- {
		if isRight := getPosAtLevel(index, i); isRight {
			cur = cur.right
		} else {
			cur = cur.left
		}
		if cur == nil {
			return nil, errors.New("Node not found in tree")
		}
	}

	return cur, nil
}

// Hash returns the hash of the subtree with the given Node as its root.
// If root has no children, it returns root's value (not its hash).
func (n *Node) Hash() []byte {
	// TODO: handle special cases: empty root, one non-empty node
	return hashNode(n)
}

func hashNode(n *Node) []byte {
	// Leaf
	if n.left == nil && n.right == nil {
		return n.value
	}
	// Only one child
	if n.left == nil || n.right == nil {
		panic("Tree incomplete")
	}
	return hashFn(append(hashNode(n.left), hashNode(n.right)...))
}

// Prove returns a list of sibling values and hashes needed
// to compute the root hash for a given general index.
func (n *Node) Prove(index int) (*Proof, error) {
	pathLen := getPathLength(index)
	proof := &Proof{Index: index}
	hashes := make([][]byte, 0, pathLen)

	cur := n
	for i := pathLen - 1; i >= 0; i-- {
		var siblingHash []byte
		if isRight := getPosAtLevel(index, i); isRight {
			siblingHash = hashNode(cur.left)
			cur = cur.right
		} else {
			siblingHash = hashNode(cur.right)
			cur = cur.left
		}
		hashes = append([][]byte{siblingHash}, hashes...)
		if cur == nil {
			return nil, errors.New("Node not found in tree")
		}
	}

	proof.Hashes = hashes
	proof.Leaf = cur.value

	return proof, nil
}

func (n *Node) ProveMulti(indices []int) (*Multiproof, error) {
	reqIndices := getRequiredIndices(indices)
	proof := &Multiproof{Indices: indices, Leaves: make([][]byte, len(indices)), Hashes: make([][]byte, len(reqIndices))}

	for i, gi := range indices {
		node, err := n.Get(gi)
		if err != nil {
			return nil, err
		}
		proof.Leaves[i] = node.value
	}

	for i, gi := range reqIndices {
		cur, err := n.Get(gi)
		if err != nil {
			return nil, err
		}
		proof.Hashes[i] = hashNode(cur)
	}

	return proof, nil
}

func LeafFromUint64(i uint64) *Node {
	buf := make([]byte, 32)
	binary.LittleEndian.PutUint64(buf[:8], i)
	return NewNodeWithValue(buf)
}

func LeafFromUint32(i uint32) *Node {
	buf := make([]byte, 32)
	binary.LittleEndian.PutUint32(buf[:4], i)
	return NewNodeWithValue(buf)
}

func LeafFromUint16(i uint16) *Node {
	buf := make([]byte, 32)
	binary.LittleEndian.PutUint16(buf[:2], i)
	return NewNodeWithValue(buf)
}

func LeafFromUint8(i uint8) *Node {
	buf := make([]byte, 32)
	buf[0] = byte(i)
	return NewNodeWithValue(buf)
}

func LeafFromBool(b bool) *Node {
	buf := make([]byte, 32)
	if b {
		buf[0] = 1
	}
	return NewNodeWithValue(buf)
}

func LeafFromBytes(b []byte) *Node {
	l := len(b)
	if l > 32 {
		panic("Unimplemented")
	}

	if l == 32 {
		return NewNodeWithValue(b[:])
	}

	return NewNodeWithValue(append(b, zeroBytes[:32-l]...))
}

func EmptyLeaf() *Node {
	return NewNodeWithValue(zeroBytes[:32])
}

func LeavesFromUint64(items []uint64) []*Node {
	if len(items) == 0 {
		return []*Node{}
	}

	numLeaves := (len(items)*8 + 31) / 32
	buf := make([]byte, numLeaves*32)
	for i, v := range items {
		binary.LittleEndian.PutUint64(buf[i*8:(i+1)*8], v)
	}

	leaves := make([]*Node, numLeaves)
	for i := 0; i < numLeaves; i++ {
		v := buf[i*32 : (i+1)*32]
		leaves[i] = NewNodeWithValue(v)
	}

	return leaves
}

func isPowerOfTwo(n int) bool {
	return (n & (n - 1)) == 0
}
