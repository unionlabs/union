package keeper_test

import (
	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/mint/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

func (s *KeeperTestSuite) TestUpdateParams() {
	testCases := []struct {
		name      string
		request   *types.MsgUpdateParams
		expectErr bool
	}{
		{
			name: "set invalid authority (not an address)",
			request: &types.MsgUpdateParams{
				Authority: "foo",
			},
			expectErr: true,
		},
		{
			name: "set invalid authority (not defined authority)",
			request: &types.MsgUpdateParams{
				Authority: "cosmos139f7kncmglres2nf3h4hc4tade85ekfr8sulz5",
			},
			expectErr: true,
		},
		{
			name: "set invalid params",
			request: &types.MsgUpdateParams{
				Authority: s.mintKeeper.GetAuthority(),
				Params: types.Params{
					MintDenom:           sdk.DefaultBondDenom,
					InflationRateChange: sdkmath.LegacyNewDecWithPrec(-13, 2),
					InflationMax:        sdkmath.LegacyNewDecWithPrec(20, 2),
					InflationMin:        sdkmath.LegacyNewDecWithPrec(7, 2),
					GoalBonded:          sdkmath.LegacyNewDecWithPrec(67, 2),
					BlocksPerYear:       uint64(60 * 60 * 8766 / 5),
				},
			},
			expectErr: true,
		},
		{
			name: "set full valid params",
			request: &types.MsgUpdateParams{
				Authority: s.mintKeeper.GetAuthority(),
				Params: types.Params{
					MintDenom:           sdk.DefaultBondDenom,
					InflationRateChange: sdkmath.LegacyNewDecWithPrec(8, 2),
					InflationMax:        sdkmath.LegacyNewDecWithPrec(20, 2),
					InflationMin:        sdkmath.LegacyNewDecWithPrec(2, 2),
					GoalBonded:          sdkmath.LegacyNewDecWithPrec(37, 2),
					BlocksPerYear:       uint64(60 * 60 * 8766 / 5),
					MaxSupply:           sdkmath.ZeroInt(), // infinite supply
				},
			},
			expectErr: false,
		},
	}

	for _, tc := range testCases {
		s.Run(tc.name, func() {
			_, err := s.msgServer.UpdateParams(s.ctx, tc.request)
			if tc.expectErr {
				s.Require().Error(err)
			} else {
				s.Require().NoError(err)
			}
		})
	}
}
