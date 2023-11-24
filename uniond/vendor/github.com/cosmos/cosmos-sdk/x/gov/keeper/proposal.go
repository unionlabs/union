package keeper

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"time"

	"cosmossdk.io/collections"
	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/gov/types"
	v1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"
)

// SubmitProposal creates a new proposal given an array of messages
func (keeper Keeper) SubmitProposal(ctx context.Context, messages []sdk.Msg, metadata, title, summary string, proposer sdk.AccAddress, expedited bool) (v1.Proposal, error) {
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	err := keeper.assertMetadataLength(metadata)
	if err != nil {
		return v1.Proposal{}, err
	}

	// assert summary is no longer than predefined max length of metadata
	err = keeper.assertSummaryLength(summary)
	if err != nil {
		return v1.Proposal{}, err
	}

	// assert title is no longer than predefined max length of metadata
	err = keeper.assertMetadataLength(title)
	if err != nil {
		return v1.Proposal{}, err
	}

	// Will hold a comma-separated string of all Msg type URLs.
	msgsStr := ""

	// Loop through all messages and confirm that each has a handler and the gov module account
	// as the only signer
	for _, msg := range messages {
		msgsStr += fmt.Sprintf(",%s", sdk.MsgTypeURL(msg))

		// perform a basic validation of the message
		if m, ok := msg.(sdk.HasValidateBasic); ok {
			if err := m.ValidateBasic(); err != nil {
				return v1.Proposal{}, errorsmod.Wrap(types.ErrInvalidProposalMsg, err.Error())
			}
		}

		signers, _, err := keeper.cdc.GetMsgV1Signers(msg)
		if err != nil {
			return v1.Proposal{}, err
		}
		if len(signers) != 1 {
			return v1.Proposal{}, types.ErrInvalidSigner
		}

		// assert that the governance module account is the only signer of the messages
		if !bytes.Equal(signers[0], keeper.GetGovernanceAccount(ctx).GetAddress()) {
			return v1.Proposal{}, errorsmod.Wrapf(types.ErrInvalidSigner, sdk.AccAddress(signers[0]).String())
		}

		// use the msg service router to see that there is a valid route for that message.
		handler := keeper.router.Handler(msg)
		if handler == nil {
			return v1.Proposal{}, errorsmod.Wrap(types.ErrUnroutableProposalMsg, sdk.MsgTypeURL(msg))
		}

		// Only if it's a MsgExecLegacyContent do we try to execute the
		// proposal in a cached context.
		// For other Msgs, we do not verify the proposal messages any further.
		// They may fail upon execution.
		// ref: https://github.com/cosmos/cosmos-sdk/pull/10868#discussion_r784872842
		if msg, ok := msg.(*v1.MsgExecLegacyContent); ok {
			cacheCtx, _ := sdkCtx.CacheContext()
			if _, err := handler(cacheCtx, msg); err != nil {
				if errors.Is(types.ErrNoProposalHandlerExists, err) {
					return v1.Proposal{}, err
				}
				return v1.Proposal{}, errorsmod.Wrap(types.ErrInvalidProposalContent, err.Error())
			}
		}

	}

	proposalID, err := keeper.ProposalID.Next(ctx)
	if err != nil {
		return v1.Proposal{}, err
	}

	params, err := keeper.Params.Get(ctx)
	if err != nil {
		return v1.Proposal{}, err
	}

	submitTime := sdkCtx.BlockHeader().Time
	depositPeriod := params.MaxDepositPeriod

	proposal, err := v1.NewProposal(messages, proposalID, submitTime, submitTime.Add(*depositPeriod), metadata, title, summary, proposer, expedited)
	if err != nil {
		return v1.Proposal{}, err
	}

	err = keeper.SetProposal(ctx, proposal)
	if err != nil {
		return v1.Proposal{}, err
	}
	err = keeper.InactiveProposalsQueue.Set(ctx, collections.Join(*proposal.DepositEndTime, proposalID), proposalID)
	if err != nil {
		return v1.Proposal{}, err
	}

	// called right after a proposal is submitted
	err = keeper.Hooks().AfterProposalSubmission(ctx, proposalID)
	if err != nil {
		return v1.Proposal{}, err
	}

	sdkCtx.EventManager().EmitEvent(
		sdk.NewEvent(
			types.EventTypeSubmitProposal,
			sdk.NewAttribute(types.AttributeKeyProposalID, fmt.Sprintf("%d", proposalID)),
			sdk.NewAttribute(types.AttributeKeyProposalMessages, msgsStr),
		),
	)

	return proposal, nil
}

// CancelProposal will cancel proposal before the voting period ends
func (keeper Keeper) CancelProposal(ctx context.Context, proposalID uint64, proposer string) error {
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	proposal, err := keeper.Proposals.Get(ctx, proposalID)
	if err != nil {
		return err
	}

	// Checking proposal have proposer or not because old proposal doesn't have proposer field,
	// https://github.com/cosmos/cosmos-sdk/blob/v0.46.2/proto/cosmos/gov/v1/gov.proto#L43
	if proposal.Proposer == "" {
		return types.ErrInvalidProposal.Wrapf("proposal %d doesn't have proposer %s, so cannot be canceled", proposalID, proposer)
	}

	// Check creator of the proposal
	if proposal.Proposer != proposer {
		return types.ErrInvalidProposer.Wrapf("invalid proposer %s", proposer)
	}

	// Check if proposal is active or not
	if (proposal.Status != v1.StatusDepositPeriod) && (proposal.Status != v1.StatusVotingPeriod) {
		return types.ErrInvalidProposal.Wrap("proposal should be in the deposit or voting period")
	}

	// Check proposal voting period is ended.
	if proposal.VotingEndTime != nil && proposal.VotingEndTime.Before(sdkCtx.BlockTime()) {
		return types.ErrVotingPeriodEnded.Wrapf("voting period is already ended for this proposal %d", proposalID)
	}

	// burn the (deposits * proposal_cancel_rate) amount or sent to cancellation destination address.
	// and deposits * (1 - proposal_cancel_rate) will be sent to depositors.
	params, err := keeper.Params.Get(ctx)
	if err != nil {
		return err
	}

	err = keeper.ChargeDeposit(ctx, proposal.Id, params.ProposalCancelDest, params.ProposalCancelRatio)
	if err != nil {
		return err
	}

	if proposal.VotingStartTime != nil {
		err = keeper.deleteVotes(ctx, proposal.Id)
		if err != nil {
			return err
		}
	}

	err = keeper.DeleteProposal(ctx, proposal.Id)
	if err != nil {
		return err
	}

	keeper.Logger(ctx).Info(
		"proposal is canceled by proposer",
		"proposal", proposal.Id,
		"proposer", proposal.Proposer,
	)

	return nil
}

// SetProposal sets a proposal to store.
func (keeper Keeper) SetProposal(ctx context.Context, proposal v1.Proposal) error {
	if proposal.Status == v1.StatusVotingPeriod {
		err := keeper.VotingPeriodProposals.Set(ctx, proposal.Id, []byte{1})
		if err != nil {
			return err
		}
	} else {
		err := keeper.VotingPeriodProposals.Remove(ctx, proposal.Id)
		if err != nil {
			return err
		}
	}

	return keeper.Proposals.Set(ctx, proposal.Id, proposal)
}

// DeleteProposal deletes a proposal from store.
func (keeper Keeper) DeleteProposal(ctx context.Context, proposalID uint64) error {
	proposal, err := keeper.Proposals.Get(ctx, proposalID)
	if err != nil {
		return err
	}

	if proposal.DepositEndTime != nil {
		err := keeper.InactiveProposalsQueue.Remove(ctx, collections.Join(*proposal.DepositEndTime, proposalID))
		if err != nil {
			return err
		}
	}
	if proposal.VotingEndTime != nil {
		err := keeper.ActiveProposalsQueue.Remove(ctx, collections.Join(*proposal.VotingEndTime, proposalID))
		if err != nil {
			return err
		}

		err = keeper.VotingPeriodProposals.Remove(ctx, proposalID)
		if err != nil {
			return err
		}
	}

	return keeper.Proposals.Remove(ctx, proposalID)
}

// ActivateVotingPeriod activates the voting period of a proposal
func (keeper Keeper) ActivateVotingPeriod(ctx context.Context, proposal v1.Proposal) error {
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	startTime := sdkCtx.BlockHeader().Time
	proposal.VotingStartTime = &startTime
	var votingPeriod *time.Duration
	params, err := keeper.Params.Get(ctx)
	if err != nil {
		return err
	}

	if proposal.Expedited {
		votingPeriod = params.ExpeditedVotingPeriod
	} else {
		votingPeriod = params.VotingPeriod
	}
	endTime := proposal.VotingStartTime.Add(*votingPeriod)
	proposal.VotingEndTime = &endTime
	proposal.Status = v1.StatusVotingPeriod
	err = keeper.SetProposal(ctx, proposal)
	if err != nil {
		return err
	}

	err = keeper.InactiveProposalsQueue.Remove(ctx, collections.Join(*proposal.DepositEndTime, proposal.Id))
	if err != nil {
		return err
	}

	return keeper.ActiveProposalsQueue.Set(ctx, collections.Join(*proposal.VotingEndTime, proposal.Id), proposal.Id)
}
