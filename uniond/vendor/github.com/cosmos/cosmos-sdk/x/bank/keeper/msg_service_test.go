package keeper_test

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
)

func (suite *KeeperTestSuite) TestMsgUpdateParams() {
	// default params
	params := banktypes.DefaultParams()

	testCases := []struct {
		name      string
		input     *banktypes.MsgUpdateParams
		expErr    bool
		expErrMsg string
	}{
		{
			name: "invalid authority",
			input: &banktypes.MsgUpdateParams{
				Authority: "invalid",
				Params:    params,
			},
			expErr:    true,
			expErrMsg: "invalid authority",
		},
		{
			name: "send enabled param",
			input: &banktypes.MsgUpdateParams{
				Authority: suite.bankKeeper.GetAuthority(),
				Params: banktypes.Params{
					SendEnabled: []*banktypes.SendEnabled{
						{Denom: "foo", Enabled: true},
					},
				},
			},
			expErr: false,
		},
		{
			name: "all good",
			input: &banktypes.MsgUpdateParams{
				Authority: suite.bankKeeper.GetAuthority(),
				Params:    params,
			},
			expErr: false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			_, err := suite.msgServer.UpdateParams(suite.ctx, tc.input)

			if tc.expErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				suite.Require().NoError(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestMsgSend() {
	origCoins := sdk.NewCoins(sdk.NewInt64Coin("sendableCoin", 100))
	suite.bankKeeper.SetSendEnabled(suite.ctx, origCoins.Denoms()[0], true)

	testCases := []struct {
		name      string
		input     *banktypes.MsgSend
		expErr    bool
		expErrMsg string
	}{
		{
			name: "invalid send to blocked address",
			input: &banktypes.MsgSend{
				FromAddress: minterAcc.GetAddress().String(),
				ToAddress:   accAddrs[4].String(),
				Amount:      origCoins,
			},
			expErr:    true,
			expErrMsg: "is not allowed to receive funds",
		},
		{
			name: "all good",
			input: &banktypes.MsgSend{
				FromAddress: minterAcc.GetAddress().String(),
				ToAddress:   baseAcc.Address,
				Amount:      origCoins,
			},
			expErr: false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.mockMintCoins(minterAcc)
			suite.bankKeeper.MintCoins(suite.ctx, minterAcc.Name, origCoins)
			if !tc.expErr {
				suite.mockSendCoins(suite.ctx, minterAcc, baseAcc.GetAddress())
			}
			_, err := suite.msgServer.Send(suite.ctx, tc.input)
			if tc.expErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				suite.Require().NoError(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestMsgMultiSend() {
	origDenom := "sendableCoin"
	origCoins := sdk.NewCoins(sdk.NewInt64Coin(origDenom, 100))
	sendCoins := sdk.NewCoins(sdk.NewInt64Coin(origDenom, 50))
	suite.bankKeeper.SetSendEnabled(suite.ctx, origDenom, true)

	testCases := []struct {
		name      string
		input     *banktypes.MsgMultiSend
		expErr    bool
		expErrMsg string
	}{
		{
			name: "invalid send to blocked address",
			input: &banktypes.MsgMultiSend{
				Inputs: []banktypes.Input{
					{Address: minterAcc.GetAddress().String(), Coins: origCoins},
				},
				Outputs: []banktypes.Output{
					{Address: accAddrs[0].String(), Coins: sendCoins},
					{Address: accAddrs[4].String(), Coins: sendCoins},
				},
			},
			expErr:    true,
			expErrMsg: "is not allowed to receive funds",
		},
		{
			name: "invalid send to blocked address",
			input: &banktypes.MsgMultiSend{
				Inputs: []banktypes.Input{
					{Address: minterAcc.GetAddress().String(), Coins: origCoins},
				},
				Outputs: []banktypes.Output{
					{Address: accAddrs[0].String(), Coins: sendCoins},
					{Address: accAddrs[1].String(), Coins: sendCoins},
				},
			},
			expErr: false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.mockMintCoins(minterAcc)
			suite.bankKeeper.MintCoins(suite.ctx, minterAcc.Name, origCoins)
			if !tc.expErr {
				suite.mockInputOutputCoins([]authtypes.AccountI{minterAcc}, accAddrs[:2])
			}
			_, err := suite.msgServer.MultiSend(suite.ctx, tc.input)
			if tc.expErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				suite.Require().NoError(err)
			}
		})
	}
}
