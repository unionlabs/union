package keeper

import (
	"context"
	stderrors "errors"
	"fmt"

	"cosmossdk.io/collections"
	"cosmossdk.io/core/event"
	"cosmossdk.io/errors"
	"cosmossdk.io/x/gov/types"
	v1 "cosmossdk.io/x/gov/types/v1"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// AddVote adds a vote on a specific proposal
func (k Keeper) AddVote(ctx context.Context, proposalID uint64, voterAddr sdk.AccAddress, options v1.WeightedVoteOptions, metadata string) error {
	// get proposal
	proposal, err := k.Proposals.Get(ctx, proposalID)
	if err != nil {
		if stderrors.Is(err, collections.ErrNotFound) {
			return errors.Wrapf(types.ErrInactiveProposal, "%d", proposalID)
		}

		return err
	}

	// check if proposal is in voting period.
	if proposal.Status != v1.StatusVotingPeriod {
		return errors.Wrapf(types.ErrInactiveProposal, "%d", proposalID)
	}

	if err := k.assertMetadataLength(metadata); err != nil {
		return err
	}

	err = k.assertVoteOptionsLen(options)
	if err != nil {
		return err
	}

	for _, option := range options {
		switch proposal.ProposalType {
		case v1.ProposalType_PROPOSAL_TYPE_OPTIMISTIC:
			if option.Option != v1.OptionNo {
				return errors.Wrap(types.ErrInvalidVote, "optimistic proposals can only be rejected")
			}
		case v1.ProposalType_PROPOSAL_TYPE_MULTIPLE_CHOICE:
			proposalOptionsStr, err := k.ProposalVoteOptions.Get(ctx, proposalID)
			if err != nil {
				if stderrors.Is(err, collections.ErrNotFound) {
					return errors.Wrap(types.ErrInvalidProposal, "invalid multiple choice proposal, no options set")
				}

				return err
			}

			// verify votes only on existing votes
			if proposalOptionsStr.OptionOne == "" && option.Option == v1.OptionOne {
				return errors.Wrap(types.ErrInvalidVote, "invalid vote option")
			} else if proposalOptionsStr.OptionTwo == "" && option.Option == v1.OptionTwo {
				return errors.Wrap(types.ErrInvalidVote, "invalid vote option")
			} else if proposalOptionsStr.OptionThree == "" && option.Option == v1.OptionThree {
				return errors.Wrap(types.ErrInvalidVote, "invalid vote option")
			} else if proposalOptionsStr.OptionFour == "" && option.Option == v1.OptionFour {
				return errors.Wrap(types.ErrInvalidVote, "invalid vote option")
			}
		}

		if !v1.ValidWeightedVoteOption(*option) {
			return errors.Wrap(types.ErrInvalidVote, option.String())
		}
	}

	voterStrAddr, err := k.authKeeper.AddressCodec().BytesToString(voterAddr)
	if err != nil {
		return err
	}
	vote := v1.NewVote(proposalID, voterStrAddr, options, metadata)
	err = k.Votes.Set(ctx, collections.Join(proposalID, voterAddr), vote)
	if err != nil {
		return err
	}

	// called after a vote on a proposal is cast
	if err = k.Hooks().AfterProposalVote(ctx, proposalID, voterAddr); err != nil {
		return err
	}

	return k.EventService.EventManager(ctx).EmitKV(types.EventTypeProposalVote,
		event.NewAttribute(types.AttributeKeyVoter, voterStrAddr),
		event.NewAttribute(types.AttributeKeyOption, options.String()),
		event.NewAttribute(types.AttributeKeyProposalID, fmt.Sprintf("%d", proposalID)),
	)
}

// deleteVotes deletes all the votes from a given proposalID.
func (k Keeper) deleteVotes(ctx context.Context, proposalID uint64) error {
	rng := collections.NewPrefixedPairRange[uint64, sdk.AccAddress](proposalID)
	err := k.Votes.Clear(ctx, rng)
	if err != nil {
		return err
	}

	return nil
}
