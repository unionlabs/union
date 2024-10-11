package keeper_test

import (
	banktypes "cosmossdk.io/x/bank/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

var govAcc = authtypes.NewEmptyModuleAccount(banktypes.GovModuleName, authtypes.Minter)

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
			expErr:    true,
			expErrMsg: "use of send_enabled in params is no longer supported",
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
	atom0 := sdk.NewCoins(sdk.NewInt64Coin("atom", 0))
	atom123eth0 := sdk.Coins{sdk.NewInt64Coin("atom", 123), sdk.NewInt64Coin("eth", 0)}

	acc4Addr, err := suite.authKeeper.AddressCodec().BytesToString(accAddrs[4])
	suite.Require().NoError(err)
	minterAccAddr, err := suite.authKeeper.AddressCodec().BytesToString(minterAcc.GetAddress())
	suite.Require().NoError(err)

	testCases := []struct {
		name      string
		input     *banktypes.MsgSend
		expErr    bool
		expErrMsg string
	}{
		{
			name: "invalid send to blocked address",
			input: &banktypes.MsgSend{
				FromAddress: minterAccAddr,
				ToAddress:   acc4Addr,
				Amount:      origCoins,
			},
			expErr:    true,
			expErrMsg: "is not allowed to receive funds",
		},
		{
			name: "invalid coins",
			input: &banktypes.MsgSend{
				FromAddress: minterAccAddr,
				ToAddress:   baseAcc.Address,
				Amount:      atom0,
			},
			expErr:    true,
			expErrMsg: "invalid coins",
		},
		{
			name: "123atom,0eth: invalid coins",
			input: &banktypes.MsgSend{
				FromAddress: minterAccAddr,
				ToAddress:   baseAcc.Address,
				Amount:      atom123eth0,
			},
			expErr:    true,
			expErrMsg: "123atom,0eth: invalid coins",
		},
		{
			name: "invalid from address: empty address string is not allowed: invalid address",
			input: &banktypes.MsgSend{
				FromAddress: "",
				ToAddress:   baseAcc.Address,
				Amount:      origCoins,
			},
			expErr:    true,
			expErrMsg: "empty address string is not allowed",
		},
		{
			name: "invalid to address: empty address string is not allowed: invalid address",
			input: &banktypes.MsgSend{
				FromAddress: minterAccAddr,
				ToAddress:   "",
				Amount:      origCoins,
			},
			expErr:    true,
			expErrMsg: "empty address string is not allowed",
		},
		{
			name: "all good",
			input: &banktypes.MsgSend{
				FromAddress: minterAccAddr,
				ToAddress:   baseAcc.Address,
				Amount:      origCoins,
			},
			expErr: false,
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			suite.mockMintCoins(minterAcc)
			err := suite.bankKeeper.MintCoins(suite.ctx, minterAcc.Name, origCoins)
			suite.Require().NoError(err)
			if !tc.expErr {
				suite.mockSendCoins(suite.ctx, minterAcc, baseAcc.GetAddress())
			}
			_, err = suite.msgServer.Send(suite.ctx, tc.input)
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

	acc0Addr, err := suite.authKeeper.AddressCodec().BytesToString(accAddrs[0])
	suite.Require().NoError(err)
	acc1Addr, err := suite.authKeeper.AddressCodec().BytesToString(accAddrs[1])
	suite.Require().NoError(err)
	acc4Addr, err := suite.authKeeper.AddressCodec().BytesToString(accAddrs[4])
	suite.Require().NoError(err)
	minterAccAddr, err := suite.authKeeper.AddressCodec().BytesToString(minterAcc.GetAddress())
	suite.Require().NoError(err)

	testCases := []struct {
		name      string
		input     *banktypes.MsgMultiSend
		expErr    bool
		expErrMsg string
	}{
		{
			name:      "no inputs to send transaction",
			input:     &banktypes.MsgMultiSend{},
			expErr:    true,
			expErrMsg: "no inputs to send transaction",
		},
		{
			name: "no inputs to send transaction",
			input: &banktypes.MsgMultiSend{
				Outputs: []banktypes.Output{
					{Address: acc4Addr, Coins: sendCoins},
				},
			},
			expErr:    true,
			expErrMsg: "no inputs to send transaction",
		},
		{
			name: "more than one inputs to send transaction",
			input: &banktypes.MsgMultiSend{
				Inputs: []banktypes.Input{
					{Address: minterAccAddr, Coins: origCoins},
					{Address: minterAccAddr, Coins: origCoins},
				},
			},
			expErr:    true,
			expErrMsg: "multiple senders not allowed",
		},
		{
			name: "no outputs to send transaction",
			input: &banktypes.MsgMultiSend{
				Inputs: []banktypes.Input{
					{Address: minterAccAddr, Coins: origCoins},
				},
			},
			expErr:    true,
			expErrMsg: "no outputs to send transaction",
		},
		{
			name: "invalid send to blocked address",
			input: &banktypes.MsgMultiSend{
				Inputs: []banktypes.Input{
					{Address: minterAccAddr, Coins: origCoins},
				},
				Outputs: []banktypes.Output{
					{Address: acc0Addr, Coins: sendCoins},
					{Address: acc4Addr, Coins: sendCoins},
				},
			},
			expErr:    true,
			expErrMsg: "is not allowed to receive funds",
		},
		{
			name: "invalid send to blocked address",
			input: &banktypes.MsgMultiSend{
				Inputs: []banktypes.Input{
					{Address: minterAccAddr, Coins: origCoins},
				},
				Outputs: []banktypes.Output{
					{Address: acc0Addr, Coins: sendCoins},
					{Address: acc1Addr, Coins: sendCoins},
				},
			},
			expErr: false,
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			suite.mockMintCoins(minterAcc)
			err := suite.bankKeeper.MintCoins(suite.ctx, minterAcc.Name, origCoins)
			suite.Require().NoError(err)
			if !tc.expErr {
				suite.mockInputOutputCoins([]sdk.AccountI{minterAcc}, accAddrs[:2])
			}
			_, err = suite.msgServer.MultiSend(suite.ctx, tc.input)
			if tc.expErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				suite.Require().NoError(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestMsgSetSendEnabled() {
	govAccAddr, err := suite.authKeeper.AddressCodec().BytesToString(govAcc.GetAddress())
	suite.Require().NoError(err)
	testCases := []struct {
		name     string
		req      *banktypes.MsgSetSendEnabled
		isExpErr bool
		errMsg   string
	}{
		{
			name: "all good",
			req: banktypes.NewMsgSetSendEnabled(
				govAccAddr,
				[]*banktypes.SendEnabled{
					banktypes.NewSendEnabled("atom1", true),
				},
				[]string{},
			),
		},
		{
			name: "all good with two denoms",
			req: banktypes.NewMsgSetSendEnabled(
				govAccAddr,
				[]*banktypes.SendEnabled{
					banktypes.NewSendEnabled("atom1", true),
					banktypes.NewSendEnabled("atom2", true),
				},
				[]string{"defcoinc", "defcoind"},
			),
		},
		{
			name: "duplicate denoms",
			req: banktypes.NewMsgSetSendEnabled(
				govAccAddr,
				[]*banktypes.SendEnabled{
					banktypes.NewSendEnabled("atom", true),
					banktypes.NewSendEnabled("atom", true),
				},
				[]string{},
			),
			isExpErr: true,
			errMsg:   `duplicate denom entries found for "atom": invalid request`,
		},
		{
			name: "bad first denom name, (invalid send enabled denom present in list)",
			req: banktypes.NewMsgSetSendEnabled(
				govAccAddr,
				[]*banktypes.SendEnabled{
					banktypes.NewSendEnabled("not a denom", true),
					banktypes.NewSendEnabled("somecoin", true),
				},
				[]string{},
			),
			isExpErr: true,
			errMsg:   `invalid SendEnabled denom "not a denom": invalid denom: not a denom: invalid request`,
		},
		{
			name: "bad second denom name, (invalid send enabled denom present in list)",
			req: banktypes.NewMsgSetSendEnabled(
				govAccAddr,
				[]*banktypes.SendEnabled{
					banktypes.NewSendEnabled("somecoin", true),
					banktypes.NewSendEnabled("not a denom", true),
				},
				[]string{},
			),
			isExpErr: true,
			errMsg:   `invalid SendEnabled denom "not a denom": invalid denom: not a denom: invalid request`,
		},
		{
			name: "invalid UseDefaultFor denom",
			req: banktypes.NewMsgSetSendEnabled(
				govAccAddr,
				[]*banktypes.SendEnabled{
					banktypes.NewSendEnabled("atom", true),
				},
				[]string{"not a denom"},
			),
			isExpErr: true,
			errMsg:   `invalid UseDefaultFor denom "not a denom": invalid denom: not a denom: invalid request`,
		},
		{
			name: "invalid authority",
			req: banktypes.NewMsgSetSendEnabled(
				"invalid",
				[]*banktypes.SendEnabled{
					banktypes.NewSendEnabled("atom", true),
				},
				[]string{},
			),
			isExpErr: true,
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			_, err := suite.msgServer.SetSendEnabled(suite.ctx, tc.req)

			if tc.isExpErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.errMsg)
			} else {
				suite.Require().NoError(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestMsgBurn() {
	origCoins := sdk.NewInt64Coin("eth", 100)
	atom0 := sdk.NewInt64Coin("atom", 0)

	multiPermAccAddr, err := suite.authKeeper.AddressCodec().BytesToString(multiPermAcc.GetAddress())
	suite.Require().NoError(err)

	testCases := []struct {
		name      string
		input     *banktypes.MsgBurn
		expErr    bool
		expErrMsg string
	}{
		{
			name: "invalid coins",
			input: &banktypes.MsgBurn{
				FromAddress: multiPermAccAddr,
				Amount:      []*sdk.Coin{&atom0},
			},
			expErr:    true,
			expErrMsg: "invalid coins",
		},

		{
			name: "invalid from address: empty address string is not allowed: invalid address",
			input: &banktypes.MsgBurn{
				FromAddress: "",
				Amount:      []*sdk.Coin{&origCoins},
			},
			expErr:    true,
			expErrMsg: "empty address string is not allowed",
		},
		{
			name: "all good",
			input: &banktypes.MsgBurn{
				FromAddress: multiPermAccAddr,
				Amount:      []*sdk.Coin{&origCoins},
			},
			expErr: false,
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			suite.mockMintCoins(multiPermAcc)
			err := suite.bankKeeper.MintCoins(suite.ctx, multiPermAcc.Name, sdk.Coins{}.Add(origCoins))
			suite.Require().NoError(err)
			if !tc.expErr {
				suite.mockBurnCoins(multiPermAcc)
			}
			_, err = suite.msgServer.Burn(suite.ctx, tc.input)
			if tc.expErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.expErrMsg)
			} else {
				suite.Require().NoError(err)
			}
		})
	}
}
