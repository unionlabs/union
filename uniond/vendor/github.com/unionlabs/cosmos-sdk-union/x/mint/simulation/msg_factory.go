package simulation

import (
	"context"

	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/mint/types"

	"github.com/cosmos/cosmos-sdk/simsx"
)

// MsgUpdateParamsFactory creates a gov proposal for param updates
func MsgUpdateParamsFactory() simsx.SimMsgFactoryFn[*types.MsgUpdateParams] {
	return func(_ context.Context, testData *simsx.ChainDataSource, reporter simsx.SimulationReporter) ([]simsx.SimAccount, *types.MsgUpdateParams) {
		r := testData.Rand()
		params := types.DefaultParams()
		params.BlocksPerYear = r.Uint64InRange(1, 1_000_000)
		params.GoalBonded = sdkmath.LegacyNewDecWithPrec(int64(r.IntInRange(1, 100)), 2)
		params.InflationMin = sdkmath.LegacyNewDecWithPrec(int64(r.IntInRange(1, 50)), 2)
		params.InflationMax = sdkmath.LegacyNewDecWithPrec(int64(r.IntInRange(50, 100)), 2)
		params.InflationRateChange = sdkmath.LegacyNewDecWithPrec(int64(r.IntInRange(1, 100)), 2)
		params.MintDenom = r.StringN(10)

		return nil, &types.MsgUpdateParams{
			Authority: testData.ModuleAccountAddress(reporter, "gov"),
			Params:    params,
		}
	}
}
