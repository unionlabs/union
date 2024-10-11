package merkle

import (
	"bytes"
	"errors"
	"fmt"
	"hash"

	cmtcrypto "github.com/cometbft/cometbft/api/cometbft/crypto/v1"
	"github.com/cometbft/cometbft/crypto/tmhash"
)

const (
	// MaxAunts is the maximum number of aunts that can be included in a Proof.
	// This corresponds to a tree of size 2^100, which should be sufficient for all conceivable purposes.
	// This maximum helps prevent Denial-of-Service attacks by limiting the size of the proofs.
	MaxAunts = 100
)

var ErrMaxAuntsLenExceeded = fmt.Errorf("merkle: maximum aunts length, %d, exceeded", MaxAunts)

type ErrInvalidHash struct {
	Err error
}

func (e ErrInvalidHash) Error() string {
	return fmt.Sprintf("merkle: invalid hash: %s", e.Err)
}

func (e ErrInvalidHash) Unwrap() error {
	return e.Err
}

type ErrInvalidProof struct {
	Err error
}

func (e ErrInvalidProof) Error() string {
	return fmt.Sprintf("merkle: invalid proof: %s", e.Err)
}

func (e ErrInvalidProof) Unwrap() error {
	return e.Err
}

// Proof represents a Merkle proof.
// NOTE: The convention for proofs is to include leaf hashes but to
// exclude the root hash.
// This convention is implemented across IAVL range proofs as well.
// Keep this consistent unless there's a very good reason to change
// everything.  This also affects the generalized proof system as
// well.
type Proof struct {
	Total    int64    `json:"total"`           // Total number of items.
	Index    int64    `json:"index"`           // Index of item to prove.
	LeafHash []byte   `json:"leaf_hash"`       // Hash of item value.
	Aunts    [][]byte `json:"aunts,omitempty"` // Hashes from leaf's sibling to a root's child.
}

// ProofsFromByteSlices computes inclusion proof for given items.
// proofs[0] is the proof for items[0].
func ProofsFromByteSlices(items [][]byte) (rootHash []byte, proofs []*Proof) {
	trails, rootSPN := trailsFromByteSlices(items)
	rootHash = rootSPN.Hash
	proofs = make([]*Proof, len(items))
	for i, trail := range trails {
		proofs[i] = &Proof{
			Total:    int64(len(items)),
			Index:    int64(i),
			LeafHash: trail.Hash,
			Aunts:    trail.FlattenAunts(),
		}
	}
	return rootHash, proofs
}

// Verify that the Proof proves the root hash.
// Check sp.Index/sp.Total manually if needed.
func (sp *Proof) Verify(rootHash []byte, leaf []byte) error {
	if rootHash == nil {
		return ErrInvalidHash{
			Err: errors.New("nil root"),
		}
	}
	if sp.Total < 0 {
		return ErrInvalidProof{
			Err: errors.New("negative proof total"),
		}
	}
	if sp.Index < 0 {
		return ErrInvalidProof{
			Err: errors.New("negative proof index"),
		}
	}
	hash := tmhash.New()
	leafHash := leafHashOpt(hash, leaf)
	if !bytes.Equal(sp.LeafHash, leafHash) {
		return ErrInvalidHash{
			Err: fmt.Errorf("leaf %x, want %x", sp.LeafHash, leafHash),
		}
	}
	computedHash, err := sp.computeRootHash(hash)
	if err != nil {
		return ErrInvalidHash{
			Err: fmt.Errorf("compute root hash: %w", err),
		}
	}
	if !bytes.Equal(computedHash, rootHash) {
		return ErrInvalidHash{
			Err: fmt.Errorf("root %x, want %x", computedHash, rootHash),
		}
	}
	return nil
}

// Compute the root hash given a leaf hash.
func (sp *Proof) computeRootHash(hash hash.Hash) ([]byte, error) {
	return computeHashFromAunts(
		hash,
		sp.Index,
		sp.Total,
		sp.LeafHash,
		sp.Aunts,
	)
}

// String implements the stringer interface for Proof.
// It is a wrapper around StringIndented.
func (sp *Proof) String() string {
	return sp.StringIndented("")
}

// StringIndented generates a canonical string representation of a Proof.
func (sp *Proof) StringIndented(indent string) string {
	return fmt.Sprintf(`Proof{
%s  Aunts: %X
%s}`,
		indent, sp.Aunts,
		indent)
}

// ValidateBasic performs basic validation.
// NOTE: it expects the LeafHash and the elements of Aunts to be of size tmhash.Size,
// and it expects at most MaxAunts elements in Aunts.
func (sp *Proof) ValidateBasic() error {
	if sp.Total < 0 {
		return ErrInvalidProof{
			Err: errors.New("negative proof total"),
		}
	}
	if sp.Index < 0 {
		return ErrInvalidProof{
			Err: errors.New("negative proof index"),
		}
	}
	if len(sp.LeafHash) != tmhash.Size {
		return ErrInvalidHash{
			Err: fmt.Errorf("leaf length %d, want %d", len(sp.LeafHash), tmhash.Size),
		}
	}
	if len(sp.Aunts) > MaxAunts {
		return ErrMaxAuntsLenExceeded
	}
	for i, auntHash := range sp.Aunts {
		if len(auntHash) != tmhash.Size {
			return ErrInvalidHash{
				Err: fmt.Errorf("aunt#%d hash length %d, want %d", i, len(auntHash), tmhash.Size),
			}
		}
	}
	return nil
}

func (sp *Proof) ToProto() *cmtcrypto.Proof {
	if sp == nil {
		return nil
	}
	pb := new(cmtcrypto.Proof)

	pb.Total = sp.Total
	pb.Index = sp.Index
	pb.LeafHash = sp.LeafHash
	pb.Aunts = sp.Aunts

	return pb
}

func ProofFromProto(pb *cmtcrypto.Proof) (*Proof, error) {
	if pb == nil {
		return nil, ErrInvalidProof{Err: errors.New("nil proof")}
	}

	sp := new(Proof)

	sp.Total = pb.Total
	sp.Index = pb.Index
	sp.LeafHash = pb.LeafHash
	sp.Aunts = pb.Aunts

	return sp, sp.ValidateBasic()
}

// Use the leafHash and innerHashes to get the root merkle hash.
// If the length of the innerHashes slice isn't exactly correct, the result is nil.
// Recursive impl.
func computeHashFromAunts(hash hash.Hash, index, total int64, leafHash []byte, innerHashes [][]byte) ([]byte, error) {
	if index >= total || index < 0 || total <= 0 {
		return nil, fmt.Errorf("invalid index %d and/or total %d", index, total)
	}
	switch total {
	case 0:
		panic("Cannot call computeHashFromAunts() with 0 total")
	case 1:
		if len(innerHashes) != 0 {
			return nil, errors.New("unexpected inner hashes")
		}
		return leafHash, nil
	default:
		if len(innerHashes) == 0 {
			return nil, errors.New("expected at least one inner hash")
		}
		numLeft := getSplitPoint(total)
		if index < numLeft {
			leftHash, err := computeHashFromAunts(hash, index, numLeft, leafHash, innerHashes[:len(innerHashes)-1])
			if err != nil {
				return nil, err
			}

			return innerHashOpt(hash, leftHash, innerHashes[len(innerHashes)-1]), nil
		}
		rightHash, err := computeHashFromAunts(hash, index-numLeft, total-numLeft, leafHash, innerHashes[:len(innerHashes)-1])
		if err != nil {
			return nil, err
		}
		return innerHashOpt(hash, innerHashes[len(innerHashes)-1], rightHash), nil
	}
}

// ProofNode is a helper structure to construct merkle proof.
// The node and the tree is thrown away afterwards.
// Exactly one of node.Left and node.Right is nil, unless node is the root, in which case both are nil.
// node.Parent.Hash = hash(node.Hash, node.Right.Hash) or
// hash(node.Left.Hash, node.Hash), depending on whether node is a left/right child.
type ProofNode struct {
	Hash   []byte
	Parent *ProofNode
	Left   *ProofNode // Left sibling  (only one of Left,Right is set)
	Right  *ProofNode // Right sibling (only one of Left,Right is set)
}

// FlattenAunts will return the inner hashes for the item corresponding to the leaf,
// starting from a leaf ProofNode.
func (spn *ProofNode) FlattenAunts() [][]byte {
	// Nonrecursive impl.
	innerHashes := [][]byte{}
	for spn != nil {
		switch {
		case spn.Left != nil:
			innerHashes = append(innerHashes, spn.Left.Hash)
		case spn.Right != nil:
			innerHashes = append(innerHashes, spn.Right.Hash)
		default:
		}
		spn = spn.Parent
	}
	return innerHashes
}

// trails[0].Hash is the leaf hash for items[0].
// trails[i].Parent.Parent....Parent == root for all i.
func trailsFromByteSlices(items [][]byte) (trails []*ProofNode, root *ProofNode) {
	return trailsFromByteSlicesInternal(tmhash.New(), items)
}

func trailsFromByteSlicesInternal(hash hash.Hash, items [][]byte) (trails []*ProofNode, root *ProofNode) {
	// Recursive impl.
	switch len(items) {
	case 0:
		return []*ProofNode{}, &ProofNode{emptyHash(), nil, nil, nil}
	case 1:
		trail := &ProofNode{leafHashOpt(hash, items[0]), nil, nil, nil}
		return []*ProofNode{trail}, trail
	default:
		k := getSplitPoint(int64(len(items)))
		lefts, leftRoot := trailsFromByteSlicesInternal(hash, items[:k])
		rights, rightRoot := trailsFromByteSlicesInternal(hash, items[k:])
		rootHash := innerHashOpt(hash, leftRoot.Hash, rightRoot.Hash)
		root := &ProofNode{rootHash, nil, nil, nil}
		leftRoot.Parent = root
		leftRoot.Right = rightRoot
		rightRoot.Parent = root
		rightRoot.Left = leftRoot
		return append(lefts, rights...), root
	}
}
