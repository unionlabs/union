package gov

import (
	"errors"
	"fmt"
	"time"

	"cosmossdk.io/collections"
	"cosmossdk.io/log"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/telemetry"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/gov/keeper"
	"github.com/cosmos/cosmos-sdk/x/gov/types"
	v1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"
)

// EndBlocker called every block, process inflation, update validator set.
func EndBlocker(ctx sdk.Context, keeper *keeper.Keeper) error {
	defer telemetry.ModuleMeasureSince(types.ModuleName, time.Now(), telemetry.MetricKeyEndBlocker)

	logger := ctx.Logger().With("module", "x/"+types.ModuleName)
	// delete dead proposals from store and returns theirs deposits.
	// A proposal is dead when it's inactive and didn't get enough deposit on time to get into voting phase.
	rng := collections.NewPrefixUntilPairRange[time.Time, uint64](ctx.BlockTime())
	err := keeper.InactiveProposalsQueue.Walk(ctx, rng, func(key collections.Pair[time.Time, uint64], _ uint64) (bool, error) {
		proposal, err := keeper.Proposals.Get(ctx, key.K2())
		if err != nil {
			// if the proposal has an encoding error, this means it cannot be processed by x/gov
			// this could be due to some types missing their registration
			// instead of returning an error (i.e, halting the chain), we fail the proposal
			if errors.Is(err, collections.ErrEncoding) {
				proposal.Id = key.K2()
				if err := failUnsupportedProposal(logger, ctx, keeper, proposal, err.Error(), false); err != nil {
					return false, err
				}

				if err = keeper.DeleteProposal(ctx, proposal.Id); err != nil {
					return false, err
				}

				return false, nil
			}

			return false, err
		}

		if err = keeper.DeleteProposal(ctx, proposal.Id); err != nil {
			return false, err
		}

		params, err := keeper.Params.Get(ctx)
		if err != nil {
			return false, err
		}
		if !params.BurnProposalDepositPrevote {
			err = keeper.RefundAndDeleteDeposits(ctx, proposal.Id) // refund deposit if proposal got removed without getting 100% of the proposal
		} else {
			err = keeper.DeleteAndBurnDeposits(ctx, proposal.Id) // burn the deposit if proposal got removed without getting 100% of the proposal
		}

		if err != nil {
			return false, err
		}

		// called when proposal become inactive
		cacheCtx, writeCache := ctx.CacheContext()
		err = keeper.Hooks().AfterProposalFailedMinDeposit(cacheCtx, proposal.Id)
		if err == nil { // purposely ignoring the error here not to halt the chain if the hook fails
			writeCache()
		} else {
			keeper.Logger(ctx).Error("failed to execute AfterProposalFailedMinDeposit hook", "error", err)
		}

		ctx.EventManager().EmitEvent(
			sdk.NewEvent(
				types.EventTypeInactiveProposal,
				sdk.NewAttribute(types.AttributeKeyProposalID, fmt.Sprintf("%d", proposal.Id)),
				sdk.NewAttribute(types.AttributeKeyProposalResult, types.AttributeValueProposalDropped),
			),
		)

		logger.Info(
			"proposal did not meet minimum deposit; deleted",
			"proposal", proposal.Id,
			"expedited", proposal.Expedited,
			"title", proposal.Title,
			"min_deposit", sdk.NewCoins(proposal.GetMinDepositFromParams(params)...).String(),
			"total_deposit", sdk.NewCoins(proposal.TotalDeposit...).String(),
		)

		return false, nil
	})
	if err != nil {
		return err
	}

	// fetch active proposals whose voting periods have ended (are passed the block time)
	rng = collections.NewPrefixUntilPairRange[time.Time, uint64](ctx.BlockTime())
	err = keeper.ActiveProposalsQueue.Walk(ctx, rng, func(key collections.Pair[time.Time, uint64], _ uint64) (bool, error) {
		proposal, err := keeper.Proposals.Get(ctx, key.K2())
		if err != nil {
			// if the proposal has an encoding error, this means it cannot be processed by x/gov
			// this could be due to some types missing their registration
			// instead of returning an error (i.e, halting the chain), we fail the proposal
			if errors.Is(err, collections.ErrEncoding) {
				proposal.Id = key.K2()
				if err := failUnsupportedProposal(logger, ctx, keeper, proposal, err.Error(), true); err != nil {
					return false, err
				}

				if err = keeper.ActiveProposalsQueue.Remove(ctx, collections.Join(*proposal.VotingEndTime, proposal.Id)); err != nil {
					return false, err
				}

				return false, nil
			}

			return false, err
		}

		var tagValue, logMsg string

		passes, burnDeposits, tallyResults, err := keeper.Tally(ctx, proposal)
		if err != nil {
			return false, err
		}

		// If an expedited proposal fails, we do not want to update
		// the deposit at this point since the proposal is converted to regular.
		// As a result, the deposits are either deleted or refunded in all cases
		// EXCEPT when an expedited proposal fails.
		if !(proposal.Expedited && !passes) {
			if burnDeposits {
				err = keeper.DeleteAndBurnDeposits(ctx, proposal.Id)
			} else {
				err = keeper.RefundAndDeleteDeposits(ctx, proposal.Id)
			}
			if err != nil {
				return false, err
			}
		}

		if err = keeper.ActiveProposalsQueue.Remove(ctx, collections.Join(*proposal.VotingEndTime, proposal.Id)); err != nil {
			return false, err
		}

		switch {
		case passes:
			var (
				idx    int
				events sdk.Events
				msg    sdk.Msg
			)

			// attempt to execute all messages within the passed proposal
			// Messages may mutate state thus we use a cached context. If one of
			// the handlers fails, no state mutation is written and the error
			// message is logged.
			cacheCtx, writeCache := ctx.CacheContext()
			messages, err := proposal.GetMsgs()
			if err != nil {
				proposal.Status = v1.StatusFailed
				proposal.FailedReason = err.Error()
				tagValue = types.AttributeValueProposalFailed
				logMsg = fmt.Sprintf("passed proposal (%v) failed to execute; msgs: %s", proposal, err)

				break
			}

			// execute all messages
			for idx, msg = range messages {
				handler := keeper.Router().Handler(msg)
				var res *sdk.Result
				res, err = safeExecuteHandler(cacheCtx, msg, handler)
				if err != nil {
					break
				}

				events = append(events, res.GetEvents()...)
			}

			// `err == nil` when all handlers passed.
			// Or else, `idx` and `err` are populated with the msg index and error.
			if err == nil {
				proposal.Status = v1.StatusPassed
				tagValue = types.AttributeValueProposalPassed
				logMsg = "passed"

				// write state to the underlying multi-store
				writeCache()

				// propagate the msg events to the current context
				ctx.EventManager().EmitEvents(events)
			} else {
				proposal.Status = v1.StatusFailed
				proposal.FailedReason = err.Error()
				tagValue = types.AttributeValueProposalFailed
				logMsg = fmt.Sprintf("passed, but msg %d (%s) failed on execution: %s", idx, sdk.MsgTypeURL(msg), err)
			}
		case proposal.Expedited:
			// When expedited proposal fails, it is converted
			// to a regular proposal. As a result, the voting period is extended, and,
			// once the regular voting period expires again, the tally is repeated
			// according to the regular proposal rules.
			proposal.Expedited = false
			params, err := keeper.Params.Get(ctx)
			if err != nil {
				return false, err
			}
			endTime := proposal.VotingStartTime.Add(*params.VotingPeriod)
			proposal.VotingEndTime = &endTime

			err = keeper.ActiveProposalsQueue.Set(ctx, collections.Join(*proposal.VotingEndTime, proposal.Id), proposal.Id)
			if err != nil {
				return false, err
			}

			tagValue = types.AttributeValueExpeditedProposalRejected
			logMsg = "expedited proposal converted to regular"
		default:
			proposal.Status = v1.StatusRejected
			proposal.FailedReason = "proposal did not get enough votes to pass"
			tagValue = types.AttributeValueProposalRejected
			logMsg = "rejected"
		}

		proposal.FinalTallyResult = &tallyResults

		err = keeper.SetProposal(ctx, proposal)
		if err != nil {
			return false, err
		}

		// when proposal become active
		cacheCtx, writeCache := ctx.CacheContext()
		err = keeper.Hooks().AfterProposalVotingPeriodEnded(cacheCtx, proposal.Id)
		if err == nil { // purposely ignoring the error here not to halt the chain if the hook fails
			writeCache()
		} else {
			keeper.Logger(ctx).Error("failed to execute AfterProposalVotingPeriodEnded hook", "error", err)
		}

		logger.Info(
			"proposal tallied",
			"proposal", proposal.Id,
			"status", proposal.Status.String(),
			"expedited", proposal.Expedited,
			"title", proposal.Title,
			"results", logMsg,
		)

		ctx.EventManager().EmitEvent(
			sdk.NewEvent(
				types.EventTypeActiveProposal,
				sdk.NewAttribute(types.AttributeKeyProposalID, fmt.Sprintf("%d", proposal.Id)),
				sdk.NewAttribute(types.AttributeKeyProposalResult, tagValue),
				sdk.NewAttribute(types.AttributeKeyProposalLog, logMsg),
			),
		)

		return false, nil
	})
	if err != nil {
		return err
	}
	return nil
}

// executes handle(msg) and recovers from panic.
func safeExecuteHandler(ctx sdk.Context, msg sdk.Msg, handler baseapp.MsgServiceHandler,
) (res *sdk.Result, err error) {
	defer func() {
		if r := recover(); r != nil {
			err = fmt.Errorf("handling x/gov proposal msg [%s] PANICKED: %v", msg, r)
		}
	}()
	res, err = handler(ctx, msg)
	return
}

// failUnsupportedProposal fails a proposal that cannot be processed by gov
func failUnsupportedProposal(
	logger log.Logger,
	ctx sdk.Context,
	keeper *keeper.Keeper,
	proposal v1.Proposal,
	errMsg string,
	active bool,
) error {
	proposal.Status = v1.StatusFailed
	proposal.FailedReason = fmt.Sprintf("proposal failed because it cannot be processed by gov: %s", errMsg)
	proposal.Messages = nil // clear out the messages

	if err := keeper.SetProposal(ctx, proposal); err != nil {
		return err
	}

	if err := keeper.RefundAndDeleteDeposits(ctx, proposal.Id); err != nil {
		return err
	}

	eventType := types.EventTypeInactiveProposal
	if active {
		eventType = types.EventTypeActiveProposal
	}

	ctx.EventManager().EmitEvent(
		sdk.NewEvent(
			eventType,
			sdk.NewAttribute(types.AttributeKeyProposalID, fmt.Sprintf("%d", proposal.Id)),
			sdk.NewAttribute(types.AttributeKeyProposalResult, types.AttributeValueProposalFailed),
		),
	)

	logger.Info(
		"proposal failed to decode; deleted",
		"proposal", proposal.Id,
		"expedited", proposal.Expedited,
		"title", proposal.Title,
		"results", errMsg,
	)

	return nil
}
