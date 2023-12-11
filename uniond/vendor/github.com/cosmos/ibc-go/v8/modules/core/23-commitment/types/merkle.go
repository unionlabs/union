package types

import (
	"bytes"
	"fmt"

	"github.com/cosmos/gogoproto/proto"
	ics23 "github.com/cosmos/ics23/go"

	errorsmod "cosmossdk.io/errors"

	tmcrypto "github.com/cometbft/cometbft/proto/tendermint/crypto"

	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// var representing the proofspecs for a SDK chain
var sdkSpecs = []*ics23.ProofSpec{ics23.IavlSpec, ics23.TendermintSpec}

// ICS 023 Merkle Types Implementation
//
// This file defines Merkle commitment types that implements ICS 023.

// Merkle proof implementation of the Proof interface
// Applied on SDK-based IBC implementation
var _ exported.Root = (*MerkleRoot)(nil)

// GetSDKSpecs is a getter function for the proofspecs of an sdk chain
func GetSDKSpecs() []*ics23.ProofSpec {
	return sdkSpecs
}

// NewMerkleRoot constructs a new MerkleRoot
func NewMerkleRoot(hash []byte) MerkleRoot {
	return MerkleRoot{
		Hash: hash,
	}
}

// GetHash implements RootI interface
func (mr MerkleRoot) GetHash() []byte {
	return mr.Hash
}

// Empty returns true if the root is empty
func (mr MerkleRoot) Empty() bool {
	return len(mr.GetHash()) == 0
}

var _ exported.Prefix = (*MerklePrefix)(nil)

// NewMerklePrefix constructs new MerklePrefix instance
func NewMerklePrefix(keyPrefix []byte) MerklePrefix {
	return MerklePrefix{
		KeyPrefix: keyPrefix,
	}
}

// Bytes returns the key prefix bytes
func (mp MerklePrefix) Bytes() []byte {
	return mp.KeyPrefix
}

// Empty returns true if the prefix is empty
func (mp MerklePrefix) Empty() bool {
	return len(mp.Bytes()) == 0
}

var _ exported.Path = (*MerklePath)(nil)

// NewMerklePath creates a new MerklePath instance
// The keys must be passed in from root-to-leaf order
func NewMerklePath(keyPath ...string) MerklePath {
	return MerklePath{
		KeyPath: keyPath,
	}
}

// GetKey will return a byte representation of the key
func (mp MerklePath) GetKey(i uint64) ([]byte, error) {
	if i >= uint64(len(mp.KeyPath)) {
		return nil, fmt.Errorf("index out of range. %d (index) >= %d (len)", i, len(mp.KeyPath))
	}
	return []byte(mp.KeyPath[i]), nil
}

// Empty returns true if the path is empty
func (mp MerklePath) Empty() bool {
	return len(mp.KeyPath) == 0
}

// ApplyPrefix constructs a new commitment path from the arguments. It prepends the prefix key
// with the given path.
func ApplyPrefix(prefix exported.Prefix, path MerklePath) (MerklePath, error) {
	if prefix == nil || prefix.Empty() {
		return MerklePath{}, errorsmod.Wrap(ErrInvalidPrefix, "prefix can't be empty")
	}
	return NewMerklePath(append([]string{string(prefix.Bytes())}, path.KeyPath...)...), nil
}

var _ exported.Proof = (*MerkleProof)(nil)

// VerifyMembership verifies the membership of a merkle proof against the given root, path, and value.
// Note that the path is expected as []string{<store key of module>, <key corresponding to requested value>}.
func (proof MerkleProof) VerifyMembership(specs []*ics23.ProofSpec, root exported.Root, path exported.Path, value []byte) error {
	if err := proof.validateVerificationArgs(specs, root); err != nil {
		return err
	}

	// VerifyMembership specific argument validation
	mpath, ok := path.(MerklePath)
	if !ok {
		return errorsmod.Wrapf(ErrInvalidProof, "path %v is not of type MerklePath", path)
	}
	if len(mpath.KeyPath) != len(specs) {
		return errorsmod.Wrapf(ErrInvalidProof, "path length %d not same as proof %d",
			len(mpath.KeyPath), len(specs))
	}
	if len(value) == 0 {
		return errorsmod.Wrap(ErrInvalidProof, "empty value in membership proof")
	}

	// Since every proof in chain is a membership proof we can use verifyChainedMembershipProof from index 0
	// to validate entire proof
	return verifyChainedMembershipProof(root.GetHash(), specs, proof.Proofs, mpath, value, 0)
}

// VerifyNonMembership verifies the absence of a merkle proof against the given root and path.
// VerifyNonMembership verifies a chained proof where the absence of a given path is proven
// at the lowest subtree and then each subtree's inclusion is proved up to the final root.
func (proof MerkleProof) VerifyNonMembership(specs []*ics23.ProofSpec, root exported.Root, path exported.Path) error {
	if err := proof.validateVerificationArgs(specs, root); err != nil {
		return err
	}

	// VerifyNonMembership specific argument validation
	mpath, ok := path.(MerklePath)
	if !ok {
		return errorsmod.Wrapf(ErrInvalidProof, "path %v is not of type MerkleProof", path)
	}
	if len(mpath.KeyPath) != len(specs) {
		return errorsmod.Wrapf(ErrInvalidProof, "path length %d not same as proof %d",
			len(mpath.KeyPath), len(specs))
	}

	switch proof.Proofs[0].Proof.(type) {
	case *ics23.CommitmentProof_Nonexist:
		// VerifyNonMembership will verify the absence of key in lowest subtree, and then chain inclusion proofs
		// of all subroots up to final root
		subroot, err := proof.Proofs[0].Calculate()
		if err != nil {
			return errorsmod.Wrapf(ErrInvalidProof, "could not calculate root for proof index 0, merkle tree is likely empty. %v", err)
		}
		key, err := mpath.GetKey(uint64(len(mpath.KeyPath) - 1))
		if err != nil {
			return errorsmod.Wrapf(ErrInvalidProof, "could not retrieve key bytes for key: %s", mpath.KeyPath[len(mpath.KeyPath)-1])
		}
		if ok := ics23.VerifyNonMembership(specs[0], subroot, proof.Proofs[0], key); !ok {
			return errorsmod.Wrapf(ErrInvalidProof, "could not verify absence of key %s. Please ensure that the path is correct.", string(key))
		}

		// Verify chained membership proof starting from index 1 with value = subroot
		if err := verifyChainedMembershipProof(root.GetHash(), specs, proof.Proofs, mpath, subroot, 1); err != nil {
			return err
		}
	case *ics23.CommitmentProof_Exist:
		return errorsmod.Wrapf(ErrInvalidProof,
			"got ExistenceProof in VerifyNonMembership. If this is unexpected, please ensure that proof was queried with the correct key.")
	default:
		return errorsmod.Wrapf(ErrInvalidProof,
			"expected proof type: %T, got: %T", &ics23.CommitmentProof_Exist{}, proof.Proofs[0].Proof)
	}
	return nil
}

// BatchVerifyMembership verifies a group of key value pairs against the given root
// NOTE: Currently left unimplemented as it is unused
func (MerkleProof) BatchVerifyMembership(specs []*ics23.ProofSpec, root exported.Root, path exported.Path, items map[string][]byte) error {
	return errorsmod.Wrap(ErrInvalidProof, "batch proofs are currently unsupported")
}

// BatchVerifyNonMembership verifies absence of a group of keys against the given root
// NOTE: Currently left unimplemented as it is unused
func (MerkleProof) BatchVerifyNonMembership(specs []*ics23.ProofSpec, root exported.Root, path exported.Path, items [][]byte) error {
	return errorsmod.Wrap(ErrInvalidProof, "batch proofs are currently unsupported")
}

// verifyChainedMembershipProof takes a list of proofs and specs and verifies each proof sequentially ensuring that the value is committed to
// by first proof and each subsequent subroot is committed to by the next subroot and checking that the final calculated root is equal to the given roothash.
// The proofs and specs are passed in from lowest subtree to the highest subtree, but the keys are passed in from highest subtree to lowest.
// The index specifies what index to start chaining the membership proofs, this is useful since the lowest proof may not be a membership proof, thus we
// will want to start the membership proof chaining from index 1 with value being the lowest subroot
func verifyChainedMembershipProof(root []byte, specs []*ics23.ProofSpec, proofs []*ics23.CommitmentProof, keys MerklePath, value []byte, index int) error {
	var (
		subroot []byte
		err     error
	)
	// Initialize subroot to value since the proofs list may be empty.
	// This may happen if this call is verifying intermediate proofs after the lowest proof has been executed.
	// In this case, there may be no intermediate proofs to verify and we just check that lowest proof root equals final root
	subroot = value
	for i := index; i < len(proofs); i++ {
		switch proofs[i].Proof.(type) {
		case *ics23.CommitmentProof_Exist:
			subroot, err = proofs[i].Calculate()
			if err != nil {
				return errorsmod.Wrapf(ErrInvalidProof, "could not calculate proof root at index %d, merkle tree may be empty. %v", i, err)
			}
			// Since keys are passed in from highest to lowest, we must grab their indices in reverse order
			// from the proofs and specs which are lowest to highest
			key, err := keys.GetKey(uint64(len(keys.KeyPath) - 1 - i))
			if err != nil {
				return errorsmod.Wrapf(ErrInvalidProof, "could not retrieve key bytes for key %s: %v", keys.KeyPath[len(keys.KeyPath)-1-i], err)
			}

			// verify membership of the proof at this index with appropriate key and value
			if ok := ics23.VerifyMembership(specs[i], subroot, proofs[i], key, value); !ok {
				return errorsmod.Wrapf(ErrInvalidProof,
					"chained membership proof failed to verify membership of value: %X in subroot %X at index %d. Please ensure the path and value are both correct.",
					value, subroot, i)
			}
			// Set value to subroot so that we verify next proof in chain commits to this subroot
			value = subroot
		case *ics23.CommitmentProof_Nonexist:
			return errorsmod.Wrapf(ErrInvalidProof,
				"chained membership proof contains nonexistence proof at index %d. If this is unexpected, please ensure that proof was queried from a height that contained the value in store and was queried with the correct key. The key used: %s",
				i, keys)
		default:
			return errorsmod.Wrapf(ErrInvalidProof,
				"expected proof type: %T, got: %T", &ics23.CommitmentProof_Exist{}, proofs[i].Proof)
		}
	}
	// Check that chained proof root equals passed-in root
	if !bytes.Equal(root, subroot) {
		return errorsmod.Wrapf(ErrInvalidProof,
			"proof did not commit to expected root: %X, got: %X. Please ensure proof was submitted with correct proofHeight and to the correct chain.",
			root, subroot)
	}
	return nil
}

// blankMerkleProof and blankProofOps will be used to compare against their zero values,
// and are declared as globals to avoid having to unnecessarily re-allocate on every comparison.
var (
	blankMerkleProof = &MerkleProof{}
	blankProofOps    = &tmcrypto.ProofOps{}
)

// Empty returns true if the root is empty
func (proof *MerkleProof) Empty() bool {
	return proof == nil || proto.Equal(proof, blankMerkleProof) || proto.Equal(proof, blankProofOps)
}

// ValidateBasic checks if the proof is empty.
func (proof MerkleProof) ValidateBasic() error {
	if proof.Empty() {
		return ErrInvalidProof
	}
	return nil
}

// validateVerificationArgs verifies the proof arguments are valid
func (proof MerkleProof) validateVerificationArgs(specs []*ics23.ProofSpec, root exported.Root) error {
	if proof.Empty() {
		return errorsmod.Wrap(ErrInvalidMerkleProof, "proof cannot be empty")
	}

	if root == nil || root.Empty() {
		return errorsmod.Wrap(ErrInvalidMerkleProof, "root cannot be empty")
	}

	if len(specs) != len(proof.Proofs) {
		return errorsmod.Wrapf(ErrInvalidMerkleProof,
			"length of specs: %d not equal to length of proof: %d",
			len(specs), len(proof.Proofs))
	}

	for i, spec := range specs {
		if spec == nil {
			return errorsmod.Wrapf(ErrInvalidProof, "spec at position %d is nil", i)
		}
	}
	return nil
}
