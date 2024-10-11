package types

import (
	"errors"
	"fmt"
	"time"

	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	cmtbytes "github.com/cometbft/cometbft/libs/bytes"
	"github.com/cometbft/cometbft/libs/protoio"
	cmttime "github.com/cometbft/cometbft/types/time"
)

var (
	ErrInvalidBlockPartSignature = errors.New("error invalid block part signature")
	ErrInvalidBlockPartHash      = errors.New("error invalid block part hash")
)

// Proposal defines a block proposal for the consensus.
// It refers to the block by BlockID field.
// It must be signed by the correct proposer for the given Height/Round
// to be considered valid. It may depend on votes from a previous round,
// a so-called Proof-of-Lock (POL) round, as noted in the POLRound.
// If POLRound >= 0, then BlockID corresponds to the block that is locked in POLRound.
type Proposal struct {
	Type      SignedMsgType
	Height    int64     `json:"height"`
	Round     int32     `json:"round"`     // there can not be greater than 2_147_483_647 rounds
	POLRound  int32     `json:"pol_round"` // -1 if null.
	BlockID   BlockID   `json:"block_id"`
	Timestamp time.Time `json:"timestamp"`
	Signature []byte    `json:"signature"`
}

// NewProposal returns a new Proposal.
// If there is no POLRound, polRound should be -1.
func NewProposal(height int64, round int32, polRound int32, blockID BlockID, ts time.Time) *Proposal {
	return &Proposal{
		Type:      ProposalType,
		Height:    height,
		Round:     round,
		BlockID:   blockID,
		POLRound:  polRound,
		Timestamp: cmttime.Canonical(ts),
	}
}

// ValidateBasic performs basic validation.
func (p *Proposal) ValidateBasic() error {
	if p.Type != ProposalType {
		return errors.New("invalid Type")
	}
	if p.Height <= 0 {
		return errors.New("non positive Height")
	}
	if p.Round < 0 {
		return errors.New("negative Round")
	}
	if p.POLRound < -1 {
		return errors.New("negative POLRound (exception: -1)")
	}
	if p.POLRound >= p.Round {
		return errors.New("POLRound >= Round")
	}
	if err := p.BlockID.ValidateBasic(); err != nil {
		return fmt.Errorf("wrong BlockID: %w", err)
	}
	// ValidateBasic above would pass even if the BlockID was empty:
	if !p.BlockID.IsComplete() {
		return fmt.Errorf("expected a complete, non-empty BlockID, got: %v", p.BlockID)
	}
	// Times must be canonical
	if cmttime.Canonical(p.Timestamp) != p.Timestamp {
		return fmt.Errorf("expected a canonical timestamp, got: %v", p.Timestamp)
	}
	if len(p.Signature) == 0 {
		return errors.New("signature is missing")
	}
	if len(p.Signature) > MaxSignatureSize {
		return fmt.Errorf("signature is too big (max: %d)", MaxSignatureSize)
	}
	return nil
}

// IsTimely validates that the proposal timestamp is 'timely' according to the
// proposer-based timestamp algorithm. To evaluate if a proposal is timely, its
// timestamp is compared to the local time of the validator when it receives
// the proposal along with the configured Precision and MessageDelay
// parameters. Specifically, a proposed proposal timestamp is considered timely
// if it is satisfies the following inequalities:
//
// proposalReceiveTime >= proposalTimestamp - Precision
// proposalReceiveTime <= proposalTimestamp + MessageDelay + Precision
//
// For more information on the meaning of 'timely', refer to the specification:
// https://github.com/cometbft/cometbft/tree/main/spec/consensus/proposer-based-timestamp
func (p *Proposal) IsTimely(recvTime time.Time, sp SynchronyParams) bool {
	// lhs is `proposalTimestamp - Precision` in the first inequality
	lhs := p.Timestamp.Add(-sp.Precision)
	// rhs is `proposalTimestamp + MessageDelay + Precision` in the second inequality
	rhs := p.Timestamp.Add(sp.MessageDelay).Add(sp.Precision)

	if recvTime.Before(lhs) || recvTime.After(rhs) {
		return false
	}
	return true
}

// String returns a string representation of the Proposal.
//
// 1. height
// 2. round
// 3. block ID
// 4. POL round
// 5. first 6 bytes of signature
// 6. timestamp
//
// See BlockID#String.
func (p *Proposal) String() string {
	return fmt.Sprintf("Proposal{%v/%v (%v, %v) %X @ %s}",
		p.Height,
		p.Round,
		p.BlockID,
		p.POLRound,
		cmtbytes.Fingerprint(p.Signature),
		CanonicalTime(p.Timestamp))
}

// ProposalSignBytes returns the proto-encoding of the canonicalized Proposal,
// for signing. Panics if the marshaling fails.
//
// The encoded Protobuf message is varint length-prefixed (using MarshalDelimited)
// for backwards-compatibility with the Amino encoding, due to e.g. hardware
// devices that rely on this encoding.
//
// See CanonicalizeProposal.
func ProposalSignBytes(chainID string, p *cmtproto.Proposal) []byte {
	pb := CanonicalizeProposal(chainID, p)
	bz, err := protoio.MarshalDelimited(&pb)
	if err != nil {
		panic(err)
	}

	return bz
}

// ToProto converts Proposal to protobuf.
func (p *Proposal) ToProto() *cmtproto.Proposal {
	if p == nil {
		return &cmtproto.Proposal{}
	}
	pb := new(cmtproto.Proposal)

	pb.BlockID = p.BlockID.ToProto()
	pb.Type = p.Type
	pb.Height = p.Height
	pb.Round = p.Round
	pb.PolRound = p.POLRound // FIXME: names do not match
	pb.Timestamp = p.Timestamp
	pb.Signature = p.Signature

	return pb
}

// ProposalFromProto sets a protobuf Proposal to the given pointer.
// It returns an error if the proposal is invalid.
func ProposalFromProto(pp *cmtproto.Proposal) (*Proposal, error) {
	if pp == nil {
		return nil, errors.New("nil proposal")
	}

	p := new(Proposal)

	blockID, err := BlockIDFromProto(&pp.BlockID)
	if err != nil {
		return nil, err
	}

	p.BlockID = *blockID
	p.Type = pp.Type
	p.Height = pp.Height
	p.Round = pp.Round
	p.POLRound = pp.PolRound // FIXME: names do not match
	p.Timestamp = pp.Timestamp
	p.Signature = pp.Signature

	return p, p.ValidateBasic()
}
