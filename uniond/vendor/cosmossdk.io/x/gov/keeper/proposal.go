package keeper

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"slices"
	"strings"
	"time"

	"cosmossdk.io/collections"
	corecontext "cosmossdk.io/core/context"
	"cosmossdk.io/core/event"
	errorsmod "cosmossdk.io/errors"
	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/gov/types"
	v1 "cosmossdk.io/x/gov/types/v1"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// SubmitProposal creates a new proposal given an array of messages
func (k Keeper) SubmitProposal(ctx context.Context, messages []sdk.Msg, metadata, title, summary string, proposer sdk.AccAddress, proposalType v1.ProposalType) (v1.Proposal, error) {
	params, err := k.Params.Get(ctx)
	if err != nil {
		return v1.Proposal{}, err
	}

	proposerAddr, err := k.authKeeper.AddressCodec().BytesToString(proposer)
	if err != nil {
		return v1.Proposal{}, err
	}

	// additional checks per proposal types
	switch proposalType {
	case v1.ProposalType_PROPOSAL_TYPE_OPTIMISTIC:
		if len(params.OptimisticAuthorizedAddresses) > 0 && !slices.Contains(params.OptimisticAuthorizedAddresses, proposerAddr) {
			return v1.Proposal{}, errorsmod.Wrap(types.ErrInvalidProposer, "proposer is not authorized to submit optimistic proposal")
		}
	case v1.ProposalType_PROPOSAL_TYPE_MULTIPLE_CHOICE:
		if len(messages) > 0 { // cannot happen, except when the proposal is created via keeper call instead of message server.
			return v1.Proposal{}, errorsmod.Wrap(types.ErrInvalidProposalMsg, "multiple choice proposal should not contain any messages")
		}
	}

	msgs := make([]string, 0, len(messages)) // will hold a string slice of all Msg type URLs.

	// Loop through all messages and confirm that each has a handler and the gov module account as the only signer
	var currentMessagedBasedParams *v1.MessageBasedParams
	for _, msg := range messages {
		msgs = append(msgs, sdk.MsgTypeURL(msg))

		// check if any of the message has message based params
		hasMessagedBasedParams := true
		messagedBasedParams, err := k.MessageBasedParams.Get(ctx, sdk.MsgTypeURL(msg))
		if err != nil {
			if !errorsmod.IsOf(err, collections.ErrNotFound) {
				return v1.Proposal{}, err
			}

			hasMessagedBasedParams = false
		}

		if hasMessagedBasedParams {
			// set initial value for currentMessagedBasedParams
			if currentMessagedBasedParams == nil {
				currentMessagedBasedParams = &messagedBasedParams
			}

			// check if newly fetched messagedBasedParams is different from the previous fetched params
			isEqual, err := currentMessagedBasedParams.Equal(&messagedBasedParams)
			if err != nil {
				return v1.Proposal{}, err
			}

			if len(messages) > 1 && !isEqual {
				return v1.Proposal{}, errorsmod.Wrap(types.ErrInvalidProposalMsg, "cannot submit multiple messages proposal with different message based params")
			}

			if proposalType != v1.ProposalType_PROPOSAL_TYPE_STANDARD {
				return v1.Proposal{}, errorsmod.Wrap(types.ErrInvalidProposalType, "cannot submit non standard proposal with message based params")
			}
		}

		// perform a basic validation of the message
		if m, ok := msg.(sdk.HasValidateBasic); ok {
			if err := m.ValidateBasic(); err != nil {
				return v1.Proposal{}, errorsmod.Wrap(types.ErrInvalidProposalMsg, err.Error())
			}
		}

		signers, _, err := k.cdc.GetMsgSigners(msg)
		if err != nil {
			return v1.Proposal{}, err
		}
		if len(signers) != 1 {
			return v1.Proposal{}, types.ErrInvalidSigner
		}

		// assert that the governance module account is the only signer of the messages
		if !bytes.Equal(signers[0], k.GetGovernanceAccount(ctx).GetAddress()) {
			addr, err := k.authKeeper.AddressCodec().BytesToString(signers[0])
			if err != nil {
				return v1.Proposal{}, errorsmod.Wrap(types.ErrInvalidSigner, err.Error())
			}
			return v1.Proposal{}, errorsmod.Wrap(types.ErrInvalidSigner, addr)
		}

		if err := k.MsgRouterService.CanInvoke(ctx, sdk.MsgTypeURL(msg)); err != nil {
			return v1.Proposal{}, errorsmod.Wrap(types.ErrUnroutableProposalMsg, err.Error())
		}

		// Only if it's a MsgExecLegacyContent we try to execute the
		// proposal in a cached context.
		// For other Msgs, we do not verify the proposal messages any further.
		// They may fail upon execution.
		// ref: https://github.com/cosmos/cosmos-sdk/pull/10868#discussion_r784872842
		msg, ok := msg.(*v1.MsgExecLegacyContent)
		if !ok {
			continue
		}

		content, err := v1.LegacyContentFromMessage(msg)
		if err != nil {
			return v1.Proposal{}, errorsmod.Wrapf(types.ErrInvalidProposalContent, "%+v", err)
		}

		// Ensure that the content has a respective handler
		if !k.legacyRouter.HasRoute(content.ProposalRoute()) {
			return v1.Proposal{}, errorsmod.Wrap(types.ErrNoProposalHandlerExists, content.ProposalRoute())
		}

		if err = k.BranchService.Execute(ctx, func(ctx context.Context) error {
			handler := k.legacyRouter.GetRoute(content.ProposalRoute())
			// NOTE: the support of legacy gov proposal in server/v2 is different than for baseapp.
			// Legacy proposal in server/v2 can only access services provided by the gov module environment.
			if err := handler(context.WithValue(ctx, corecontext.EnvironmentContextKey, k.Environment), content); err != nil {
				return types.ErrInvalidProposalContent.Wrapf("failed to run legacy handler %s, %+v", content.ProposalRoute(), err)
			}

			return errors.New("we don't want to execute the proposal, we just want to check if it can be executed")
		}); errors.Is(err, types.ErrInvalidProposalContent) {
			return v1.Proposal{}, err
		}
	}

	proposalID, err := k.ProposalID.Next(ctx)
	if err != nil {
		return v1.Proposal{}, err
	}

	submitTime := k.HeaderService.HeaderInfo(ctx).Time
	proposal, err := v1.NewProposal(messages, proposalID, submitTime, submitTime.Add(*params.MaxDepositPeriod), metadata, title, summary, proposerAddr, proposalType)
	if err != nil {
		return v1.Proposal{}, err
	}

	if err = k.Proposals.Set(ctx, proposal.Id, proposal); err != nil {
		return v1.Proposal{}, err
	}
	err = k.InactiveProposalsQueue.Set(ctx, collections.Join(*proposal.DepositEndTime, proposalID), proposalID)
	if err != nil {
		return v1.Proposal{}, err
	}

	// called right after a proposal is submitted
	err = k.Hooks().AfterProposalSubmission(ctx, proposalID)
	if err != nil {
		return v1.Proposal{}, err
	}

	if err := k.EventService.EventManager(ctx).EmitKV(
		types.EventTypeSubmitProposal,
		event.NewAttribute(types.AttributeKeyProposalType, proposalType.String()),
		event.NewAttribute(types.AttributeKeyProposalID, fmt.Sprintf("%d", proposalID)),
		event.NewAttribute(types.AttributeKeyProposalProposer, proposerAddr),
		event.NewAttribute(types.AttributeKeyProposalMessages, strings.Join(msgs, ",")),
	); err != nil {
		return v1.Proposal{}, fmt.Errorf("failed to emit event: %w", err)
	}

	return proposal, nil
}

// CancelProposal will cancel proposal before the voting period ends
func (k Keeper) CancelProposal(ctx context.Context, proposalID uint64, proposer string) error {
	proposal, err := k.Proposals.Get(ctx, proposalID)
	if err != nil {
		if errors.Is(err, collections.ErrNotFound) {
			return types.ErrInvalidProposal.Wrapf("proposal %d doesn't exist", proposalID)
		}
		return err
	}

	params, err := k.Params.Get(ctx)
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

	// Check proposal is not too far in voting period to be canceled
	if proposal.VotingEndTime != nil {
		currentTime := k.HeaderService.HeaderInfo(ctx).Time

		maxCancelPeriodRate := sdkmath.LegacyMustNewDecFromStr(params.ProposalCancelMaxPeriod)
		maxCancelPeriod := time.Duration(float64(proposal.VotingEndTime.Sub(*proposal.VotingStartTime)) * maxCancelPeriodRate.MustFloat64()).Round(time.Second)

		if proposal.VotingEndTime.Before(currentTime) {
			return types.ErrVotingPeriodEnded.Wrapf("voting period is already ended for this proposal %d", proposalID)
		} else if proposal.VotingEndTime.Add(-maxCancelPeriod).Before(currentTime) {
			return types.ErrTooLateToCancel.Wrapf("proposal %d is too late to cancel", proposalID)
		}
	}

	// burn the (deposits * proposal_cancel_rate) amount or sent to cancellation destination address.
	// and deposits * (1 - proposal_cancel_rate) will be sent to depositors.
	err = k.ChargeDeposit(ctx, proposal.Id, params.ProposalCancelDest, params.ProposalCancelRatio)
	if err != nil {
		return err
	}

	if proposal.VotingStartTime != nil {
		err = k.deleteVotes(ctx, proposal.Id)
		if err != nil {
			return err
		}
	}

	err = k.DeleteProposal(ctx, proposal.Id)
	if err != nil {
		return err
	}

	k.Logger.Info(
		"proposal is canceled by proposer",
		"proposal", proposal.Id,
		"proposer", proposal.Proposer,
	)

	return nil
}

// DeleteProposal deletes a proposal from store.
func (k Keeper) DeleteProposal(ctx context.Context, proposalID uint64) error {
	proposal, err := k.Proposals.Get(ctx, proposalID)
	if err != nil {
		return err
	}

	if proposal.DepositEndTime != nil {
		err := k.InactiveProposalsQueue.Remove(ctx, collections.Join(*proposal.DepositEndTime, proposalID))
		if err != nil {
			return err
		}
	}
	if proposal.VotingEndTime != nil {
		err := k.ActiveProposalsQueue.Remove(ctx, collections.Join(*proposal.VotingEndTime, proposalID))
		if err != nil {
			return err
		}
	}

	return k.Proposals.Remove(ctx, proposalID)
}

// ActivateVotingPeriod activates the voting period of a proposal
func (k Keeper) ActivateVotingPeriod(ctx context.Context, proposal v1.Proposal) error {
	startTime := k.HeaderService.HeaderInfo(ctx).Time
	proposal.VotingStartTime = &startTime

	params, err := k.Params.Get(ctx)
	if err != nil {
		return err
	}

	var votingPeriod *time.Duration
	switch proposal.ProposalType {
	case v1.ProposalType_PROPOSAL_TYPE_EXPEDITED:
		votingPeriod = params.ExpeditedVotingPeriod
	default:
		votingPeriod = params.VotingPeriod

		if len(proposal.Messages) > 0 {
			// check if any of the message has message based params
			customMessageParams, err := k.MessageBasedParams.Get(ctx, sdk.MsgTypeURL(proposal.Messages[0]))
			if err == nil {
				votingPeriod = customMessageParams.VotingPeriod
			} else if !errors.Is(err, collections.ErrNotFound) {
				return err
			}
		}
	}

	endTime := proposal.VotingStartTime.Add(*votingPeriod)
	proposal.VotingEndTime = &endTime
	proposal.Status = v1.StatusVotingPeriod
	if err = k.Proposals.Set(ctx, proposal.Id, proposal); err != nil {
		return err
	}

	if err = k.InactiveProposalsQueue.Remove(ctx, collections.Join(*proposal.DepositEndTime, proposal.Id)); err != nil {
		return err
	}

	return k.ActiveProposalsQueue.Set(ctx, collections.Join(*proposal.VotingEndTime, proposal.Id), proposal.Id)
}
