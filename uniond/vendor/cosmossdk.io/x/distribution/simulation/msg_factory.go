package simulation

import (
	"context"
	"errors"

	"cosmossdk.io/collections"
	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/distribution/keeper"
	"cosmossdk.io/x/distribution/types"

	"github.com/cosmos/cosmos-sdk/simsx"
)

func MsgSetWithdrawAddressFactory(k keeper.Keeper) simsx.SimMsgFactoryFn[*types.MsgSetWithdrawAddress] {
	return func(ctx context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *types.MsgSetWithdrawAddress) {
		switch enabled, err := k.GetWithdrawAddrEnabled(ctx); {
		case err != nil:
			reporter.Skip("error getting params")
			return nil, nil
		case !enabled:
			reporter.Skip("withdrawal is not enabled")
			return nil, nil
		}
		delegator := testData.AnyAccount(reporter)
		withdrawer := testData.AnyAccount(reporter, simsx.ExcludeAccounts(delegator))
		msg := types.NewMsgSetWithdrawAddress(delegator.AddressBech32, withdrawer.AddressBech32)
		return []simsx.SimAccount{delegator}, msg
	}
}

func MsgWithdrawDelegatorRewardFactory(k keeper.Keeper, sk types.StakingKeeper) simsx.SimMsgFactoryFn[*types.MsgWithdrawDelegatorReward] {
	return func(ctx context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *types.MsgWithdrawDelegatorReward) {
		delegator := testData.AnyAccount(reporter)

		delegations, err := sk.GetAllDelegatorDelegations(ctx, delegator.Address)
		switch {
		case err != nil:
			reporter.Skipf("error getting delegations: %v", err)
			return nil, nil
		case len(delegations) == 0:
			reporter.Skip("no delegations found")
			return nil, nil
		}
		delegation := delegations[testData.Rand().Intn(len(delegations))]

		valAddr, err := sk.ValidatorAddressCodec().StringToBytes(delegation.GetValidatorAddr())
		if err != nil {
			reporter.Skip(err.Error())
			return nil, nil
		}

		var valOper string
		switch validator, err := sk.Validator(ctx, valAddr); {
		case err != nil:
			reporter.Skip(err.Error())
			return nil, nil
		case validator == nil:
			reporter.Skipf("validator %s not found", delegation.GetValidatorAddr())
			return nil, nil
		default:
			valOper = validator.GetOperator()
		}
		// get outstanding rewards so we can first check if the withdrawable coins are sendable
		outstanding, err := k.GetValidatorOutstandingRewardsCoins(ctx, valAddr)
		if err != nil {
			reporter.Skipf("get outstanding rewards: %v", err)
			return nil, nil
		}

		for _, v := range outstanding {
			if !testData.IsSendEnabledDenom(v.Denom) {
				reporter.Skipf("denom send not enabled: " + v.Denom)
				return nil, nil
			}
		}

		msg := types.NewMsgWithdrawDelegatorReward(delegator.AddressBech32, valOper)
		return []simsx.SimAccount{delegator}, msg
	}
}

func MsgWithdrawValidatorCommissionFactory(k keeper.Keeper, sk types.StakingKeeper) simsx.SimMsgFactoryFn[*types.MsgWithdrawValidatorCommission] {
	return func(ctx context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *types.MsgWithdrawValidatorCommission) {
		allVals, err := sk.GetAllValidators(ctx)
		if err != nil {
			reporter.Skip(err.Error())
			return nil, nil
		}
		val := simsx.OneOf(testData.Rand(), allVals)
		valAddrBz, err := sk.ValidatorAddressCodec().StringToBytes(val.GetOperator())
		if err != nil {
			reporter.Skip(err.Error())
			return nil, nil
		}

		commission, err := k.ValidatorsAccumulatedCommission.Get(ctx, valAddrBz)
		if err != nil && !errors.Is(err, collections.ErrNotFound) {
			reporter.Skip(err.Error())
			return nil, nil
		}

		if commission.Commission.IsZero() {
			reporter.Skip("validator commission is zero")
			return nil, nil
		}
		msg := types.NewMsgWithdrawValidatorCommission(val.GetOperator())
		valAccount := testData.GetAccountbyAccAddr(reporter, valAddrBz)
		return []simsx.SimAccount{valAccount}, msg
	}
}

func MsgUpdateParamsFactory() simsx.SimMsgFactoryFn[*types.MsgUpdateParams] {
	return func(_ context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *types.MsgUpdateParams) {
		r := testData.Rand()
		params := types.DefaultParams()
		params.CommunityTax = r.DecN(sdkmath.LegacyNewDec(1))
		params.WithdrawAddrEnabled = r.Intn(2) == 0

		return nil, &types.MsgUpdateParams{
			Authority: testData.ModuleAccountAddress(reporter, "gov"),
			Params:    params,
		}
	}
}
