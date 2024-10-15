package types

import (
	"fmt"

	cmtprotocrypto "github.com/cometbft/cometbft/api/cometbft/crypto/v1"
	"github.com/cometbft/cometbft/crypto/merkle"
	ics23 "github.com/cosmos/ics23/go"

	errorsmod "cosmossdk.io/errors"
	sdkmaps "cosmossdk.io/store/internal/maps"
	sdkproofs "cosmossdk.io/store/internal/proofs"
)

const (
	ProofOpIAVLCommitment         = "ics23:iavl"
	ProofOpSimpleMerkleCommitment = "ics23:simple"
	ProofOpSMTCommitment          = "ics23:smt"
)

// CommitmentOp implements merkle.ProofOperator by wrapping an ics23 CommitmentProof
// It also contains a Key field to determine which key the proof is proving.
// NOTE: CommitmentProof currently can either be ExistenceProof or NonexistenceProof
//
// Type and Spec are classified by the kind of merkle proof it represents allowing
// the code to be reused by more types. Spec is never on the wire, but mapped from type in the code.
type CommitmentOp struct {
	Type  string
	Spec  *ics23.ProofSpec
	Key   []byte
	Proof *ics23.CommitmentProof
}

var _ merkle.ProofOperator = CommitmentOp{}

func NewIavlCommitmentOp(key []byte, proof *ics23.CommitmentProof) CommitmentOp {
	return CommitmentOp{
		Type:  ProofOpIAVLCommitment,
		Spec:  ics23.IavlSpec,
		Key:   key,
		Proof: proof,
	}
}

func NewSimpleMerkleCommitmentOp(key []byte, proof *ics23.CommitmentProof) CommitmentOp {
	return CommitmentOp{
		Type:  ProofOpSimpleMerkleCommitment,
		Spec:  ics23.TendermintSpec,
		Key:   key,
		Proof: proof,
	}
}

func NewSmtCommitmentOp(key []byte, proof *ics23.CommitmentProof) CommitmentOp {
	return CommitmentOp{
		Type:  ProofOpSMTCommitment,
		Spec:  ics23.SmtSpec,
		Key:   key,
		Proof: proof,
	}
}

// CommitmentOpDecoder takes a merkle.ProofOp and attempts to decode it into a CommitmentOp ProofOperator
// The proofOp.Data is just a marshaled CommitmentProof. The Key of the CommitmentOp is extracted
// from the unmarshalled proof.
func CommitmentOpDecoder(pop cmtprotocrypto.ProofOp) (merkle.ProofOperator, error) {
	var spec *ics23.ProofSpec
	switch pop.Type {
	case ProofOpIAVLCommitment:
		spec = ics23.IavlSpec
	case ProofOpSimpleMerkleCommitment:
		spec = ics23.TendermintSpec
	case ProofOpSMTCommitment:
		spec = ics23.SmtSpec
	default:
		return nil, errorsmod.Wrapf(ErrInvalidProof, "unexpected ProofOp.Type; got %s, want supported ics23 subtypes 'ProofOpSimpleMerkleCommitment', 'ProofOpIAVLCommitment', or 'ProofOpSMTCommitment'", pop.Type)
	}

	proof := &ics23.CommitmentProof{}
	err := proof.Unmarshal(pop.Data)
	if err != nil {
		return nil, err
	}

	op := CommitmentOp{
		Type:  pop.Type,
		Key:   pop.Key,
		Spec:  spec,
		Proof: proof,
	}
	return op, nil
}

func (op CommitmentOp) GetKey() []byte {
	return op.Key
}

// Run takes in a list of arguments and attempts to run the proof op against these arguments
// Returns the root wrapped in [][]byte if the proof op succeeds with given args. If not,
// it will return an error.
//
// CommitmentOp will accept args of length 1 or length 0
// If length 1 args is passed in, then CommitmentOp will attempt to prove the existence of the key
// with the value provided by args[0] using the embedded CommitmentProof and return the CommitmentRoot of the proof
// If length 0 args is passed in, then CommitmentOp will attempt to prove the absence of the key
// in the CommitmentOp and return the CommitmentRoot of the proof
func (op CommitmentOp) Run(args [][]byte) ([][]byte, error) {
	// calculate root from proof
	root, err := op.Proof.Calculate()
	if err != nil {
		return nil, errorsmod.Wrapf(ErrInvalidProof, "could not calculate root for proof: %v", err)
	}
	// Only support an existence proof or nonexistence proof (batch proofs currently unsupported)
	switch len(args) {
	case 0:
		// Args are nil, so we verify the absence of the key.
		absent := ics23.VerifyNonMembership(op.Spec, root, op.Proof, op.Key)
		if !absent {
			return nil, errorsmod.Wrapf(ErrInvalidProof, "proof did not verify absence of key: %s", string(op.Key))
		}

	case 1:
		// Args is length 1, verify existence of key with value args[0]
		if !ics23.VerifyMembership(op.Spec, root, op.Proof, op.Key, args[0]) {
			return nil, errorsmod.Wrapf(ErrInvalidProof, "proof did not verify existence of key %s with given value %x", op.Key, args[0])
		}
	default:
		return nil, errorsmod.Wrapf(ErrInvalidProof, "args must be length 0 or 1, got: %d", len(args))
	}

	return [][]byte{root}, nil
}

// ProofOp implements ProofOperator interface and converts a CommitmentOp
// into a merkle.ProofOp format that can later be decoded by CommitmentOpDecoder
// back into a CommitmentOp for proof verification
func (op CommitmentOp) ProofOp() cmtprotocrypto.ProofOp {
	bz, err := op.Proof.Marshal()
	if err != nil {
		panic(err.Error())
	}
	return cmtprotocrypto.ProofOp{
		Type: op.Type,
		Key:  op.Key,
		Data: bz,
	}
}

// ProofOpFromMap generates a single proof from a map and converts it to a ProofOp.
func ProofOpFromMap(cmap map[string][]byte, storeName string) (ret cmtprotocrypto.ProofOp, err error) {
	_, proofs, _ := sdkmaps.ProofsFromMap(cmap)

	proof := proofs[storeName]
	if proof == nil {
		err = fmt.Errorf("ProofOp for %s but not registered store name", storeName)
		return
	}

	// convert merkle.SimpleProof to CommitmentProof
	existProof, err := sdkproofs.ConvertExistenceProof(proof, []byte(storeName), cmap[storeName])
	if err != nil {
		err = fmt.Errorf("could not convert simple proof to existence proof: %w", err)
		return
	}

	commitmentProof := &ics23.CommitmentProof{
		Proof: &ics23.CommitmentProof_Exist{
			Exist: existProof,
		},
	}

	ret = NewSimpleMerkleCommitmentOp([]byte(storeName), commitmentProof).ProofOp()
	return
}
