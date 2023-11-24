package iavl

import (
	"encoding/binary"
	"fmt"

	ics23 "github.com/cosmos/ics23/go"
)

/*
GetMembershipProof will produce a CommitmentProof that the given key (and queries value) exists in the iavl tree.
If the key doesn't exist in the tree, this will return an error.
*/
func (t *ImmutableTree) GetMembershipProof(key []byte) (*ics23.CommitmentProof, error) {
	exist, err := t.createExistenceProof(key)
	if err != nil {
		return nil, err
	}
	proof := &ics23.CommitmentProof{
		Proof: &ics23.CommitmentProof_Exist{
			Exist: exist,
		},
	}
	return proof, nil
}

// VerifyMembership returns true iff proof is an ExistenceProof for the given key.
func (t *ImmutableTree) VerifyMembership(proof *ics23.CommitmentProof, key []byte) (bool, error) {
	val, err := t.Get(key)
	if err != nil {
		return false, err
	}
	root := t.Hash()

	return ics23.VerifyMembership(ics23.IavlSpec, root, proof, key, val), nil
}

/*
GetNonMembershipProof will produce a CommitmentProof that the given key doesn't exist in the iavl tree.
If the key exists in the tree, this will return an error.
*/
func (t *ImmutableTree) GetNonMembershipProof(key []byte) (*ics23.CommitmentProof, error) {
	// idx is one node right of what we want....
	var err error
	idx, val, err := t.GetWithIndex(key)
	if err != nil {
		return nil, err
	}

	if val != nil {
		return nil, fmt.Errorf("cannot create NonExistanceProof when Key in State")
	}

	nonexist := &ics23.NonExistenceProof{
		Key: key,
	}

	if idx >= 1 {
		leftkey, _, err := t.GetByIndex(idx - 1)
		if err != nil {
			return nil, err
		}

		nonexist.Left, err = t.createExistenceProof(leftkey)
		if err != nil {
			return nil, err
		}
	}

	// this will be nil if nothing right of the queried key
	rightkey, _, err := t.GetByIndex(idx)
	if err != nil {
		return nil, err
	}

	if rightkey != nil {
		nonexist.Right, err = t.createExistenceProof(rightkey)
		if err != nil {
			return nil, err
		}
	}

	proof := &ics23.CommitmentProof{
		Proof: &ics23.CommitmentProof_Nonexist{
			Nonexist: nonexist,
		},
	}
	return proof, nil
}

// VerifyNonMembership returns true iff proof is a NonExistenceProof for the given key.
func (t *ImmutableTree) VerifyNonMembership(proof *ics23.CommitmentProof, key []byte) (bool, error) {
	root := t.Hash()

	return ics23.VerifyNonMembership(ics23.IavlSpec, root, proof, key), nil
}

// createExistenceProof will get the proof from the tree and convert the proof into a valid
// existence proof, if that's what it is.
func (t *ImmutableTree) createExistenceProof(key []byte) (*ics23.ExistenceProof, error) {
	t.Hash()
	path, node, err := t.root.PathToLeaf(t, key, t.version+1)
	nodeVersion := t.version + 1
	if node.nodeKey != nil {
		nodeVersion = node.nodeKey.version
	}
	return &ics23.ExistenceProof{
		Key:   node.key,
		Value: node.value,
		Leaf:  convertLeafOp(nodeVersion),
		Path:  convertInnerOps(path),
	}, err
}

func convertLeafOp(version int64) *ics23.LeafOp {
	var varintBuf [binary.MaxVarintLen64]byte
	// this is adapted from iavl/proof.go:proofLeafNode.Hash()
	prefix := convertVarIntToBytes(0, varintBuf)
	prefix = append(prefix, convertVarIntToBytes(1, varintBuf)...)
	prefix = append(prefix, convertVarIntToBytes(version, varintBuf)...)

	return &ics23.LeafOp{
		Hash:         ics23.HashOp_SHA256,
		PrehashValue: ics23.HashOp_SHA256,
		Length:       ics23.LengthOp_VAR_PROTO,
		Prefix:       prefix,
	}
}

// we cannot get the proofInnerNode type, so we need to do the whole path in one function
func convertInnerOps(path PathToLeaf) []*ics23.InnerOp {
	steps := make([]*ics23.InnerOp, 0, len(path))

	// lengthByte is the length prefix prepended to each of the sha256 sub-hashes
	var lengthByte byte = 0x20

	var varintBuf [binary.MaxVarintLen64]byte

	// we need to go in reverse order, iavl starts from root to leaf,
	// we want to go up from the leaf to the root
	for i := len(path) - 1; i >= 0; i-- {
		// this is adapted from iavl/proof.go:proofInnerNode.Hash()
		prefix := convertVarIntToBytes(int64(path[i].Height), varintBuf)
		prefix = append(prefix, convertVarIntToBytes(path[i].Size, varintBuf)...)
		prefix = append(prefix, convertVarIntToBytes(path[i].Version, varintBuf)...)

		var suffix []byte
		if len(path[i].Left) > 0 {
			// length prefixed left side
			prefix = append(prefix, lengthByte)
			prefix = append(prefix, path[i].Left...)
			// prepend the length prefix for child
			prefix = append(prefix, lengthByte)
		} else {
			// prepend the length prefix for child
			prefix = append(prefix, lengthByte)
			// length-prefixed right side
			suffix = []byte{lengthByte}
			suffix = append(suffix, path[i].Right...)
		}

		op := &ics23.InnerOp{
			Hash:   ics23.HashOp_SHA256,
			Prefix: prefix,
			Suffix: suffix,
		}
		steps = append(steps, op)
	}
	return steps
}

func convertVarIntToBytes(orig int64, buf [binary.MaxVarintLen64]byte) []byte {
	n := binary.PutVarint(buf[:], orig)
	return buf[:n]
}

// GetProof gets the proof for the given key.
func (t *ImmutableTree) GetProof(key []byte) (*ics23.CommitmentProof, error) {
	if t.root == nil {
		return nil, fmt.Errorf("cannot generate the proof with nil root")
	}

	exist, err := t.Has(key)
	if err != nil {
		return nil, err
	}

	if exist {
		return t.GetMembershipProof(key)
	}
	return t.GetNonMembershipProof(key)
}

// VerifyProof checks if the proof is correct for the given key.
func (t *ImmutableTree) VerifyProof(proof *ics23.CommitmentProof, key []byte) (bool, error) {
	if proof.GetExist() != nil {
		return t.VerifyMembership(proof, key)
	}
	return t.VerifyNonMembership(proof, key)
}

// GetVersionedProof gets the proof for the given key at the specified version.
func (tree *MutableTree) GetVersionedProof(key []byte, version int64) (*ics23.CommitmentProof, error) {
	if tree.VersionExists(version) {
		t, err := tree.GetImmutable(version)
		if err != nil {
			return nil, err
		}
		return t.GetProof(key)
	}
	return nil, ErrVersionDoesNotExist
}
