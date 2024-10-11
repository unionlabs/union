package simulation

import (
	"context"
	"errors"
	"time"

	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/slashing/keeper"
	"cosmossdk.io/x/slashing/types"

	"github.com/cosmos/cosmos-sdk/simsx"
)

func MsgUnjailFactory(k keeper.Keeper, sk types.StakingKeeper) simsx.SimMsgFactoryX {
	return simsx.NewSimMsgFactoryWithDeliveryResultHandler[*types.MsgUnjail](func(ctx context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *types.MsgUnjail, simsx.SimDeliveryResultHandler) {
		allVals, err := sk.GetAllValidators(ctx)
		if err != nil {
			reporter.Skip(err.Error())
			return nil, nil, nil
		}
		validator := simsx.OneOf(testData.Rand(), allVals)
		if !validator.IsJailed() {
			reporter.Skip("validator not jailed")
			return nil, nil, nil
		}
		if validator.InvalidExRate() {
			reporter.Skip("validator with invalid exchange rate")
			return nil, nil, nil
		}

		info, err := k.ValidatorSigningInfo.Get(ctx, must(validator.GetConsAddr()))
		if err != nil {
			reporter.Skip(err.Error())
			return nil, nil, nil
		}
		valOperBz := must(sk.ValidatorAddressCodec().StringToBytes(validator.GetOperator()))
		valOper := testData.GetAccountbyAccAddr(reporter, valOperBz)
		if reporter.IsSkipped() {
			return nil, nil, nil
		}

		selfDel, err := sk.Delegation(ctx, valOper.Address, valOperBz)
		if selfDel == nil || err != nil {
			reporter.Skip("no self delegation")
			return nil, nil, nil
		}
		var handler simsx.SimDeliveryResultHandler
		// result should fail if:
		// - validator cannot be unjailed due to tombstone
		// - validator is still in jailed period
		// - self delegation too low
		if info.Tombstoned ||
			simsx.BlockTime(ctx).Before(info.JailedUntil) ||
			selfDel.GetShares().IsNil() ||
			validator.TokensFromShares(selfDel.GetShares()).TruncateInt().LT(validator.GetMinSelfDelegation()) {
			handler = func(err error) error {
				if err == nil {
					switch {
					case info.Tombstoned:
						return errors.New("validator should not have been unjailed if validator tombstoned")
					case simsx.BlockTime(ctx).Before(info.JailedUntil):
						return errors.New("validator unjailed while validator still in jail period")
					case selfDel.GetShares().IsNil() || validator.TokensFromShares(selfDel.GetShares()).TruncateInt().LT(validator.GetMinSelfDelegation()):
						return errors.New("validator unjailed even though self-delegation too low")
					}
				}
				return nil
			}
		}
		return []simsx.SimAccount{valOper}, types.NewMsgUnjail(validator.GetOperator()), handler
	})
}

// MsgUpdateParamsFactory creates a gov proposal for param updates
func MsgUpdateParamsFactory() simsx.SimMsgFactoryFn[*types.MsgUpdateParams] {
	return func(_ context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *types.MsgUpdateParams) {
		r := testData.Rand()
		params := types.DefaultParams()
		params.DowntimeJailDuration = time.Duration(r.Timestamp().UnixNano())
		params.SignedBlocksWindow = int64(r.IntInRange(1, 1000))
		params.MinSignedPerWindow = sdkmath.LegacyNewDecWithPrec(int64(r.IntInRange(1, 100)), 2)
		params.SlashFractionDoubleSign = sdkmath.LegacyNewDecWithPrec(int64(r.IntInRange(1, 100)), 2)
		params.SlashFractionDowntime = sdkmath.LegacyNewDecWithPrec(int64(r.IntInRange(1, 100)), 2)

		return nil, &types.MsgUpdateParams{
			Authority: testData.ModuleAccountAddress(reporter, "gov"),
			Params:    params,
		}
	}
}

func must[T any](r T, err error) T {
	if err != nil {
		panic(err)
	}
	return r
}
