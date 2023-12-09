package types

import (
	ics23 "github.com/cosmos/ics23/go"

	errorsmod "cosmossdk.io/errors"

	crypto "github.com/cometbft/cometbft/proto/tendermint/crypto"
)

// ConvertProofs converts crypto.ProofOps into MerkleProof
func ConvertProofs(tmProof *crypto.ProofOps) (MerkleProof, error) {
	if tmProof == nil {
		return MerkleProof{}, errorsmod.Wrapf(ErrInvalidMerkleProof, "tendermint proof is nil")
	}
	// Unmarshal all proof ops to CommitmentProof
	proofs := make([]*ics23.CommitmentProof, len(tmProof.Ops))
	for i, op := range tmProof.Ops {
		var p ics23.CommitmentProof
		err := p.Unmarshal(op.Data)
		if err != nil || p.Proof == nil {
			return MerkleProof{}, errorsmod.Wrapf(ErrInvalidMerkleProof, "could not unmarshal proof op into CommitmentProof at index %d: %v", i, err)
		}
		proofs[i] = &p
	}
	return MerkleProof{
		Proofs: proofs,
	}, nil
}
