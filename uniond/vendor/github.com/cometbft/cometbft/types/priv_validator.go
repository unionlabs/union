package types

import (
	"bytes"
	"errors"
	"fmt"

	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/ed25519"
)

// PrivValidator defines the functionality of a local CometBFT validator
// that signs votes and proposals, and never double signs.
type PrivValidator interface {
	// GetPubKey returns the public key of the validator.
	GetPubKey() (crypto.PubKey, error)

	// FIXME: should use the domain types defined in this package, not the proto types

	// SignVote signs a canonical representation of the vote. If signExtension is
	// true, it also signs the vote extension.
	SignVote(chainID string, vote *cmtproto.Vote, signExtension bool) error

	// SignProposal signs a canonical representation of the proposal.
	SignProposal(chainID string, proposal *cmtproto.Proposal) error

	// SignBytes signs an arbitrary array of bytes.
	SignBytes(bytes []byte) ([]byte, error)
}

type PrivValidatorsByAddress []PrivValidator

func (pvs PrivValidatorsByAddress) Len() int {
	return len(pvs)
}

func (pvs PrivValidatorsByAddress) Less(i, j int) bool {
	pvi, err := pvs[i].GetPubKey()
	if err != nil {
		panic(err)
	}
	pvj, err := pvs[j].GetPubKey()
	if err != nil {
		panic(err)
	}

	return bytes.Compare(pvi.Address(), pvj.Address()) == -1
}

func (pvs PrivValidatorsByAddress) Swap(i, j int) {
	pvs[i], pvs[j] = pvs[j], pvs[i]
}

// ----------------------------------------
// MockPV

// MockPV implements PrivValidator without any safety or persistence.
// Only use it for testing.
type MockPV struct {
	PrivKey              crypto.PrivKey
	breakProposalSigning bool
	breakVoteSigning     bool
}

func NewMockPV() MockPV {
	return MockPV{ed25519.GenPrivKey(), false, false}
}

// NewMockPVWithParams allows one to create a MockPV instance, but with finer
// grained control over the operation of the mock validator. This is useful for
// mocking test failures.
func NewMockPVWithParams(privKey crypto.PrivKey, breakProposalSigning, breakVoteSigning bool) MockPV {
	return MockPV{privKey, breakProposalSigning, breakVoteSigning}
}

// GetPubKey implements PrivValidator.
func (pv MockPV) GetPubKey() (crypto.PubKey, error) {
	return pv.PrivKey.PubKey(), nil
}

// SignVote implements PrivValidator.
func (pv MockPV) SignVote(chainID string, vote *cmtproto.Vote, signExtension bool) error {
	useChainID := chainID
	if pv.breakVoteSigning {
		useChainID = "incorrect-chain-id"
	}

	signBytes := VoteSignBytes(useChainID, vote)
	sig, err := pv.PrivKey.Sign(signBytes)
	if err != nil {
		return err
	}
	vote.Signature = sig

	if signExtension {
		var extSig []byte
		// We only sign vote extensions for non-nil precommits
		if vote.Type == PrecommitType && !ProtoBlockIDIsNil(&vote.BlockID) {
			extSignBytes := VoteExtensionSignBytes(useChainID, vote)
			extSig, err = pv.PrivKey.Sign(extSignBytes)
			if err != nil {
				return err
			}
		} else if len(vote.Extension) > 0 {
			return errors.New("unexpected vote extension - vote extensions are only allowed in non-nil precommits")
		}
		vote.ExtensionSignature = extSig
	}
	return nil
}

// SignProposal implements PrivValidator.
func (pv MockPV) SignProposal(chainID string, proposal *cmtproto.Proposal) error {
	useChainID := chainID
	if pv.breakProposalSigning {
		useChainID = "incorrect-chain-id"
	}

	signBytes := ProposalSignBytes(useChainID, proposal)
	sig, err := pv.PrivKey.Sign(signBytes)
	if err != nil {
		return err
	}
	proposal.Signature = sig
	return nil
}

// SignBytes implements PrivValidator.
func (pv MockPV) SignBytes(bytes []byte) ([]byte, error) {
	return pv.PrivKey.Sign(bytes)
}

func (pv MockPV) ExtractIntoValidator(votingPower int64) *Validator {
	pubKey, _ := pv.GetPubKey()
	return &Validator{
		Address:     pubKey.Address(),
		PubKey:      pubKey,
		VotingPower: votingPower,
	}
}

// String returns a string representation of the MockPV.
func (pv MockPV) String() string {
	mpv, _ := pv.GetPubKey() // mockPV will never return an error, ignored here
	return fmt.Sprintf("MockPV{%v}", mpv.Address())
}

// XXX: Implement.
func (MockPV) DisableChecks() {
	// Currently this does nothing,
	// as MockPV has no safety checks at all.
}

type ErroringMockPV struct {
	MockPV
}

var ErroringMockPVErr = errors.New("erroringMockPV always returns an error")

// SignVote implements PrivValidator.
func (*ErroringMockPV) SignVote(string, *cmtproto.Vote, bool) error {
	return ErroringMockPVErr
}

// SignProposal implements PrivValidator.
func (*ErroringMockPV) SignProposal(string, *cmtproto.Proposal) error {
	return ErroringMockPVErr
}

// NewErroringMockPV returns a MockPV that fails on each signing request. Again, for testing only.

func NewErroringMockPV() *ErroringMockPV {
	return &ErroringMockPV{MockPV{ed25519.GenPrivKey(), false, false}}
}
