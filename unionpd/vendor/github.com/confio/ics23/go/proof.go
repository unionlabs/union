package ics23

import (
	"bytes"
	"errors"
	"fmt"
)

// IavlSpec constrains the format from proofs-iavl (iavl merkle proofs)
var IavlSpec = &ProofSpec{
	LeafSpec: &LeafOp{
		Prefix:       []byte{0},
		PrehashKey:   HashOp_NO_HASH,
		Hash:         HashOp_SHA256,
		PrehashValue: HashOp_SHA256,
		Length:       LengthOp_VAR_PROTO,
	},
	InnerSpec: &InnerSpec{
		ChildOrder:      []int32{0, 1},
		MinPrefixLength: 4,
		MaxPrefixLength: 12,
		ChildSize:       33, // (with length byte)
		EmptyChild:      nil,
		Hash:            HashOp_SHA256,
	},
}

// TendermintSpec constrains the format from proofs-tendermint (crypto/merkle SimpleProof)
var TendermintSpec = &ProofSpec{
	LeafSpec: &LeafOp{
		Prefix:       []byte{0},
		PrehashKey:   HashOp_NO_HASH,
		Hash:         HashOp_SHA256,
		PrehashValue: HashOp_SHA256,
		Length:       LengthOp_VAR_PROTO,
	},
	InnerSpec: &InnerSpec{
		ChildOrder:      []int32{0, 1},
		MinPrefixLength: 1,
		MaxPrefixLength: 1,
		ChildSize:       32, // (no length byte)
		Hash:            HashOp_SHA256,
	},
}

// SmtSpec constrains the format for SMT proofs (as implemented by github.com/celestiaorg/smt)
var SmtSpec = &ProofSpec{
	LeafSpec: &LeafOp{
		Hash:         HashOp_SHA256,
		PrehashKey:   HashOp_NO_HASH,
		PrehashValue: HashOp_SHA256,
		Length:       LengthOp_NO_PREFIX,
		Prefix:       []byte{0},
	},
	InnerSpec: &InnerSpec{
		ChildOrder:      []int32{0, 1},
		ChildSize:       32,
		MinPrefixLength: 1,
		MaxPrefixLength: 1,
		EmptyChild:      make([]byte, 32),
		Hash:            HashOp_SHA256,
	},
	MaxDepth: 256,
}

func encodeVarintProto(l int) []byte {
	// avoid multiple allocs for normal case
	res := make([]byte, 0, 8)
	for l >= 1<<7 {
		res = append(res, uint8(l&0x7f|0x80))
		l >>= 7
	}
	res = append(res, uint8(l))
	return res
}

// Calculate determines the root hash that matches a given Commitment proof
// by type switching and calculating root based on proof type
// NOTE: Calculate will return the first calculated root in the proof,
// you must validate that all other embedded ExistenceProofs commit to the same root.
// This can be done with the Verify method
func (p *CommitmentProof) Calculate() (CommitmentRoot, error) {
	switch v := p.Proof.(type) {
	case *CommitmentProof_Exist:
		return v.Exist.Calculate()
	case *CommitmentProof_Nonexist:
		return v.Nonexist.Calculate()
	case *CommitmentProof_Batch:
		if len(v.Batch.GetEntries()) == 0 || v.Batch.GetEntries()[0] == nil {
			return nil, errors.New("batch proof has empty entry")
		}
		if e := v.Batch.GetEntries()[0].GetExist(); e != nil {
			return e.Calculate()
		}
		if n := v.Batch.GetEntries()[0].GetNonexist(); n != nil {
			return n.Calculate()
		}
	case *CommitmentProof_Compressed:
		proof := Decompress(p)
		return proof.Calculate()
	default:
		return nil, errors.New("unrecognized proof type")
	}
	return nil, errors.New("unrecognized proof type")
}

// Verify does all checks to ensure this proof proves this key, value -> root
// and matches the spec.
func (p *ExistenceProof) Verify(spec *ProofSpec, root CommitmentRoot, key []byte, value []byte) error {
	if err := p.CheckAgainstSpec(spec); err != nil {
		return err
	}

	if !bytes.Equal(key, p.Key) {
		return fmt.Errorf("provided key doesn't match proof")
	}
	if !bytes.Equal(value, p.Value) {
		return fmt.Errorf("provided value doesn't match proof")
	}

	calc, err := p.calculate(spec)
	if err != nil {
		return fmt.Errorf("error calculating root, %w", err)
	}
	if !bytes.Equal(root, calc) {
		return fmt.Errorf("calculcated root doesn't match provided root")
	}

	return nil
}

// Calculate determines the root hash that matches the given proof.
// You must validate the result is what you have in a header.
// Returns error if the calculations cannot be performed.
func (p *ExistenceProof) Calculate() (CommitmentRoot, error) {
	return p.calculate(nil)
}

func (p *ExistenceProof) calculate(spec *ProofSpec) (CommitmentRoot, error) {
	if p.GetLeaf() == nil {
		return nil, errors.New("existence Proof needs defined LeafOp")
	}

	// leaf step takes the key and value as input
	res, err := p.Leaf.Apply(p.Key, p.Value)
	if err != nil {
		return nil, fmt.Errorf("leaf, %w", err)
	}

	// the rest just take the output of the last step (reducing it)
	for _, step := range p.Path {
		res, err = step.Apply(res)
		if err != nil {
			return nil, fmt.Errorf("inner, %w", err)
		}
		if spec != nil {
			if len(res) > int(spec.InnerSpec.ChildSize) && int(spec.InnerSpec.ChildSize) >= 32 {
				return nil, fmt.Errorf("inner, %w", err)
			}
		}
	}
	return res, nil
}

// Calculate determines the root hash that matches the given nonexistence rpoog.
// You must validate the result is what you have in a header.
// Returns error if the calculations cannot be performed.
func (p *NonExistenceProof) Calculate() (CommitmentRoot, error) {
	// A Nonexist proof may have left or right proof nil
	switch {
	case p.Left != nil:
		return p.Left.Calculate()
	case p.Right != nil:
		return p.Right.Calculate()
	default:
		return nil, errors.New("nonexistence proof has empty Left and Right proof")
	}
}

// CheckAgainstSpec will verify the leaf and all path steps are in the format defined in spec
func (p *ExistenceProof) CheckAgainstSpec(spec *ProofSpec) error {
	if p.GetLeaf() == nil {
		return errors.New("existence Proof needs defined LeafOp")
	}
	err := p.Leaf.CheckAgainstSpec(spec)
	if err != nil {
		return fmt.Errorf("leaf, %w", err)
	}
	if spec.MinDepth > 0 && len(p.Path) < int(spec.MinDepth) {
		return fmt.Errorf("innerOps depth too short: %d", len(p.Path))
	}
	if spec.MaxDepth > 0 && len(p.Path) > int(spec.MaxDepth) {
		return fmt.Errorf("innerOps depth too long: %d", len(p.Path))
	}

	layerNum := 1

	for _, inner := range p.Path {
		if err := inner.CheckAgainstSpec(spec, layerNum); err != nil {
			return fmt.Errorf("inner, %w", err)
		}
		layerNum += 1
	}
	return nil
}

// Verify does all checks to ensure the proof has valid non-existence proofs,
// and they ensure the given key is not in the CommitmentState
func (p *NonExistenceProof) Verify(spec *ProofSpec, root CommitmentRoot, key []byte) error {
	// ensure the existence proofs are valid
	var leftKey, rightKey []byte
	if p.Left != nil {
		if err := p.Left.Verify(spec, root, p.Left.Key, p.Left.Value); err != nil {
			return fmt.Errorf("left proof, %w", err)
		}
		leftKey = p.Left.Key
	}
	if p.Right != nil {
		if err := p.Right.Verify(spec, root, p.Right.Key, p.Right.Value); err != nil {
			return fmt.Errorf("right proof, %w", err)
		}
		rightKey = p.Right.Key
	}

	// If both proofs are missing, this is not a valid proof
	if leftKey == nil && rightKey == nil {
		return errors.New("both left and right proofs missing")
	}

	// Ensure in valid range
	if rightKey != nil {
		if bytes.Compare(key, rightKey) >= 0 {
			return errors.New("key is not left of right proof")
		}
	}

	if leftKey != nil {
		if bytes.Compare(key, leftKey) <= 0 {
			return errors.New("key is not right of left proof")
		}
	}

	switch {
	case leftKey == nil:
		if !IsLeftMost(spec.InnerSpec, p.Right.Path) {
			return errors.New("left proof missing, right proof must be left-most")
		}
	case rightKey == nil:
		if !IsRightMost(spec.InnerSpec, p.Left.Path) {
			return errors.New("right proof missing, left proof must be right-most")
		}
	default:
		if !IsLeftNeighbor(spec.InnerSpec, p.Left.Path, p.Right.Path) {
			return errors.New("right proof missing, left proof must be right-most")
		}
	}

	return nil
}

// IsLeftMost returns true if this is the left-most path in the tree, excluding placeholder (empty child) nodes
func IsLeftMost(spec *InnerSpec, path []*InnerOp) bool {
	minPrefix, maxPrefix, suffix := getPadding(spec, 0)

	// ensure every step has a prefix and suffix defined to be leftmost, unless it is a placeholder node
	for _, step := range path {
		if !hasPadding(step, minPrefix, maxPrefix, suffix) && !leftBranchesAreEmpty(spec, step) {
			return false
		}
	}
	return true
}

// IsRightMost returns true if this is the left-most path in the tree, excluding placeholder (empty child) nodes
func IsRightMost(spec *InnerSpec, path []*InnerOp) bool {
	last := len(spec.ChildOrder) - 1
	minPrefix, maxPrefix, suffix := getPadding(spec, int32(last))

	// ensure every step has a prefix and suffix defined to be rightmost, unless it is a placeholder node
	for _, step := range path {
		if !hasPadding(step, minPrefix, maxPrefix, suffix) && !rightBranchesAreEmpty(spec, step) {
			return false
		}
	}
	return true
}

// IsLeftNeighbor returns true if `right` is the next possible path right of `left`
//
//	Find the common suffix from the Left.Path and Right.Path and remove it. We have LPath and RPath now, which must be neighbors.
//	Validate that LPath[len-1] is the left neighbor of RPath[len-1]
//	For step in LPath[0..len-1], validate step is right-most node
//	For step in RPath[0..len-1], validate step is left-most node
func IsLeftNeighbor(spec *InnerSpec, left []*InnerOp, right []*InnerOp) bool {
	// count common tail (from end, near root)
	left, topleft := left[:len(left)-1], left[len(left)-1]
	right, topright := right[:len(right)-1], right[len(right)-1]
	for bytes.Equal(topleft.Prefix, topright.Prefix) && bytes.Equal(topleft.Suffix, topright.Suffix) {
		left, topleft = left[:len(left)-1], left[len(left)-1]
		right, topright = right[:len(right)-1], right[len(right)-1]
	}

	// now topleft and topright are the first divergent nodes
	// make sure they are left and right of each other
	if !isLeftStep(spec, topleft, topright) {
		return false
	}

	// left and right are remaining children below the split,
	// ensure left child is the rightmost path, and visa versa
	if !IsRightMost(spec, left) {
		return false
	}
	if !IsLeftMost(spec, right) {
		return false
	}
	return true
}

// isLeftStep assumes left and right have common parents
// checks if left is exactly one slot to the left of right
func isLeftStep(spec *InnerSpec, left *InnerOp, right *InnerOp) bool {
	leftidx, err := orderFromPadding(spec, left)
	if err != nil {
		panic(err)
	}
	rightidx, err := orderFromPadding(spec, right)
	if err != nil {
		panic(err)
	}

	// TODO: is it possible there are empty (nil) children???
	return rightidx == leftidx+1
}

// checks if an op has the expected padding
func hasPadding(op *InnerOp, minPrefix, maxPrefix, suffix int) bool {
	if len(op.Prefix) < minPrefix {
		return false
	}
	if len(op.Prefix) > maxPrefix {
		return false
	}
	return len(op.Suffix) == suffix
}

// getPadding determines prefix and suffix with the given spec and position in the tree
func getPadding(spec *InnerSpec, branch int32) (minPrefix, maxPrefix, suffix int) {
	idx := getPosition(spec.ChildOrder, branch)

	// count how many children are in the prefix
	prefix := idx * int(spec.ChildSize)
	minPrefix = prefix + int(spec.MinPrefixLength)
	maxPrefix = prefix + int(spec.MaxPrefixLength)

	// count how many children are in the suffix
	suffix = (len(spec.ChildOrder) - 1 - idx) * int(spec.ChildSize)
	return
}

// leftBranchesAreEmpty returns true if the padding bytes correspond to all empty siblings
// on the left side of a branch, ie. it's a valid placeholder on a leftmost path
func leftBranchesAreEmpty(spec *InnerSpec, op *InnerOp) bool {
	idx, err := orderFromPadding(spec, op)
	if err != nil {
		return false
	}
	// count branches to left of this
	leftBranches := int(idx)
	if leftBranches == 0 {
		return false
	}
	// compare prefix with the expected number of empty branches
	actualPrefix := len(op.Prefix) - leftBranches*int(spec.ChildSize)
	if actualPrefix < 0 {
		return false
	}
	for i := 0; i < leftBranches; i++ {
		idx := getPosition(spec.ChildOrder, int32(i))
		from := actualPrefix + idx*int(spec.ChildSize)
		if !bytes.Equal(spec.EmptyChild, op.Prefix[from:from+int(spec.ChildSize)]) {
			return false
		}
	}
	return true
}

// rightBranchesAreEmpty returns true if the padding bytes correspond to all empty siblings
// on the right side of a branch, ie. it's a valid placeholder on a rightmost path
func rightBranchesAreEmpty(spec *InnerSpec, op *InnerOp) bool {
	idx, err := orderFromPadding(spec, op)
	if err != nil {
		return false
	}
	// count branches to right of this one
	rightBranches := len(spec.ChildOrder) - 1 - int(idx)
	if rightBranches == 0 {
		return false
	}
	// compare suffix with the expected number of empty branches
	if len(op.Suffix) != rightBranches*int(spec.ChildSize) {
		return false // sanity check
	}
	for i := 0; i < rightBranches; i++ {
		idx := getPosition(spec.ChildOrder, int32(i))
		from := idx * int(spec.ChildSize)
		if !bytes.Equal(spec.EmptyChild, op.Suffix[from:from+int(spec.ChildSize)]) {
			return false
		}
	}
	return true
}

// getPosition checks where the branch is in the order and returns
// the index of this branch
func getPosition(order []int32, branch int32) int {
	if branch < 0 || int(branch) >= len(order) {
		panic(fmt.Errorf("invalid branch: %d", branch))
	}
	for i, item := range order {
		if branch == item {
			return i
		}
	}
	panic(fmt.Errorf("branch %d not found in order %v", branch, order))
}

// This will look at the proof and determine which order it is...
// So we can see if it is branch 0, 1, 2 etc... to determine neighbors
func orderFromPadding(spec *InnerSpec, inner *InnerOp) (int32, error) {
	maxbranch := int32(len(spec.ChildOrder))
	for branch := int32(0); branch < maxbranch; branch++ {
		minp, maxp, suffix := getPadding(spec, branch)
		if hasPadding(inner, minp, maxp, suffix) {
			return branch, nil
		}
	}
	return 0, errors.New("cannot find any valid spacing for this node")
}

// over-declares equality, which we cosnider fine for now.
func (p *ProofSpec) SpecEquals(spec *ProofSpec) bool {
	return p.LeafSpec.Hash == spec.LeafSpec.Hash &&
		p.LeafSpec.PrehashKey == spec.LeafSpec.PrehashKey &&
		p.LeafSpec.PrehashValue == spec.LeafSpec.PrehashValue &&
		p.LeafSpec.Length == spec.LeafSpec.Length &&
		p.InnerSpec.Hash == spec.InnerSpec.Hash &&
		p.InnerSpec.MinPrefixLength == spec.InnerSpec.MinPrefixLength &&
		p.InnerSpec.MaxPrefixLength == spec.InnerSpec.MaxPrefixLength &&
		p.InnerSpec.ChildSize == spec.InnerSpec.ChildSize &&
		len(p.InnerSpec.ChildOrder) == len(spec.InnerSpec.ChildOrder)
}
