package keeper_test

import (
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

func (s *KeeperTestSuite) TestUpdateParams() {
	testCases := []struct {
		name      string
		req       *types.MsgUpdateParams
		expectErr bool
		expErrMsg string
	}{
		{
			name: "set invalid authority",
			req: &types.MsgUpdateParams{
				Authority: "foo",
			},
			expectErr: true,
			expErrMsg: "invalid authority",
		},
		{
			name: "set invalid max memo characters",
			req: &types.MsgUpdateParams{
				Authority: s.accountKeeper.GetAuthority(),
				Params: types.Params{
					MaxMemoCharacters:      0,
					TxSigLimit:             9,
					TxSizeCostPerByte:      5,
					SigVerifyCostED25519:   694,
					SigVerifyCostSecp256k1: 511,
				},
			},
			expectErr: true,
			expErrMsg: "invalid max memo characters",
		},
		{
			name: "set invalid tx sig limit",
			req: &types.MsgUpdateParams{
				Authority: s.accountKeeper.GetAuthority(),
				Params: types.Params{
					MaxMemoCharacters:      140,
					TxSigLimit:             0,
					TxSizeCostPerByte:      5,
					SigVerifyCostED25519:   694,
					SigVerifyCostSecp256k1: 511,
				},
			},
			expectErr: true,
			expErrMsg: "invalid tx signature limit",
		},
		{
			name: "set invalid tx size cost per bytes",
			req: &types.MsgUpdateParams{
				Authority: s.accountKeeper.GetAuthority(),
				Params: types.Params{
					MaxMemoCharacters:      140,
					TxSigLimit:             9,
					TxSizeCostPerByte:      0,
					SigVerifyCostED25519:   694,
					SigVerifyCostSecp256k1: 511,
				},
			},
			expectErr: true,
			expErrMsg: "invalid tx size cost per byte",
		},
		{
			name: "set invalid sig verify cost ED25519",
			req: &types.MsgUpdateParams{
				Authority: s.accountKeeper.GetAuthority(),
				Params: types.Params{
					MaxMemoCharacters:      140,
					TxSigLimit:             9,
					TxSizeCostPerByte:      5,
					SigVerifyCostED25519:   0,
					SigVerifyCostSecp256k1: 511,
				},
			},
			expectErr: true,
			expErrMsg: "invalid ED25519 signature verification cost",
		},
		{
			name: "set invalid sig verify cost Secp256k1",
			req: &types.MsgUpdateParams{
				Authority: s.accountKeeper.GetAuthority(),
				Params: types.Params{
					MaxMemoCharacters:      140,
					TxSigLimit:             9,
					TxSizeCostPerByte:      5,
					SigVerifyCostED25519:   694,
					SigVerifyCostSecp256k1: 0,
				},
			},
			expectErr: true,
			expErrMsg: "invalid SECK256k1 signature verification cost",
		},
	}

	for _, tc := range testCases {
		tc := tc
		s.Run(tc.name, func() {
			_, err := s.msgServer.UpdateParams(s.ctx, tc.req)
			if tc.expectErr {
				s.Require().Error(err)
				s.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				s.Require().NoError(err)
			}
		})
	}
}
