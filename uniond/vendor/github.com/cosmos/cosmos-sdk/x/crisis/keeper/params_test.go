package keeper_test

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func (s *KeeperTestSuite) TestParams() {
	// default params
	constantFee := sdk.NewCoin(sdk.DefaultBondDenom, sdk.NewInt(1000))

	testCases := []struct {
		name        string
		constantFee sdk.Coin
		expErr      bool
		expErrMsg   string
	}{
		{
			name:        "invalid constant fee",
			constantFee: sdk.Coin{},
			expErr:      true,
		},
		{
			name:        "negative constant fee",
			constantFee: sdk.Coin{Denom: sdk.DefaultBondDenom, Amount: sdk.NewInt(-1000)},
			expErr:      true,
		},
		{
			name:        "all good",
			constantFee: constantFee,
			expErr:      false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			expected := s.keeper.GetConstantFee(s.ctx)
			err := s.keeper.SetConstantFee(s.ctx, tc.constantFee)

			if tc.expErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				expected = tc.constantFee
				s.Require().NoError(err)
			}

			params := s.keeper.GetConstantFee(s.ctx)
			s.Require().Equal(expected, params)
		})
	}
}
