package keeper_test

import (
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

func (s *KeeperTestSuite) TestParams() {
	testCases := []struct {
		name      string
		input     types.Params
		expectErr bool
		expErrMsg string
	}{
		{
			name: "set invalid max memo characters",
			input: types.Params{
				MaxMemoCharacters:      0,
				TxSigLimit:             9,
				TxSizeCostPerByte:      5,
				SigVerifyCostED25519:   694,
				SigVerifyCostSecp256k1: 511,
			},
			expectErr: true,
			expErrMsg: "invalid max memo characters",
		},
		{
			name: "set invalid tx sig limit",
			input: types.Params{
				MaxMemoCharacters:      140,
				TxSigLimit:             0,
				TxSizeCostPerByte:      5,
				SigVerifyCostED25519:   694,
				SigVerifyCostSecp256k1: 511,
			},
			expectErr: true,
			expErrMsg: "invalid tx signature limit",
		},
		{
			name: "set invalid tx size cost per bytes",
			input: types.Params{
				MaxMemoCharacters:      140,
				TxSigLimit:             9,
				TxSizeCostPerByte:      0,
				SigVerifyCostED25519:   694,
				SigVerifyCostSecp256k1: 511,
			},
			expectErr: true,
			expErrMsg: "invalid tx size cost per byte",
		},
		{
			name: "set invalid sig verify cost ED25519",
			input: types.Params{
				MaxMemoCharacters:      140,
				TxSigLimit:             9,
				TxSizeCostPerByte:      5,
				SigVerifyCostED25519:   0,
				SigVerifyCostSecp256k1: 511,
			},
			expectErr: true,
			expErrMsg: "invalid ED25519 signature verification cost",
		},
		{
			name: "set invalid sig verify cost Secp256k1",
			input: types.Params{
				MaxMemoCharacters:      140,
				TxSigLimit:             9,
				TxSizeCostPerByte:      5,
				SigVerifyCostED25519:   694,
				SigVerifyCostSecp256k1: 0,
			},
			expectErr: true,
			expErrMsg: "invalid SECK256k1 signature verification cost",
		},
	}
	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			expected := s.accountKeeper.GetParams(s.ctx)
			err := s.accountKeeper.SetParams(s.ctx, tc.input)
			if tc.expectErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				expected = tc.input
				s.Require().NoError(err)
			}

			params := s.accountKeeper.GetParams(s.ctx)
			s.Require().Equal(expected, params)
		})
	}
}
