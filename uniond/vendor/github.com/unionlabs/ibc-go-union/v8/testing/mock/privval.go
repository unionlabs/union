package mock

import (
	"errors"

	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"

	"github.com/cometbft/cometbft/crypto"
	tmproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	tmtypes "github.com/cometbft/cometbft/types"
)

var _ tmtypes.PrivValidator = PV{}

// MockPV implements PrivValidator without any safety or persistence.
// Only use it for testing.
type PV struct {
	PrivKey cryptotypes.PrivKey
}

func NewPV() PV {
	return PV{ed25519.GenPrivKey()}
}

// GetPubKey implements PrivValidator interface
func (pv PV) GetPubKey() (crypto.PubKey, error) {
	return cryptocodec.ToCmtPubKeyInterface(pv.PrivKey.PubKey())
}

// SignVote implements PrivValidator interface
func (pv PV) SignVote(chainID string, vote *tmproto.Vote) error {
	signBytes := tmtypes.VoteSignBytes(chainID, vote)
	sig, err := pv.PrivKey.Sign(signBytes)
	if err != nil {
		return err
	}
	vote.Signature = sig

	var extSig []byte
	// We only sign vote extensions for non-nil precommits
	if vote.Type == tmproto.PrecommitType && !tmtypes.ProtoBlockIDIsNil(&vote.BlockID) {
		extSignBytes := tmtypes.VoteExtensionSignBytes(chainID, vote)
		extSig, err = pv.PrivKey.Sign(extSignBytes)
		if err != nil {
			return err
		}
	} else if len(vote.Extension) > 0 {
		return errors.New("unexpected vote extension - vote extensions are only allowed in non-nil precommits")
	}
	vote.ExtensionSignature = extSig
	return nil
}

// SignProposal implements PrivValidator interface
func (pv PV) SignProposal(chainID string, proposal *tmproto.Proposal) error {
	signBytes := tmtypes.ProposalSignBytes(chainID, proposal)
	sig, err := pv.PrivKey.Sign(signBytes)
	if err != nil {
		return err
	}
	proposal.Signature = sig
	return nil
}
