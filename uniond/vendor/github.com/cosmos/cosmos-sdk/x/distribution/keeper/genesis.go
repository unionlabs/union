package keeper

import (
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/distribution/types"
)

// InitGenesis sets distribution information for genesis
func (k Keeper) InitGenesis(ctx sdk.Context, data types.GenesisState) {
	var moduleHoldings sdk.DecCoins

	k.SetFeePool(ctx, data.FeePool)

	if err := k.SetParams(ctx, data.Params); err != nil {
		panic(err)
	}

	for _, dwi := range data.DelegatorWithdrawInfos {
		delegatorAddress := sdk.MustAccAddressFromBech32(dwi.DelegatorAddress)
		withdrawAddress := sdk.MustAccAddressFromBech32(dwi.WithdrawAddress)
		k.SetDelegatorWithdrawAddr(ctx, delegatorAddress, withdrawAddress)
	}

	var previousProposer sdk.ConsAddress
	if data.PreviousProposer != "" {
		var err error
		previousProposer, err = sdk.ConsAddressFromBech32(data.PreviousProposer)
		if err != nil {
			panic(err)
		}
	}

	k.SetPreviousProposerConsAddr(ctx, previousProposer)

	for _, rew := range data.OutstandingRewards {
		valAddr, err := sdk.ValAddressFromBech32(rew.ValidatorAddress)
		if err != nil {
			panic(err)
		}
		k.SetValidatorOutstandingRewards(ctx, valAddr, types.ValidatorOutstandingRewards{Rewards: rew.OutstandingRewards})
		moduleHoldings = moduleHoldings.Add(rew.OutstandingRewards...)
	}
	for _, acc := range data.ValidatorAccumulatedCommissions {
		valAddr, err := sdk.ValAddressFromBech32(acc.ValidatorAddress)
		if err != nil {
			panic(err)
		}
		k.SetValidatorAccumulatedCommission(ctx, valAddr, acc.Accumulated)
	}
	for _, his := range data.ValidatorHistoricalRewards {
		valAddr, err := sdk.ValAddressFromBech32(his.ValidatorAddress)
		if err != nil {
			panic(err)
		}
		k.SetValidatorHistoricalRewards(ctx, valAddr, his.Period, his.Rewards)
	}
	for _, cur := range data.ValidatorCurrentRewards {
		valAddr, err := sdk.ValAddressFromBech32(cur.ValidatorAddress)
		if err != nil {
			panic(err)
		}
		k.SetValidatorCurrentRewards(ctx, valAddr, cur.Rewards)
	}
	for _, del := range data.DelegatorStartingInfos {
		valAddr, err := sdk.ValAddressFromBech32(del.ValidatorAddress)
		if err != nil {
			panic(err)
		}
		delegatorAddress := sdk.MustAccAddressFromBech32(del.DelegatorAddress)

		k.SetDelegatorStartingInfo(ctx, valAddr, delegatorAddress, del.StartingInfo)
	}
	for _, evt := range data.ValidatorSlashEvents {
		valAddr, err := sdk.ValAddressFromBech32(evt.ValidatorAddress)
		if err != nil {
			panic(err)
		}
		k.SetValidatorSlashEvent(ctx, valAddr, evt.Height, evt.Period, evt.ValidatorSlashEvent)
	}

	moduleHoldings = moduleHoldings.Add(data.FeePool.CommunityPool...)
	moduleHoldingsInt, _ := moduleHoldings.TruncateDecimal()

	// check if the module account exists
	moduleAcc := k.GetDistributionAccount(ctx)
	if moduleAcc == nil {
		panic(fmt.Sprintf("%s module account has not been set", types.ModuleName))
	}

	balances := k.bankKeeper.GetAllBalances(ctx, moduleAcc.GetAddress())
	if balances.IsZero() {
		k.authKeeper.SetModuleAccount(ctx, moduleAcc)
	}
	if !balances.IsEqual(moduleHoldingsInt) {
		panic(fmt.Sprintf("distribution module balance does not match the module holdings: %s <-> %s", balances, moduleHoldingsInt))
	}
}

// ExportGenesis returns a GenesisState for a given context and keeper.
func (k Keeper) ExportGenesis(ctx sdk.Context) *types.GenesisState {
	feePool := k.GetFeePool(ctx)
	params := k.GetParams(ctx)

	dwi := make([]types.DelegatorWithdrawInfo, 0)
	k.IterateDelegatorWithdrawAddrs(ctx, func(del sdk.AccAddress, addr sdk.AccAddress) (stop bool) {
		dwi = append(dwi, types.DelegatorWithdrawInfo{
			DelegatorAddress: del.String(),
			WithdrawAddress:  addr.String(),
		})
		return false
	})

	pp := k.GetPreviousProposerConsAddr(ctx)
	outstanding := make([]types.ValidatorOutstandingRewardsRecord, 0)

	k.IterateValidatorOutstandingRewards(ctx,
		func(addr sdk.ValAddress, rewards types.ValidatorOutstandingRewards) (stop bool) {
			outstanding = append(outstanding, types.ValidatorOutstandingRewardsRecord{
				ValidatorAddress:   addr.String(),
				OutstandingRewards: rewards.Rewards,
			})
			return false
		},
	)

	acc := make([]types.ValidatorAccumulatedCommissionRecord, 0)
	k.IterateValidatorAccumulatedCommissions(ctx,
		func(addr sdk.ValAddress, commission types.ValidatorAccumulatedCommission) (stop bool) {
			acc = append(acc, types.ValidatorAccumulatedCommissionRecord{
				ValidatorAddress: addr.String(),
				Accumulated:      commission,
			})
			return false
		},
	)

	his := make([]types.ValidatorHistoricalRewardsRecord, 0)
	k.IterateValidatorHistoricalRewards(ctx,
		func(val sdk.ValAddress, period uint64, rewards types.ValidatorHistoricalRewards) (stop bool) {
			his = append(his, types.ValidatorHistoricalRewardsRecord{
				ValidatorAddress: val.String(),
				Period:           period,
				Rewards:          rewards,
			})
			return false
		},
	)

	cur := make([]types.ValidatorCurrentRewardsRecord, 0)
	k.IterateValidatorCurrentRewards(ctx,
		func(val sdk.ValAddress, rewards types.ValidatorCurrentRewards) (stop bool) {
			cur = append(cur, types.ValidatorCurrentRewardsRecord{
				ValidatorAddress: val.String(),
				Rewards:          rewards,
			})
			return false
		},
	)

	dels := make([]types.DelegatorStartingInfoRecord, 0)
	k.IterateDelegatorStartingInfos(ctx,
		func(val sdk.ValAddress, del sdk.AccAddress, info types.DelegatorStartingInfo) (stop bool) {
			dels = append(dels, types.DelegatorStartingInfoRecord{
				ValidatorAddress: val.String(),
				DelegatorAddress: del.String(),
				StartingInfo:     info,
			})
			return false
		},
	)

	slashes := make([]types.ValidatorSlashEventRecord, 0)
	k.IterateValidatorSlashEvents(ctx,
		func(val sdk.ValAddress, height uint64, event types.ValidatorSlashEvent) (stop bool) {
			slashes = append(slashes, types.ValidatorSlashEventRecord{
				ValidatorAddress:    val.String(),
				Height:              height,
				Period:              event.ValidatorPeriod,
				ValidatorSlashEvent: event,
			})
			return false
		},
	)

	return types.NewGenesisState(params, feePool, dwi, pp, outstanding, acc, his, cur, dels, slashes)
}
