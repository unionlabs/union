package types

import (
	"cosmossdk.io/collections"
)

const (
	// ModuleName is the name of the module
	ModuleName = "gov"

	// StoreKey is the store key string for gov
	StoreKey = ModuleName

	// RouterKey is the message route for gov
	RouterKey = ModuleName
)

var (
	ProposalsKeyPrefix            = collections.NewPrefix(0)  // ProposalsKeyPrefix stores the proposals raw bytes.
	ActiveProposalQueuePrefix     = collections.NewPrefix(1)  // ActiveProposalQueuePrefix stores the active proposals.
	InactiveProposalQueuePrefix   = collections.NewPrefix(2)  // InactiveProposalQueuePrefix stores the inactive proposals.
	ProposalIDKey                 = collections.NewPrefix(3)  // ProposalIDKey stores the sequence representing the next proposal ID.
	VotingPeriodProposalKeyPrefix = collections.NewPrefix(4)  // VotingPeriodProposalKeyPrefix stores which proposals are on voting period.
	DepositsKeyPrefix             = collections.NewPrefix(16) // DepositsKeyPrefix stores deposits.
	VotesKeyPrefix                = collections.NewPrefix(32) // VotesKeyPrefix stores the votes of proposals.
	ParamsKey                     = collections.NewPrefix(48) // ParamsKey stores the module's params.
	ConstitutionKey               = collections.NewPrefix(49) // ConstitutionKey stores a chain's constitution.
)
