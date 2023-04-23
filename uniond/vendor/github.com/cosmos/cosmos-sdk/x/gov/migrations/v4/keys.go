package v4

import "encoding/binary"

const (
	// ModuleName is the name of the module
	ModuleName = "gov"
)

var (
	// ParamsKey is the key of x/gov params
	ParamsKey = []byte{0x30}

	// - 0x04<proposalID_Bytes>: ProposalContents
	VotingPeriodProposalKeyPrefix = []byte{0x04}
)

// VotingPeriodProposalKey gets if a proposal is in voting period.
func VotingPeriodProposalKey(proposalID uint64) []byte {
	return append(VotingPeriodProposalKeyPrefix, GetProposalIDBytes(proposalID)...)
}

// GetProposalIDBytes returns the byte representation of the proposalID
func GetProposalIDBytes(proposalID uint64) (proposalIDBz []byte) {
	proposalIDBz = make([]byte, 8)
	binary.BigEndian.PutUint64(proposalIDBz, proposalID)
	return
}
