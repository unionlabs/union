package keeper

import (
	"context"
	"fmt"
	"time"

	"cosmossdk.io/math"
	"cosmossdk.io/x/protocolpool/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

func (k Keeper) InitGenesis(ctx context.Context, data *types.GenesisState) error {
	currentTime := k.HeaderService.HeaderInfo(ctx).Time
	for _, cf := range data.ContinuousFund {
		// ignore expired ContinuousFunds
		if cf.Expiry != nil && cf.Expiry.Before(currentTime) {
			continue
		}

		recipientAddress, err := k.authKeeper.AddressCodec().StringToBytes(cf.Recipient)
		if err != nil {
			return fmt.Errorf("failed to decode recipient address: %w", err)
		}
		if err := k.ContinuousFund.Set(ctx, recipientAddress, *cf); err != nil {
			return fmt.Errorf("failed to set continuous fund for recipient %s: %w", recipientAddress, err)
		}
	}
	for _, budget := range data.Budget {
		// Validate StartTime
		if budget.LastClaimedAt == nil || budget.LastClaimedAt.IsZero() {
			budget.LastClaimedAt = &currentTime
		}
		// ignore budgets with period <= 0 || nil
		if budget.Period == nil || (budget.Period != nil && budget.Period.Seconds() <= 0) {
			continue
		}

		// ignore budget with start time < currentTime
		if budget.LastClaimedAt.Before(currentTime) {
			continue
		}

		recipientAddress, err := k.authKeeper.AddressCodec().StringToBytes(budget.RecipientAddress)
		if err != nil {
			return fmt.Errorf("failed to decode recipient address: %w", err)
		}
		if err = k.BudgetProposal.Set(ctx, recipientAddress, *budget); err != nil {
			return fmt.Errorf("failed to set budget for recipient %s: %w", recipientAddress, err)
		}
	}

	if err := k.LastBalance.Set(ctx, data.LastBalance); err != nil {
		return fmt.Errorf("failed to set last balance: %w", err)
	}

	totalToBeDistributed := math.ZeroInt()
	for _, distribution := range data.Distributions {
		totalToBeDistributed = totalToBeDistributed.Add(distribution.Amount)
		if err := k.Distributions.Set(ctx, *distribution.Time, distribution.Amount); err != nil {
			return fmt.Errorf("failed to set distribution: %w", err)
		}
	}

	// sanity check to avoid trying to distribute more than what is available
	if data.LastBalance.LT(totalToBeDistributed) {
		return fmt.Errorf("total to be distributed is greater than the last balance")
	}

	return nil
}

func (k Keeper) ExportGenesis(ctx context.Context) (*types.GenesisState, error) {
	var cf []*types.ContinuousFund
	err := k.ContinuousFund.Walk(ctx, nil, func(key sdk.AccAddress, value types.ContinuousFund) (stop bool, err error) {
		recipient, err := k.authKeeper.AddressCodec().BytesToString(key)
		if err != nil {
			return true, err
		}
		cf = append(cf, &types.ContinuousFund{
			Recipient:  recipient,
			Percentage: value.Percentage,
			Expiry:     value.Expiry,
		})
		return false, nil
	})
	if err != nil {
		return nil, err
	}

	var budget []*types.Budget
	err = k.BudgetProposal.Walk(ctx, nil, func(key sdk.AccAddress, value types.Budget) (stop bool, err error) {
		recipient, err := k.authKeeper.AddressCodec().BytesToString(key)
		if err != nil {
			return true, err
		}
		budget = append(budget, &types.Budget{
			RecipientAddress: recipient,
			ClaimedAmount:    value.ClaimedAmount,
			LastClaimedAt:    value.LastClaimedAt,
			TranchesLeft:     value.TranchesLeft,
			Period:           value.Period,
			BudgetPerTranche: value.BudgetPerTranche,
		})
		return false, nil
	})
	if err != nil {
		return nil, err
	}

	genState := types.NewGenesisState(cf, budget)

	genState.LastBalance, err = k.LastBalance.Get(ctx)
	if err != nil {
		return nil, err
	}

	err = k.Distributions.Walk(ctx, nil, func(key time.Time, value math.Int) (stop bool, err error) {
		genState.Distributions = append(genState.Distributions, &types.Distribution{
			Time:   &key,
			Amount: value,
		})

		return false, nil
	})
	if err != nil {
		return nil, err
	}

	return genState, nil
}
