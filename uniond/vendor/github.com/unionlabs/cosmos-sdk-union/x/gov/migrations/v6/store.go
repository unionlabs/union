package v6

import (
	"context"
	"fmt"

	"cosmossdk.io/collections"
	corestoretypes "cosmossdk.io/core/store"
	v1 "cosmossdk.io/x/gov/types/v1"
)

var votingPeriodProposalKeyPrefix = collections.NewPrefix(4) // VotingPeriodProposalKeyPrefix stores which proposals are on voting period.

// MigrateStore performs in-place store migrations from v5 (v0.50) to v6 (v0.52). The
// migration includes:
//
// Addition of new field in params to store types of proposals that can be submitted.
// Addition of gov params for optimistic proposals.
// Addition of gov params for proposal cancel max period.
// Cleanup of old proposal stores.
func MigrateStore(ctx context.Context, storeService corestoretypes.KVStoreService, paramsCollection collections.Item[v1.Params], proposalCollection collections.Map[uint64, v1.Proposal]) error {
	// Migrate **all** proposals
	err := proposalCollection.Walk(ctx, nil, func(key uint64, proposal v1.Proposal) (bool, error) {
		if proposal.Expedited {
			proposal.ProposalType = v1.ProposalType_PROPOSAL_TYPE_EXPEDITED
		} else {
			proposal.ProposalType = v1.ProposalType_PROPOSAL_TYPE_STANDARD
		}

		if err := proposalCollection.Set(ctx, key, proposal); err != nil {
			return false, err
		}

		return false, nil
	})
	if err != nil {
		return err
	}

	// Clear old proposal stores
	sb := collections.NewSchemaBuilder(storeService)
	votingPeriodProposals := collections.NewMap(sb, votingPeriodProposalKeyPrefix, "voting_period_proposals", collections.Uint64Key, collections.BytesValue)
	if err := votingPeriodProposals.Clear(ctx, nil); err != nil {
		return err
	}

	// Migrate params
	govParams, err := paramsCollection.Get(ctx)
	if err != nil {
		return fmt.Errorf("failed to get gov params: %w", err)
	}
	govParams.ExpeditedQuorum = govParams.Quorum

	defaultParams := v1.DefaultParams()
	govParams.YesQuorum = defaultParams.YesQuorum
	govParams.OptimisticAuthorizedAddresses = defaultParams.OptimisticAuthorizedAddresses
	govParams.OptimisticRejectedThreshold = defaultParams.OptimisticRejectedThreshold
	govParams.ProposalCancelMaxPeriod = defaultParams.ProposalCancelMaxPeriod
	govParams.ProposalExecutionGas = defaultParams.ProposalExecutionGas

	return paramsCollection.Set(ctx, govParams)
}
