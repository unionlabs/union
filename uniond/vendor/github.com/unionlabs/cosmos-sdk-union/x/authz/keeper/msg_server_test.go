package keeper_test

import (
	"time"

	"github.com/golang/mock/gomock"

	"cosmossdk.io/core/header"
	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/x/authz"
	banktypes "cosmossdk.io/x/bank/types"

	"github.com/cosmos/cosmos-sdk/codec/address"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

func (suite *TestSuite) createAccounts() []sdk.AccAddress {
	addrs := simtestutil.CreateIncrementalAccounts(2)
	suite.accountKeeper.EXPECT().GetAccount(gomock.Any(), suite.addrs[0]).Return(authtypes.NewBaseAccountWithAddress(suite.addrs[0])).AnyTimes()
	suite.accountKeeper.EXPECT().GetAccount(gomock.Any(), suite.addrs[1]).Return(authtypes.NewBaseAccountWithAddress(suite.addrs[1])).AnyTimes()
	return addrs
}

func (suite *TestSuite) TestGrant() {
	ctx := suite.ctx.WithHeaderInfo(header.Info{Time: time.Now()})
	addrs := suite.createAccounts()
	curBlockTime := ctx.HeaderInfo().Time

	suite.accountKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec("cosmos")).AnyTimes()

	oneHour := curBlockTime.Add(time.Hour)
	oneYear := curBlockTime.AddDate(1, 0, 0)

	coins := sdk.NewCoins(sdk.NewCoin("steak", sdkmath.NewInt(10)))

	grantee, granter := addrs[0], addrs[1]
	granterStrAddr, err := suite.accountKeeper.AddressCodec().BytesToString(granter)
	suite.Require().NoError(err)
	granteeStrAddr, err := suite.accountKeeper.AddressCodec().BytesToString(grantee)
	suite.Require().NoError(err)

	testCases := []struct {
		name     string
		malleate func() *authz.MsgGrant
		expErr   bool
		errMsg   string
	}{
		{
			name: "identical grantee and granter",
			malleate: func() *authz.MsgGrant {
				grant, err := authz.NewGrant(curBlockTime, banktypes.NewSendAuthorization(coins, nil, suite.accountKeeper.AddressCodec()), &oneYear)
				suite.Require().NoError(err)
				return &authz.MsgGrant{
					Granter: granteeStrAddr,
					Grantee: granteeStrAddr,
					Grant:   grant,
				}
			},
			expErr: true,
			errMsg: "grantee and granter should be different",
		},
		{
			name: "invalid granter",
			malleate: func() *authz.MsgGrant {
				grant, err := authz.NewGrant(curBlockTime, banktypes.NewSendAuthorization(coins, nil, suite.accountKeeper.AddressCodec()), &oneYear)
				suite.Require().NoError(err)
				return &authz.MsgGrant{
					Granter: "invalid",
					Grantee: granteeStrAddr,
					Grant:   grant,
				}
			},
			expErr: true,
			errMsg: "invalid bech32 string",
		},
		{
			name: "invalid grantee",
			malleate: func() *authz.MsgGrant {
				grant, err := authz.NewGrant(curBlockTime, banktypes.NewSendAuthorization(coins, nil, suite.accountKeeper.AddressCodec()), &oneYear)
				suite.Require().NoError(err)
				return &authz.MsgGrant{
					Granter: granterStrAddr,
					Grantee: "invalid",
					Grant:   grant,
				}
			},
			expErr: true,
			errMsg: "invalid bech32 string",
		},
		{
			name: "invalid grant",
			malleate: func() *authz.MsgGrant {
				return &authz.MsgGrant{
					Granter: granterStrAddr,
					Grantee: granteeStrAddr,
					Grant: authz.Grant{
						Expiration: &oneYear,
					},
				}
			},
			expErr: true,
			errMsg: "authorization is nil: invalid type",
		},
		{
			name: "invalid grant, past time",
			malleate: func() *authz.MsgGrant {
				pTime := curBlockTime.Add(-time.Hour)
				grant, err := authz.NewGrant(curBlockTime, banktypes.NewSendAuthorization(coins, nil, suite.accountKeeper.AddressCodec()), &oneHour) // we only need the authorization
				suite.Require().NoError(err)
				return &authz.MsgGrant{
					Granter: granterStrAddr,
					Grantee: granteeStrAddr,
					Grant: authz.Grant{
						Authorization: grant.Authorization,
						Expiration:    &pTime,
					},
				}
			},
			expErr: true,
			errMsg: "expiration must be after the current block time",
		},
		{
			name: "grantee account does not exist on chain: valid grant",
			malleate: func() *authz.MsgGrant {
				newAcc := sdk.AccAddress("valid")
				suite.accountKeeper.EXPECT().GetAccount(gomock.Any(), newAcc).Return(nil).AnyTimes()
				acc := authtypes.NewBaseAccountWithAddress(newAcc)
				suite.accountKeeper.EXPECT().NewAccountWithAddress(gomock.Any(), newAcc).Return(acc).AnyTimes()

				grant, err := authz.NewGrant(curBlockTime, banktypes.NewSendAuthorization(coins, nil, suite.accountKeeper.AddressCodec()), &oneYear)
				suite.Require().NoError(err)

				addr, err := suite.accountKeeper.AddressCodec().BytesToString(newAcc)
				suite.Require().NoError(err)

				return &authz.MsgGrant{
					Granter: granterStrAddr,
					Grantee: addr,
					Grant:   grant,
				}
			},
		},
		{
			name: "valid grant",
			malleate: func() *authz.MsgGrant {
				grant, err := authz.NewGrant(curBlockTime, banktypes.NewSendAuthorization(coins, nil, suite.accountKeeper.AddressCodec()), &oneYear)
				suite.Require().NoError(err)
				return &authz.MsgGrant{
					Granter: granterStrAddr,
					Grantee: granteeStrAddr,
					Grant:   grant,
				}
			},
		},
		{
			name: "valid grant, same grantee, granter pair but different msgType",
			malleate: func() *authz.MsgGrant {
				g, err := authz.NewGrant(curBlockTime, banktypes.NewSendAuthorization(coins, nil, suite.accountKeeper.AddressCodec()), &oneHour)
				suite.Require().NoError(err)
				_, err = suite.msgSrvr.Grant(suite.ctx, &authz.MsgGrant{
					Granter: granterStrAddr,
					Grantee: granteeStrAddr,
					Grant:   g,
				})
				suite.Require().NoError(err)

				grant, err := authz.NewGrant(curBlockTime, authz.NewGenericAuthorization("/cosmos.bank.v1beta1.MsgUpdateParams"), &oneHour)
				suite.Require().NoError(err)
				return &authz.MsgGrant{
					Granter: granterStrAddr,
					Grantee: granteeStrAddr,
					Grant:   grant,
				}
			},
		},
		{
			name: "valid grant with allow list",
			malleate: func() *authz.MsgGrant {
				grant, err := authz.NewGrant(curBlockTime, banktypes.NewSendAuthorization(coins, []sdk.AccAddress{granter}, suite.accountKeeper.AddressCodec()), &oneYear)
				suite.Require().NoError(err)
				return &authz.MsgGrant{
					Granter: granterStrAddr,
					Grantee: granteeStrAddr,
					Grant:   grant,
				}
			},
		},
		{
			name: "valid grant with nil expiration time",
			malleate: func() *authz.MsgGrant {
				grant, err := authz.NewGrant(curBlockTime, banktypes.NewSendAuthorization(coins, nil, suite.accountKeeper.AddressCodec()), nil)
				suite.Require().NoError(err)
				return &authz.MsgGrant{
					Granter: granterStrAddr,
					Grantee: granteeStrAddr,
					Grant:   grant,
				}
			},
		},
		{
			name: "invalid grant with msg grant",
			malleate: func() *authz.MsgGrant {
				grant, err := authz.NewGrant(curBlockTime, authz.NewGenericAuthorization("/cosmos.authz.v1beta1.MsgGrant"), nil)
				suite.Require().NoError(err)
				return &authz.MsgGrant{
					Granter: granterStrAddr,
					Grantee: granteeStrAddr,
					Grant:   grant,
				}
			},
			expErr: true,
			errMsg: "authz msgGrant is not allowed",
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			_, err := suite.msgSrvr.Grant(suite.ctx, tc.malleate())
			if tc.expErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.errMsg)
			} else {
				suite.Require().NoError(err)
			}
		})
	}
}

func (suite *TestSuite) TestRevoke() {
	addrs := suite.createAccounts()

	grantee, granter := addrs[0], addrs[1]
	granterStrAddr, err := suite.accountKeeper.AddressCodec().BytesToString(granter)
	suite.Require().NoError(err)
	granteeStrAddr, err := suite.accountKeeper.AddressCodec().BytesToString(grantee)
	suite.Require().NoError(err)

	testCases := []struct {
		name     string
		malleate func() *authz.MsgRevoke
		expErr   bool
		errMsg   string
	}{
		{
			name: "identical grantee and granter",
			malleate: func() *authz.MsgRevoke {
				return &authz.MsgRevoke{
					Granter:    granteeStrAddr,
					Grantee:    granteeStrAddr,
					MsgTypeUrl: bankSendAuthMsgType,
				}
			},
			expErr: true,
			errMsg: "grantee and granter should be different",
		},
		{
			name: "invalid granter",
			malleate: func() *authz.MsgRevoke {
				return &authz.MsgRevoke{
					Granter:    "invalid",
					Grantee:    granteeStrAddr,
					MsgTypeUrl: bankSendAuthMsgType,
				}
			},
			expErr: true,
			errMsg: "invalid bech32 string",
		},
		{
			name: "invalid grantee",
			malleate: func() *authz.MsgRevoke {
				return &authz.MsgRevoke{
					Granter:    granterStrAddr,
					Grantee:    "invalid",
					MsgTypeUrl: bankSendAuthMsgType,
				}
			},
			expErr: true,
			errMsg: "invalid bech32 string",
		},
		{
			name: "no msg given",
			malleate: func() *authz.MsgRevoke {
				return &authz.MsgRevoke{
					Granter:    granterStrAddr,
					Grantee:    granteeStrAddr,
					MsgTypeUrl: "",
				}
			},
			expErr: true,
			errMsg: "missing msg method name",
		},
		{
			name: "valid grant",
			malleate: func() *authz.MsgRevoke {
				suite.createSendAuthorization(grantee, granter)

				return &authz.MsgRevoke{
					Granter:    granterStrAddr,
					Grantee:    granteeStrAddr,
					MsgTypeUrl: bankSendAuthMsgType,
				}
			},
		},
		{
			name: "no existing grant to revoke",
			malleate: func() *authz.MsgRevoke {
				return &authz.MsgRevoke{
					Granter:    granterStrAddr,
					Grantee:    granteeStrAddr,
					MsgTypeUrl: bankSendAuthMsgType,
				}
			},
			expErr: true,
			errMsg: "authorization not found",
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			_, err := suite.msgSrvr.Revoke(suite.ctx, tc.malleate())
			if tc.expErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.errMsg)
			} else {
				suite.Require().NoError(err)
			}
		})
	}
}

func (suite *TestSuite) TestExec() {
	addrs := suite.createAccounts()

	grantee, granter := addrs[0], addrs[1]
	granterStrAddr, err := suite.accountKeeper.AddressCodec().BytesToString(granter)
	suite.Require().NoError(err)
	granteeStrAddr, err := suite.accountKeeper.AddressCodec().BytesToString(grantee)
	suite.Require().NoError(err)
	coins := sdk.NewCoins(sdk.NewCoin("steak", sdkmath.NewInt(10)))

	msg := &banktypes.MsgSend{
		FromAddress: granterStrAddr,
		ToAddress:   granteeStrAddr,
		Amount:      coins,
	}

	testCases := []struct {
		name     string
		malleate func() authz.MsgExec
		expErr   bool
		errMsg   string
	}{
		{
			name: "invalid grantee (empty)",
			malleate: func() authz.MsgExec {
				return authz.NewMsgExec("", []sdk.Msg{msg})
			},
			expErr: true,
			errMsg: "empty address string is not allowed",
		},
		{
			name: "non existing grant",
			malleate: func() authz.MsgExec {
				return authz.NewMsgExec(granteeStrAddr, []sdk.Msg{msg})
			},
			expErr: true,
			errMsg: "authorization not found",
		},
		{
			name: "no message case",
			malleate: func() authz.MsgExec {
				return authz.NewMsgExec(granteeStrAddr, []sdk.Msg{})
			},
			expErr: true,
			errMsg: "messages cannot be empty",
		},
		{
			name: "valid case",
			malleate: func() authz.MsgExec {
				suite.createSendAuthorization(grantee, granter)
				return authz.NewMsgExec(granteeStrAddr, []sdk.Msg{msg})
			},
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			req := tc.malleate()
			_, err := suite.msgSrvr.Exec(suite.ctx, &req)
			if tc.expErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.errMsg)
			} else {
				suite.Require().NoError(err)
			}
		})
	}
}

func (suite *TestSuite) TestPruneExpiredGrants() {
	addrs := suite.createAccounts()

	addr0, err := suite.accountKeeper.AddressCodec().BytesToString(addrs[0])
	suite.Require().NoError(err)
	addr1, err := suite.accountKeeper.AddressCodec().BytesToString(addrs[1])
	suite.Require().NoError(err)

	timeNow := suite.ctx.BlockTime()
	expiration := timeNow.Add(time.Hour)
	coins := sdk.NewCoins(sdk.NewCoin("steak", sdkmath.NewInt(10)))
	grant, err := authz.NewGrant(timeNow, banktypes.NewSendAuthorization(coins, nil, suite.accountKeeper.AddressCodec()), &expiration)
	suite.Require().NoError(err)

	_, err = suite.msgSrvr.Grant(suite.ctx, &authz.MsgGrant{
		Granter: addr0,
		Grantee: addr1,
		Grant:   grant,
	})
	suite.Require().NoError(err)

	_, err = suite.msgSrvr.Grant(suite.ctx, &authz.MsgGrant{
		Granter: addr1,
		Grantee: addr0,
		Grant:   grant,
	})
	suite.Require().NoError(err)

	totalGrants := 0
	_ = suite.authzKeeper.IterateGrants(suite.ctx, func(sdk.AccAddress, sdk.AccAddress, authz.Grant) (bool, error) {
		totalGrants++
		return false, nil
	})
	suite.Require().Equal(len(addrs), totalGrants)

	// prune expired grants
	headerInfo := suite.ctx.HeaderInfo()
	headerInfo.Time = headerInfo.Time.Add(2 * time.Hour)
	suite.ctx = suite.ctx.WithHeaderInfo(headerInfo)

	_, err = suite.authzKeeper.PruneExpiredGrants(suite.ctx, &authz.MsgPruneExpiredGrants{Pruner: addr0})
	suite.Require().NoError(err)

	totalGrants = 0
	_ = suite.authzKeeper.IterateGrants(suite.ctx, func(sdk.AccAddress, sdk.AccAddress, authz.Grant) (bool, error) {
		totalGrants++
		return false, nil
	})
	suite.Require().Equal(0, totalGrants)
}

func (suite *TestSuite) TestRevokeAllGrants() {
	addrs := simtestutil.CreateIncrementalAccounts(3)

	grantee, grantee2, granter := addrs[0], addrs[1], addrs[2]
	granterStrAddr, err := suite.accountKeeper.AddressCodec().BytesToString(granter)
	suite.Require().NoError(err)

	testCases := []struct {
		name     string
		malleate func() *authz.MsgRevokeAll
		expErr   bool
		errMsg   string
	}{
		{
			name: "invalid granter",
			malleate: func() *authz.MsgRevokeAll {
				return &authz.MsgRevokeAll{
					Granter: "invalid",
				}
			},
			expErr: true,
			errMsg: "invalid bech32 string",
		},
		{
			name: "no existing grant to revoke",
			malleate: func() *authz.MsgRevokeAll {
				return &authz.MsgRevokeAll{
					Granter: granterStrAddr,
				}
			},
			expErr: true,
			errMsg: "authorization not found",
		},
		{
			name: "valid grant",
			malleate: func() *authz.MsgRevokeAll {
				suite.createSendAuthorization(grantee, granter)
				suite.createSendAuthorization(grantee2, granter)
				return &authz.MsgRevokeAll{
					Granter: granterStrAddr,
				}
			},
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			_, err := suite.msgSrvr.RevokeAll(suite.ctx, tc.malleate())
			if tc.expErr {
				suite.Require().Error(err)
				suite.Require().Contains(err.Error(), tc.errMsg)
			} else {
				suite.Require().NoError(err)
				totalGrants := 0
				_ = suite.authzKeeper.IterateGranterGrants(suite.ctx, granter, func(sdk.AccAddress, string) (bool, error) {
					totalGrants++
					return false, nil
				})
				suite.Require().Equal(0, totalGrants)
			}
		})
	}
}
