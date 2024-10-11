package slasher

import (
	"context"

	"github.com/pkg/errors"
	slashertypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/slasher/types"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"go.opencensus.io/trace"
)

// detectProposerSlashings takes in signed block header wrappers and returns a list of proposer slashings detected.
func (s *Service) detectProposerSlashings(
	ctx context.Context,
	proposedBlocks []*slashertypes.SignedBlockHeaderWrapper,
) ([]*ethpb.ProposerSlashing, error) {
	ctx, span := trace.StartSpan(ctx, "slasher.detectProposerSlashings")
	defer span.End()
	// We check if there are any slashable double proposals in the input list
	// of proposals with respect to each other.
	slashings := make([]*ethpb.ProposerSlashing, 0)
	existingProposals := make(map[string]*slashertypes.SignedBlockHeaderWrapper)
	for i, proposal := range proposedBlocks {
		key := proposalKey(proposal)
		existingProposal, ok := existingProposals[key]
		if !ok {
			existingProposals[key] = proposal
			continue
		}
		if isDoubleProposal(proposedBlocks[i].SigningRoot, existingProposal.SigningRoot) {
			doubleProposalsTotal.Inc()
			slashing := &ethpb.ProposerSlashing{
				Header_1: existingProposal.SignedBeaconBlockHeader,
				Header_2: proposedBlocks[i].SignedBeaconBlockHeader,
			}
			slashings = append(slashings, slashing)
		}
	}

	proposerSlashings, err := s.serviceCfg.Database.CheckDoubleBlockProposals(ctx, proposedBlocks)
	if err != nil {
		return nil, errors.Wrap(err, "could not check for double proposals on disk")
	}
	if err := s.saveSafeProposals(ctx, proposedBlocks, proposerSlashings); err != nil {
		return nil, errors.Wrap(err, "could not save safe proposals")
	}
	slashings = append(slashings, proposerSlashings...)
	return slashings, nil
}

// Check for double proposals in our database given a list of incoming block proposals.
// For the proposals that were not slashable, we save them to the database.
func (s *Service) saveSafeProposals(
	ctx context.Context,
	proposedBlocks []*slashertypes.SignedBlockHeaderWrapper,
	proposerSlashings []*ethpb.ProposerSlashing,
) error {
	ctx, span := trace.StartSpan(ctx, "slasher.saveSafeProposals")
	defer span.End()
	return s.serviceCfg.Database.SaveBlockProposals(
		ctx,
		filterSafeProposals(proposedBlocks, proposerSlashings),
	)
}

func filterSafeProposals(
	proposedBlocks []*slashertypes.SignedBlockHeaderWrapper,
	proposerSlashings []*ethpb.ProposerSlashing,
) []*slashertypes.SignedBlockHeaderWrapper {
	// We initialize a map of proposers that are safe from slashing.
	safeProposers := make(map[primitives.ValidatorIndex]*slashertypes.SignedBlockHeaderWrapper, len(proposedBlocks))
	for _, proposal := range proposedBlocks {
		safeProposers[proposal.SignedBeaconBlockHeader.Header.ProposerIndex] = proposal
	}
	for _, doubleProposal := range proposerSlashings {
		// If a proposer is found to have committed a slashable offense, we delete
		// them from the safe proposers map.
		delete(safeProposers, doubleProposal.Header_1.Header.ProposerIndex)
	}
	// We save all the proposals that are determined "safe" and not-slashable to our database.
	safeProposals := make([]*slashertypes.SignedBlockHeaderWrapper, 0, len(safeProposers))
	for _, proposal := range safeProposers {
		safeProposals = append(safeProposals, proposal)
	}
	return safeProposals
}

func proposalKey(proposal *slashertypes.SignedBlockHeaderWrapper) string {
	header := proposal.SignedBeaconBlockHeader.Header
	return uintToString(uint64(header.Slot)) + ":" + uintToString(uint64(header.ProposerIndex))
}
