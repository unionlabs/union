package types

import (
	"time"

	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	cmttime "github.com/cometbft/cometbft/types/time"
)

// Canonical* wraps the structs in types for amino encoding them for use in SignBytes / the Signable interface.

// TimeFormat is used for generating the sigs.
const TimeFormat = time.RFC3339Nano

// -----------------------------------
// Canonicalize the structs

func CanonicalizeBlockID(bid cmtproto.BlockID) *cmtproto.CanonicalBlockID {
	rbid, err := BlockIDFromProto(&bid)
	if err != nil {
		panic(err)
	}
	var cbid *cmtproto.CanonicalBlockID
	if rbid == nil || rbid.IsNil() {
		cbid = nil
	} else {
		cbid = &cmtproto.CanonicalBlockID{
			Hash:          bid.Hash,
			PartSetHeader: CanonicalizePartSetHeader(bid.PartSetHeader),
		}
	}

	return cbid
}

// CanonicalizePartSetHeader transforms the given PartSetHeader to a CanonicalPartSetHeader.
func CanonicalizePartSetHeader(psh cmtproto.PartSetHeader) cmtproto.CanonicalPartSetHeader {
	return cmtproto.CanonicalPartSetHeader(psh)
}

// CanonicalizeProposal transforms the given Proposal to a CanonicalProposal.
func CanonicalizeProposal(chainID string, proposal *cmtproto.Proposal) cmtproto.CanonicalProposal {
	return cmtproto.CanonicalProposal{
		Type:      ProposalType,
		Height:    proposal.Height,          // encoded as sfixed64
		Round:     int64(proposal.Round),    // encoded as sfixed64
		POLRound:  int64(proposal.PolRound), // FIXME: not matching
		BlockID:   CanonicalizeBlockID(proposal.BlockID),
		Timestamp: proposal.Timestamp,
		ChainID:   chainID,
	}
}

// CanonicalizeVote transforms the given Vote to a CanonicalVote, which does
// not contain ValidatorIndex and ValidatorAddress fields, or any fields
// relating to vote extensions.
func CanonicalizeVote(chainID string, vote *cmtproto.Vote) cmtproto.CanonicalVote {
	return cmtproto.CanonicalVote{
		Type:      vote.Type,
		Height:    vote.Height,       // encoded as sfixed64
		Round:     int64(vote.Round), // encoded as sfixed64
		BlockID:   CanonicalizeBlockID(vote.BlockID),
		ChainID:   chainID,
	}
}

// CanonicalizeVoteExtension extracts the vote extension from the given vote
// and constructs a CanonicalizeVoteExtension struct, whose representation in
// bytes is what is signed in order to produce the vote extension's signature.
func CanonicalizeVoteExtension(chainID string, vote *cmtproto.Vote) cmtproto.CanonicalVoteExtension {
	return cmtproto.CanonicalVoteExtension{
		Extension: vote.Extension,
		Height:    vote.Height,
		Round:     int64(vote.Round),
		ChainId:   chainID,
	}
}

// CanonicalTime can be used to stringify time in a canonical way.
func CanonicalTime(t time.Time) string {
	// Note that sending time over amino resets it to
	// local time, we need to force UTC here, so the
	// signatures match
	return cmttime.Canonical(t).Format(TimeFormat)
}
