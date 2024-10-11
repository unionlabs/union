package keeper

import (
	"context"
	"errors"
	"fmt"
	"time"

	"google.golang.org/protobuf/runtime/protoiface"

	"cosmossdk.io/collections"
	"cosmossdk.io/core/event"
	"cosmossdk.io/core/router"
	"cosmossdk.io/x/gov/types"
	v1 "cosmossdk.io/x/gov/types/v1"

	"github.com/cosmos/cosmos-sdk/telemetry"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// EndBlocker is called every block.
func (k Keeper) EndBlocker(ctx context.Context) error {
	start := telemetry.Now()
	defer telemetry.ModuleMeasureSince(types.ModuleName, start, telemetry.MetricKeyEndBlocker)

	// delete dead proposals from store and returns theirs deposits.
	// A proposal is dead when it's inactive and didn't get enough deposit on time to get into voting phase.
	rng := collections.NewPrefixUntilPairRange[time.Time, uint64](k.HeaderService.HeaderInfo(ctx).Time)
	iter, err := k.InactiveProposalsQueue.Iterate(ctx, rng)
	if err != nil {
		return err
	}

	inactiveProps, err := iter.KeyValues()
	if err != nil {
		return err
	}

	params, err := k.Params.Get(ctx)
	if err != nil {
		return err
	}

	for _, prop := range inactiveProps {
		proposal, err := k.Proposals.Get(ctx, prop.Key.K2())
		if err != nil {
			// if the proposal has an encoding error, this means it cannot be processed by x/gov
			// this could be due to some types missing their registration
			// instead of returning an error (i.e, halting the chain), we fail the proposal
			if errors.Is(err, collections.ErrEncoding) {
				proposal.Id = prop.Key.K2()
				if err := failUnsupportedProposal(ctx, k, proposal, err.Error(), false); err != nil {
					return err
				}

				if err = k.DeleteProposal(ctx, proposal.Id); err != nil {
					return err
				}

				continue
			}

			return err
		}

		if err = k.DeleteProposal(ctx, proposal.Id); err != nil {
			return err
		}

		if !params.BurnProposalDepositPrevote {
			err = k.RefundAndDeleteDeposits(ctx, proposal.Id) // refund deposit if proposal got removed without getting 100% of the proposal
		} else {
			err = k.DeleteAndBurnDeposits(ctx, proposal.Id) // burn the deposit if proposal got removed without getting 100% of the proposal
		}

		if err != nil {
			return err
		}

		// called when proposal become inactive
		// call hook when proposal become inactive
		if err = k.BranchService.Execute(ctx, func(ctx context.Context) error {
			return k.Hooks().AfterProposalFailedMinDeposit(ctx, proposal.Id)
		}); err != nil {
			// purposely ignoring the error here not to halt the chain if the hook fails
			k.Logger.Error("failed to execute AfterProposalFailedMinDeposit hook", "error", err)
		}

		if err := k.EventService.EventManager(ctx).EmitKV(types.EventTypeInactiveProposal,
			event.NewAttribute(types.AttributeKeyProposalID, fmt.Sprintf("%d", proposal.Id)),
			event.NewAttribute(types.AttributeKeyProposalResult, types.AttributeValueProposalDropped),
		); err != nil {
			k.Logger.Error("failed to emit event", "error", err)
		}

		k.Logger.Info(
			"proposal did not meet minimum deposit; deleted",
			"proposal", proposal.Id,
			"proposal_type", proposal.ProposalType,
			"title", proposal.Title,
			"min_deposit", sdk.NewCoins(proposal.GetMinDepositFromParams(params)...).String(),
			"total_deposit", sdk.NewCoins(proposal.TotalDeposit...).String(),
		)
	}

	// fetch active proposals whose voting periods have ended (are passed the block time)
	rng = collections.NewPrefixUntilPairRange[time.Time, uint64](k.HeaderService.HeaderInfo(ctx).Time)

	iter, err = k.ActiveProposalsQueue.Iterate(ctx, rng)
	if err != nil {
		return err
	}

	activeProps, err := iter.KeyValues()
	if err != nil {
		return err
	}

	// err = k.ActiveProposalsQueue.Walk(ctx, rng, func(key collections.Pair[time.Time, uint64], _ uint64) (bool, error) {
	for _, prop := range activeProps {
		proposal, err := k.Proposals.Get(ctx, prop.Key.K2())
		if err != nil {
			// if the proposal has an encoding error, this means it cannot be processed by x/gov
			// this could be due to some types missing their registration
			// instead of returning an error (i.e, halting the chain), we fail the proposal
			if errors.Is(err, collections.ErrEncoding) {
				proposal.Id = prop.Key.K2()
				if err := failUnsupportedProposal(ctx, k, proposal, err.Error(), true); err != nil {
					return err
				}

				if err = k.ActiveProposalsQueue.Remove(ctx, collections.Join(*proposal.VotingEndTime, proposal.Id)); err != nil {
					return err
				}

				continue
			}

			return err
		}

		var tagValue, logMsg string

		passes, burnDeposits, tallyResults, err := k.Tally(ctx, proposal)
		if err != nil {
			return err
		}

		// Deposits are always burned if tally said so, regardless of the proposal type.
		// If a proposal passes, deposits are always refunded, regardless of the proposal type.
		// If a proposal fails, and isn't spammy, deposits are refunded, unless the proposal is expedited or optimistic.
		// An expedited or optimistic proposal that fails and isn't spammy is converted to a regular proposal.
		if burnDeposits {
			err = k.DeleteAndBurnDeposits(ctx, proposal.Id)
		} else if passes || !(proposal.ProposalType == v1.ProposalType_PROPOSAL_TYPE_EXPEDITED || proposal.ProposalType == v1.ProposalType_PROPOSAL_TYPE_OPTIMISTIC) {
			err = k.RefundAndDeleteDeposits(ctx, proposal.Id)
		}
		if err != nil {
			// in case of an error, log it and emit an event
			// we do not want to halt the chain if the refund/burn fails
			// as it could happen due to a governance mistake (governance has let a proposal pass that sends gov funds that were from proposal deposits)
			k.Logger.Error("failed to refund or burn deposits", "error", err)

			if err := k.EventService.EventManager(ctx).EmitKV(types.EventTypeProposalDeposit,
				event.NewAttribute(types.AttributeKeyProposalID, fmt.Sprintf("%d", proposal.Id)),
				event.NewAttribute(types.AttributeKeyProposalDepositError, "failed to refund or burn deposits"),
				event.NewAttribute("error", err.Error()),
			); err != nil {
				k.Logger.Error("failed to emit event", "error", err)
			}
		}

		if err = k.ActiveProposalsQueue.Remove(ctx, collections.Join(*proposal.VotingEndTime, proposal.Id)); err != nil {
			return err
		}

		switch {
		case passes:
			var (
				idx int
				msg sdk.Msg
			)

			messages, err := proposal.GetMsgs()
			if err != nil {
				proposal.Status = v1.StatusFailed
				proposal.FailedReason = err.Error()
				tagValue = types.AttributeValueProposalFailed
				logMsg = fmt.Sprintf("passed proposal (%v) failed to execute; msgs: %s", proposal, err)

				break
			}

			// attempt to execute all messages within the passed proposal
			// Messages may mutate state thus we use a cached context. If one of
			// the handlers fails, no state mutation is written and the error
			// message is logged.
			_, err = k.BranchService.ExecuteWithGasLimit(ctx, params.ProposalExecutionGas, func(ctx context.Context) error {
				// execute all messages
				for idx, msg = range messages {
					if _, err := safeExecuteHandler(ctx, msg, k.MsgRouterService); err != nil {
						return err
					}
				}

				proposal.Status = v1.StatusPassed
				tagValue = types.AttributeValueProposalPassed
				logMsg = "passed"

				return nil
			})
			if err != nil {
				// `idx` and `err` are populated with the msg index and error.
				proposal.Status = v1.StatusFailed
				proposal.FailedReason = err.Error()
				tagValue = types.AttributeValueProposalFailed
				logMsg = fmt.Sprintf("passed, but msg %d (%s) failed on execution: %s", idx, sdk.MsgTypeURL(msg), err)

				break // We do not anything with the error. Returning an error halts the chain, and proposal struct is already updated.
			}
		case !burnDeposits && (proposal.ProposalType == v1.ProposalType_PROPOSAL_TYPE_EXPEDITED ||
			proposal.ProposalType == v1.ProposalType_PROPOSAL_TYPE_OPTIMISTIC):
			// When a non spammy expedited/optimistic proposal fails, it is converted
			// to a regular proposal. As a result, the voting period is extended, and,
			// once the regular voting period expires again, the tally is repeated
			// according to the regular proposal rules.
			proposal.ProposalType = v1.ProposalType_PROPOSAL_TYPE_STANDARD
			endTime := proposal.VotingStartTime.Add(*params.VotingPeriod)
			proposal.VotingEndTime = &endTime

			err = k.ActiveProposalsQueue.Set(ctx, collections.Join(*proposal.VotingEndTime, proposal.Id), proposal.Id)
			if err != nil {
				return err
			}

			if proposal.ProposalType == v1.ProposalType_PROPOSAL_TYPE_EXPEDITED {
				tagValue = types.AttributeValueExpeditedProposalRejected
				logMsg = "expedited proposal converted to regular"
			} else {
				tagValue = types.AttributeValueOptimisticProposalRejected
				logMsg = "optimistic proposal converted to regular"
			}
		default:
			proposal.Status = v1.StatusRejected
			proposal.FailedReason = "proposal did not get enough votes to pass"
			tagValue = types.AttributeValueProposalRejected
			logMsg = "rejected"
		}

		proposal.FinalTallyResult = &tallyResults

		if err = k.Proposals.Set(ctx, proposal.Id, proposal); err != nil {
			return err
		}

		// call hook when proposal become active
		if err := k.BranchService.Execute(ctx, func(ctx context.Context) error {
			return k.Hooks().AfterProposalVotingPeriodEnded(ctx, proposal.Id)
		}); err != nil {
			// purposely ignoring the error here not to halt the chain if the hook fails
			k.Logger.Error("failed to execute AfterProposalVotingPeriodEnded hook", "error", err)
		}

		k.Logger.Info(
			"proposal tallied",
			"proposal", proposal.Id,
			"proposal_type", proposal.ProposalType,
			"status", proposal.Status.String(),
			"title", proposal.Title,
			"results", logMsg,
		)

		if err := k.EventService.EventManager(ctx).EmitKV(types.EventTypeActiveProposal,
			event.NewAttribute(types.AttributeKeyProposalID, fmt.Sprintf("%d", proposal.Id)),
			event.NewAttribute(types.AttributeKeyProposalResult, tagValue),
			event.NewAttribute(types.AttributeKeyProposalLog, logMsg),
		); err != nil {
			k.Logger.Error("failed to emit event", "error", err)
		}
	}
	return nil
}

// executes route(msg) and recovers from panic.
func safeExecuteHandler(ctx context.Context, msg sdk.Msg, router router.Service) (res protoiface.MessageV1, err error) {
	defer func() {
		if r := recover(); r != nil {
			err = fmt.Errorf("handling x/gov proposal msg [%s] PANICKED: %v", msg, r)
		}
	}()

	res, err = router.Invoke(ctx, msg)
	return
}

// failUnsupportedProposal fails a proposal that cannot be processed by gov
func failUnsupportedProposal(
	ctx context.Context,
	k Keeper,
	proposal v1.Proposal,
	errMsg string,
	active bool,
) error {
	proposal.Status = v1.StatusFailed
	proposal.FailedReason = fmt.Sprintf("proposal failed because it cannot be processed by gov: %s", errMsg)
	proposal.Messages = nil // clear out the messages

	if err := k.Proposals.Set(ctx, proposal.Id, proposal); err != nil {
		return err
	}

	if err := k.RefundAndDeleteDeposits(ctx, proposal.Id); err != nil {
		return err
	}

	eventType := types.EventTypeInactiveProposal
	if active {
		eventType = types.EventTypeActiveProposal
	}

	if err := k.EventService.EventManager(ctx).EmitKV(eventType,
		event.NewAttribute(types.AttributeKeyProposalID, fmt.Sprintf("%d", proposal.Id)),
		event.NewAttribute(types.AttributeKeyProposalResult, types.AttributeValueProposalFailed),
	); err != nil {
		k.Logger.Error("failed to emit event", "error", err)
	}

	k.Logger.Info(
		"proposal failed to decode; deleted",
		"proposal", proposal.Id,
		"proposal_type", proposal.ProposalType,
		"title", proposal.Title,
		"results", errMsg,
	)

	return nil
}
