package iavl

import (
	"bytes"
	"crypto/sha256"
	"errors"
	"fmt"
	"sync"

	hexbytes "github.com/cosmos/iavl/internal/bytes"
	"github.com/cosmos/iavl/internal/encoding"
)

var bufPool = &sync.Pool{
	New: func() interface{} {
		return new(bytes.Buffer)
	},
}

var (
	// ErrInvalidProof is returned by Verify when a proof cannot be validated.
	ErrInvalidProof = fmt.Errorf("invalid proof")

	// ErrInvalidInputs is returned when the inputs passed to the function are invalid.
	ErrInvalidInputs = fmt.Errorf("invalid inputs")

	// ErrInvalidRoot is returned when the root passed in does not match the proof's.
	ErrInvalidRoot = fmt.Errorf("invalid root")
)

//----------------------------------------
// ProofInnerNode
// Contract: Left and Right can never both be set. Will result in a empty `[]` roothash

type ProofInnerNode struct {
	Height  int8   `json:"height"`
	Size    int64  `json:"size"`
	Version int64  `json:"version"`
	Left    []byte `json:"left"`
	Right   []byte `json:"right"`
}

func (pin ProofInnerNode) String() string {
	return pin.stringIndented("")
}

func (pin ProofInnerNode) stringIndented(indent string) string {
	return fmt.Sprintf(`ProofInnerNode{
%s  Height:  %v
%s  Size:    %v
%s  Version: %v
%s  Left:    %X
%s  Right:   %X
%s}`,
		indent, pin.Height,
		indent, pin.Size,
		indent, pin.Version,
		indent, pin.Left,
		indent, pin.Right,
		indent)
}

func (pin ProofInnerNode) Hash(childHash []byte) ([]byte, error) {
	hasher := sha256.New()

	buf := bufPool.Get().(*bytes.Buffer)
	buf.Reset()
	defer bufPool.Put(buf)

	err := encoding.EncodeVarint(buf, int64(pin.Height))
	if err == nil {
		err = encoding.EncodeVarint(buf, pin.Size)
	}
	if err == nil {
		err = encoding.EncodeVarint(buf, pin.Version)
	}

	if len(pin.Left) > 0 && len(pin.Right) > 0 {
		return nil, errors.New("both left and right child hashes are set")
	}

	if len(pin.Left) == 0 {
		if err == nil {
			err = encoding.EncodeBytes(buf, childHash)
		}
		if err == nil {
			err = encoding.EncodeBytes(buf, pin.Right)
		}
	} else {
		if err == nil {
			err = encoding.EncodeBytes(buf, pin.Left)
		}
		if err == nil {
			err = encoding.EncodeBytes(buf, childHash)
		}
	}

	if err != nil {
		return nil, fmt.Errorf("failed to hash ProofInnerNode: %v", err)
	}

	_, err = hasher.Write(buf.Bytes())
	if err != nil {
		return nil, err
	}
	return hasher.Sum(nil), nil
}

//----------------------------------------

type ProofLeafNode struct {
	Key       hexbytes.HexBytes `json:"key"`
	ValueHash hexbytes.HexBytes `json:"value"`
	Version   int64             `json:"version"`
}

func (pln ProofLeafNode) String() string {
	return pln.stringIndented("")
}

func (pln ProofLeafNode) stringIndented(indent string) string {
	return fmt.Sprintf(`ProofLeafNode{
%s  Key:       %v
%s  ValueHash: %X
%s  Version:   %v
%s}`,
		indent, pln.Key,
		indent, pln.ValueHash,
		indent, pln.Version,
		indent)
}

func (pln ProofLeafNode) Hash() ([]byte, error) {
	hasher := sha256.New()

	buf := bufPool.Get().(*bytes.Buffer)
	buf.Reset()
	defer bufPool.Put(buf)

	err := encoding.EncodeVarint(buf, 0)
	if err == nil {
		err = encoding.EncodeVarint(buf, 1)
	}
	if err == nil {
		err = encoding.EncodeVarint(buf, pln.Version)
	}
	if err == nil {
		err = encoding.EncodeBytes(buf, pln.Key)
	}
	if err == nil {
		err = encoding.EncodeBytes(buf, pln.ValueHash)
	}
	if err != nil {
		return nil, fmt.Errorf("failed to hash ProofLeafNode: %v", err)
	}
	_, err = hasher.Write(buf.Bytes())
	if err != nil {
		return nil, err
	}

	return hasher.Sum(nil), nil
}

//----------------------------------------

// If the key does not exist, returns the path to the next leaf left of key (w/
// path), except when key is less than the least item, in which case it returns
// a path to the least item.
func (node *Node) PathToLeaf(t *ImmutableTree, key []byte, version int64) (PathToLeaf, *Node, error) {
	path := new(PathToLeaf)
	val, err := node.pathToLeaf(t, key, version, path)
	return *path, val, err
}

// pathToLeaf is a helper which recursively constructs the PathToLeaf.
// As an optimization the already constructed path is passed in as an argument
// and is shared among recursive calls.
func (node *Node) pathToLeaf(t *ImmutableTree, key []byte, version int64, path *PathToLeaf) (*Node, error) {
	if node.subtreeHeight == 0 {
		if bytes.Equal(node.key, key) {
			return node, nil
		}
		return node, errors.New("key does not exist")
	}

	nodeVersion := version
	if node.nodeKey != nil {
		nodeVersion = node.nodeKey.version
	}
	// Note that we do not store the left child in the ProofInnerNode when we're going to add the
	// left node as part of the path, similarly we don't store the right child info when going down
	// the right child node. This is done as an optimization since the child info is going to be
	// already stored in the next ProofInnerNode in PathToLeaf.
	if bytes.Compare(key, node.key) < 0 {
		// left side
		rightNode, err := node.getRightNode(t)
		if err != nil {
			return nil, err
		}

		pin := ProofInnerNode{
			Height:  node.subtreeHeight,
			Size:    node.size,
			Version: nodeVersion,
			Left:    nil,
			Right:   rightNode.hash,
		}
		*path = append(*path, pin)

		leftNode, err := node.getLeftNode(t)
		if err != nil {
			return nil, err
		}
		n, err := leftNode.pathToLeaf(t, key, version, path)
		return n, err
	}
	// right side
	leftNode, err := node.getLeftNode(t)
	if err != nil {
		return nil, err
	}

	pin := ProofInnerNode{
		Height:  node.subtreeHeight,
		Size:    node.size,
		Version: nodeVersion,
		Left:    leftNode.hash,
		Right:   nil,
	}
	*path = append(*path, pin)

	rightNode, err := node.getRightNode(t)
	if err != nil {
		return nil, err
	}

	n, err := rightNode.pathToLeaf(t, key, version, path)
	return n, err
}
