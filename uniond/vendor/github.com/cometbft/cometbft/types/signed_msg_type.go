package types

import cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"

type SignedMsgType = cmtproto.SignedMsgType

const (
	UnknownType   SignedMsgType = cmtproto.UnknownType
	PrevoteType   SignedMsgType = cmtproto.PrevoteType
	PrecommitType SignedMsgType = cmtproto.PrecommitType
	ProposalType  SignedMsgType = cmtproto.ProposalType
)

// IsVoteTypeValid returns true if t is a valid vote type.
func IsVoteTypeValid(t SignedMsgType) bool {
	switch t {
	case PrevoteType, PrecommitType:
		return true
	default:
		return false
	}
}

var signedMsgTypeToShortName = map[SignedMsgType]string{
	UnknownType:   "unknown",
	PrevoteType:   "prevote",
	PrecommitType: "precommit",
	ProposalType:  "proposal",
}

// Returns a short lowercase descriptor for a signed message type.
func SignedMsgTypeToShortString(t SignedMsgType) string {
	if shortName, ok := signedMsgTypeToShortName[t]; ok {
		return shortName
	}
	return "unknown"
}
