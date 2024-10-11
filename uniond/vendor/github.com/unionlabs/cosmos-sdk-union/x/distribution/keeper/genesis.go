package keeper

import (
	"context"
	"fmt"

	"cosmossdk.io/collections"
	"cosmossdk.io/x/distribution/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// InitGenesis sets distribution information for genesis
func (k Keeper) InitGenesis(ctx context.Context, data types.GenesisState) error {
	var moduleHoldings sdk.DecCoins

	if err := k.FeePool.Set(ctx, data.FeePool); err != nil {
		return err
	}

	if err := k.Params.Set(ctx, data.Params); err != nil {
		return err
	}

	for _, dwi := range data.DelegatorWithdrawInfos {
		delegatorAddress, err := k.authKeeper.AddressCodec().StringToBytes(dwi.DelegatorAddress)
		if err != nil {
			return err
		}
		withdrawAddress, err := k.authKeeper.AddressCodec().StringToBytes(dwi.WithdrawAddress)
		if err != nil {
			return err
		}
		err = k.DelegatorsWithdrawAddress.Set(ctx, delegatorAddress, withdrawAddress)
		if err != nil {
			return err
		}
	}

	for _, rew := range data.OutstandingRewards {
		valAddr, err := k.stakingKeeper.ValidatorAddressCodec().StringToBytes(rew.ValidatorAddress)
		if err != nil {
			return err
		}
		err = k.ValidatorOutstandingRewards.Set(ctx, valAddr, types.ValidatorOutstandingRewards{Rewards: rew.OutstandingRewards})
		if err != nil {
			return err
		}
		moduleHoldings = moduleHoldings.Add(rew.OutstandingRewards...)
	}
	for _, acc := range data.ValidatorAccumulatedCommissions {
		valAddr, err := k.stakingKeeper.ValidatorAddressCodec().StringToBytes(acc.ValidatorAddress)
		if err != nil {
			return err
		}
		err = k.ValidatorsAccumulatedCommission.Set(ctx, valAddr, acc.Accumulated)
		if err != nil {
			return err
		}
	}
	for _, his := range data.ValidatorHistoricalRewards {
		valAddr, err := k.stakingKeeper.ValidatorAddressCodec().StringToBytes(his.ValidatorAddress)
		if err != nil {
			return err
		}
		err = k.ValidatorHistoricalRewards.Set(ctx, collections.Join(sdk.ValAddress(valAddr), his.Period), his.Rewards)
		if err != nil {
			return err
		}
	}
	for _, cur := range data.ValidatorCurrentRewards {
		valAddr, err := k.stakingKeeper.ValidatorAddressCodec().StringToBytes(cur.ValidatorAddress)
		if err != nil {
			return err
		}
		err = k.ValidatorCurrentRewards.Set(ctx, valAddr, cur.Rewards)
		if err != nil {
			return err
		}
	}
	for _, del := range data.DelegatorStartingInfos {
		valAddr, err := k.stakingKeeper.ValidatorAddressCodec().StringToBytes(del.ValidatorAddress)
		if err != nil {
			return err
		}
		delegatorAddress, err := k.authKeeper.AddressCodec().StringToBytes(del.DelegatorAddress)
		if err != nil {
			return err
		}

		err = k.DelegatorStartingInfo.Set(ctx, collections.Join(sdk.ValAddress(valAddr), sdk.AccAddress(delegatorAddress)), del.StartingInfo)
		if err != nil {
			return err
		}
	}
	for _, evt := range data.ValidatorSlashEvents {
		valAddr, err := k.stakingKeeper.ValidatorAddressCodec().StringToBytes(evt.ValidatorAddress)
		if err != nil {
			return err
		}

		err = k.ValidatorSlashEvents.Set(
			ctx,
			collections.Join3(
				sdk.ValAddress(valAddr),
				evt.Height,
				evt.Period,
			),
			evt.ValidatorSlashEvent,
		)
		if err != nil {
			return err
		}
	}

	moduleHoldings = moduleHoldings.Add(data.FeePool.DecimalPool...)
	moduleHoldingsInt, _ := moduleHoldings.TruncateDecimal()

	// check if the module account exists
	moduleAcc := k.GetDistributionAccount(ctx)
	if moduleAcc == nil {
		return fmt.Errorf("%s module account has not been set", types.ModuleName)
	}

	balances := k.bankKeeper.GetAllBalances(ctx, moduleAcc.GetAddress())
	if balances.IsZero() {
		k.authKeeper.SetModuleAccount(ctx, moduleAcc)
	}
	if !balances.Equal(moduleHoldingsInt) {
		return fmt.Errorf("distribution module balance does not match the module holdings: %s <-> %s", balances, moduleHoldingsInt)
	}
	return nil
}

// ExportGenesis returns a GenesisState for a given context and keeper.
func (k Keeper) ExportGenesis(ctx context.Context) (*types.GenesisState, error) {
	feePool, err := k.FeePool.Get(ctx)
	if err != nil {
		return nil, err
	}

	params, err := k.Params.Get(ctx)
	if err != nil {
		return nil, err
	}

	var dwi []types.DelegatorWithdrawInfo
	err = k.DelegatorsWithdrawAddress.Walk(ctx, nil, func(key, value sdk.AccAddress) (stop bool, err error) {
		keyAddr, err := k.authKeeper.AddressCodec().BytesToString(key)
		if err != nil {
			return true, err
		}
		valueAddr, err := k.authKeeper.AddressCodec().BytesToString(value)
		if err != nil {
			return true, err
		}
		dwi = append(dwi, types.DelegatorWithdrawInfo{
			DelegatorAddress: keyAddr,
			WithdrawAddress:  valueAddr,
		})
		return false, nil
	})
	if err != nil {
		return nil, err
	}

	outstanding := make([]types.ValidatorOutstandingRewardsRecord, 0)

	err = k.ValidatorOutstandingRewards.Walk(ctx, nil, func(addr sdk.ValAddress, rewards types.ValidatorOutstandingRewards) (stop bool, err error) {
		valAddr, err := k.stakingKeeper.ValidatorAddressCodec().BytesToString(addr)
		if err != nil {
			return true, err
		}

		outstanding = append(outstanding, types.ValidatorOutstandingRewardsRecord{
			ValidatorAddress:   valAddr,
			OutstandingRewards: rewards.Rewards,
		})
		return false, nil
	},
	)
	if err != nil {
		return nil, err
	}

	acc := make([]types.ValidatorAccumulatedCommissionRecord, 0)
	err = k.ValidatorsAccumulatedCommission.Walk(ctx, nil, func(addr sdk.ValAddress, commission types.ValidatorAccumulatedCommission) (stop bool, err error) {
		valAddr, err := k.stakingKeeper.ValidatorAddressCodec().BytesToString(addr)
		if err != nil {
			return true, err
		}

		acc = append(acc, types.ValidatorAccumulatedCommissionRecord{
			ValidatorAddress: valAddr,
			Accumulated:      commission,
		})
		return false, nil
	})
	if err != nil {
		return nil, err
	}

	his := make([]types.ValidatorHistoricalRewardsRecord, 0)
	err = k.ValidatorHistoricalRewards.Walk(ctx, nil,
		func(key collections.Pair[sdk.ValAddress, uint64], rewards types.ValidatorHistoricalRewards) (stop bool, err error) {
			valAddr, err := k.stakingKeeper.ValidatorAddressCodec().BytesToString(key.K1())
			if err != nil {
				return true, err
			}

			his = append(his, types.ValidatorHistoricalRewardsRecord{
				ValidatorAddress: valAddr,
				Period:           key.K2(),
				Rewards:          rewards,
			})
			return false, nil
		},
	)
	if err != nil {
		return nil, err
	}

	cur := make([]types.ValidatorCurrentRewardsRecord, 0)
	err = k.ValidatorCurrentRewards.Walk(ctx, nil,
		func(val sdk.ValAddress, rewards types.ValidatorCurrentRewards) (stop bool, err error) {
			valAddr, err := k.stakingKeeper.ValidatorAddressCodec().BytesToString(val)
			if err != nil {
				return true, err
			}

			cur = append(cur, types.ValidatorCurrentRewardsRecord{
				ValidatorAddress: valAddr,
				Rewards:          rewards,
			})
			return false, nil
		},
	)
	if err != nil {
		return nil, err
	}

	dels := make([]types.DelegatorStartingInfoRecord, 0)
	err = k.DelegatorStartingInfo.Walk(ctx, nil, func(key collections.Pair[sdk.ValAddress, sdk.AccAddress], value types.DelegatorStartingInfo) (stop bool, err error) {
		delAddr, err := k.authKeeper.AddressCodec().BytesToString(key.K2())
		if err != nil {
			return true, err
		}

		valAddr, err := k.stakingKeeper.ValidatorAddressCodec().BytesToString(key.K1())
		if err != nil {
			return true, err
		}

		dels = append(dels, types.DelegatorStartingInfoRecord{
			DelegatorAddress: delAddr,
			ValidatorAddress: valAddr,
			StartingInfo:     value,
		})
		return false, nil
	})
	if err != nil {
		return nil, err
	}

	slashes := make([]types.ValidatorSlashEventRecord, 0)
	err = k.ValidatorSlashEvents.Walk(
		ctx,
		nil,
		func(key collections.Triple[sdk.ValAddress, uint64, uint64], event types.ValidatorSlashEvent) (stop bool, err error) {
			valAddr, err := k.stakingKeeper.ValidatorAddressCodec().BytesToString(key.K1())
			if err != nil {
				return true, err
			}

			slashes = append(slashes, types.ValidatorSlashEventRecord{
				ValidatorAddress:    valAddr,
				Height:              key.K2(),
				Period:              event.ValidatorPeriod,
				ValidatorSlashEvent: event,
			})
			return false, nil
		},
	)
	if err != nil {
		return nil, err
	}

	return types.NewGenesisState(params, feePool, dwi, outstanding, acc, his, cur, dels, slashes), nil
}
