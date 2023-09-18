package keeper_test

import (
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/testutil"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	"github.com/cosmos/cosmos-sdk/x/feegrant"
	"github.com/cosmos/cosmos-sdk/x/feegrant/keeper"
	"github.com/cosmos/cosmos-sdk/x/feegrant/module"
	feegranttestutil "github.com/cosmos/cosmos-sdk/x/feegrant/testutil"
)

type KeeperTestSuite struct {
	suite.Suite

	ctx            sdk.Context
	addrs          []sdk.AccAddress
	msgSrvr        feegrant.MsgServer
	atom           sdk.Coins
	feegrantKeeper keeper.Keeper
	accountKeeper  *feegranttestutil.MockAccountKeeper
}

func TestKeeperTestSuite(t *testing.T) {
	suite.Run(t, new(KeeperTestSuite))
}

func (suite *KeeperTestSuite) SetupTest() {
	suite.addrs = simtestutil.CreateIncrementalAccounts(4)
	key := sdk.NewKVStoreKey(feegrant.StoreKey)
	testCtx := testutil.DefaultContextWithDB(suite.T(), key, sdk.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(module.AppModuleBasic{})

	// setup gomock and initialize some globally expected executions
	ctrl := gomock.NewController(suite.T())
	suite.accountKeeper = feegranttestutil.NewMockAccountKeeper(ctrl)
	suite.accountKeeper.EXPECT().GetAccount(gomock.Any(), suite.addrs[0]).Return(authtypes.NewBaseAccountWithAddress(suite.addrs[0])).AnyTimes()
	suite.accountKeeper.EXPECT().GetAccount(gomock.Any(), suite.addrs[1]).Return(authtypes.NewBaseAccountWithAddress(suite.addrs[1])).AnyTimes()
	suite.accountKeeper.EXPECT().GetAccount(gomock.Any(), suite.addrs[2]).Return(authtypes.NewBaseAccountWithAddress(suite.addrs[2])).AnyTimes()
	suite.accountKeeper.EXPECT().GetAccount(gomock.Any(), suite.addrs[3]).Return(authtypes.NewBaseAccountWithAddress(suite.addrs[3])).AnyTimes()

	suite.feegrantKeeper = keeper.NewKeeper(encCfg.Codec, key, suite.accountKeeper)
	suite.ctx = testCtx.Ctx
	suite.msgSrvr = keeper.NewMsgServerImpl(suite.feegrantKeeper)
	suite.atom = sdk.NewCoins(sdk.NewCoin("atom", sdk.NewInt(555)))
}

func (suite *KeeperTestSuite) TestKeeperCrud() {
	// some helpers
	eth := sdk.NewCoins(sdk.NewInt64Coin("eth", 123))
	exp := suite.ctx.BlockTime().AddDate(1, 0, 0)
	exp2 := suite.ctx.BlockTime().AddDate(2, 0, 0)
	basic := &feegrant.BasicAllowance{
		SpendLimit: suite.atom,
		Expiration: &exp,
	}

	basic2 := &feegrant.BasicAllowance{
		SpendLimit: eth,
		Expiration: &exp,
	}

	basic3 := &feegrant.BasicAllowance{
		SpendLimit: eth,
		Expiration: &exp2,
	}

	// let's set up some initial state here
	err := suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[0], suite.addrs[1], basic)
	suite.Require().NoError(err)

	err = suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[0], suite.addrs[2], basic2)
	suite.Require().NoError(err)

	err = suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[1], suite.addrs[2], basic)
	suite.Require().NoError(err)

	err = suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[1], suite.addrs[3], basic)
	suite.Require().NoError(err)

	err = suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[3], suite.addrs[0], basic2)
	suite.Require().NoError(err)

	// remove some, overwrite other
	_, err = suite.msgSrvr.RevokeAllowance(suite.ctx, &feegrant.MsgRevokeAllowance{Granter: suite.addrs[0].String(), Grantee: suite.addrs[1].String()})
	suite.Require().NoError(err)
	_, err = suite.msgSrvr.RevokeAllowance(suite.ctx, &feegrant.MsgRevokeAllowance{Granter: suite.addrs[0].String(), Grantee: suite.addrs[2].String()})
	suite.Require().NoError(err)

	// revoke non-exist fee allowance
	_, err = suite.msgSrvr.RevokeAllowance(suite.ctx, &feegrant.MsgRevokeAllowance{Granter: suite.addrs[0].String(), Grantee: suite.addrs[2].String()})
	suite.Require().Error(err)

	err = suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[0], suite.addrs[2], basic)
	suite.Require().NoError(err)

	err = suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[1], suite.addrs[2], basic3)
	suite.Require().NoError(err)

	// end state:
	// addr -> addr3 (basic)
	// addr2 -> addr3 (basic2), addr4(basic)
	// addr4 -> addr (basic2)

	// then lots of queries
	cases := map[string]struct {
		grantee   sdk.AccAddress
		granter   sdk.AccAddress
		allowance feegrant.FeeAllowanceI
	}{
		"addr revoked": {
			granter: suite.addrs[0],
			grantee: suite.addrs[1],
		},
		"addr revoked and added": {
			granter:   suite.addrs[0],
			grantee:   suite.addrs[2],
			allowance: basic,
		},
		"addr never there": {
			granter: suite.addrs[0],
			grantee: suite.addrs[3],
		},
		"addr modified": {
			granter:   suite.addrs[1],
			grantee:   suite.addrs[2],
			allowance: basic3,
		},
	}

	for name, tc := range cases {
		tc := tc
		suite.Run(name, func() {
			allow, _ := suite.feegrantKeeper.GetAllowance(suite.ctx, tc.granter, tc.grantee)

			if tc.allowance == nil {
				suite.Nil(allow)
				return
			}
			suite.NotNil(allow)
			suite.Equal(tc.allowance, allow)
		})
	}
	accAddr, err := sdk.AccAddressFromBech32("cosmos1rxr4mq58w3gtnx5tsc438mwjjafv3mja7k5pnu")
	suite.Require().NoError(err)
	suite.accountKeeper.EXPECT().GetAccount(gomock.Any(), accAddr).Return(authtypes.NewBaseAccountWithAddress(accAddr)).AnyTimes()

	// let's grant and revoke authorization to non existing account
	err = suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[3], accAddr, basic2)
	suite.Require().NoError(err)

	_, err = suite.feegrantKeeper.GetAllowance(suite.ctx, suite.addrs[3], accAddr)
	suite.Require().NoError(err)

	_, err = suite.msgSrvr.RevokeAllowance(suite.ctx, &feegrant.MsgRevokeAllowance{Granter: suite.addrs[3].String(), Grantee: accAddr.String()})
	suite.Require().NoError(err)
}

func (suite *KeeperTestSuite) TestUseGrantedFee() {
	eth := sdk.NewCoins(sdk.NewInt64Coin("eth", 123))
	blockTime := suite.ctx.BlockTime()
	oneYear := blockTime.AddDate(1, 0, 0)

	future := &feegrant.BasicAllowance{
		SpendLimit: suite.atom,
		Expiration: &oneYear,
	}

	// for testing limits of the contract
	hugeAtom := sdk.NewCoins(sdk.NewInt64Coin("atom", 9999))
	smallAtom := sdk.NewCoins(sdk.NewInt64Coin("atom", 1))
	futureAfterSmall := &feegrant.BasicAllowance{
		SpendLimit: sdk.NewCoins(sdk.NewInt64Coin("atom", 554)),
		Expiration: &oneYear,
	}

	// then lots of queries
	cases := map[string]struct {
		grantee sdk.AccAddress
		granter sdk.AccAddress
		fee     sdk.Coins
		allowed bool
		final   feegrant.FeeAllowanceI
	}{
		"use entire pot": {
			granter: suite.addrs[0],
			grantee: suite.addrs[1],
			fee:     suite.atom,
			allowed: true,
			final:   nil,
		},
		"too high": {
			granter: suite.addrs[0],
			grantee: suite.addrs[1],
			fee:     hugeAtom,
			allowed: false,
			final:   future,
		},
		"use a little": {
			granter: suite.addrs[0],
			grantee: suite.addrs[1],
			fee:     smallAtom,
			allowed: true,
			final:   futureAfterSmall,
		},
	}

	for name, tc := range cases {
		tc := tc
		suite.Run(name, func() {
			err := suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[0], suite.addrs[1], future)
			suite.Require().NoError(err)

			err = suite.feegrantKeeper.UseGrantedFees(suite.ctx, tc.granter, tc.grantee, tc.fee, []sdk.Msg{})
			if tc.allowed {
				suite.NoError(err)
			} else {
				suite.Error(err)
			}

			loaded, _ := suite.feegrantKeeper.GetAllowance(suite.ctx, tc.granter, tc.grantee)
			suite.Equal(tc.final, loaded)
		})
	}

	basicAllowance := &feegrant.BasicAllowance{
		SpendLimit: eth,
		Expiration: &blockTime,
	}

	// create basic fee allowance
	err := suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[0], suite.addrs[2], basicAllowance)
	suite.Require().NoError(err)

	// waiting for future blocks, allowance to be pruned.
	ctx := suite.ctx.WithBlockTime(oneYear)

	// expect error: feegrant expired
	err = suite.feegrantKeeper.UseGrantedFees(ctx, suite.addrs[0], suite.addrs[2], eth, []sdk.Msg{})
	suite.Error(err)
	suite.Contains(err.Error(), "fee allowance expired")

	// verify: feegrant is revoked
	_, err = suite.feegrantKeeper.GetAllowance(ctx, suite.addrs[0], suite.addrs[2])
	suite.Error(err)
	suite.Contains(err.Error(), "fee-grant not found")
}

func (suite *KeeperTestSuite) TestIterateGrants() {
	eth := sdk.NewCoins(sdk.NewInt64Coin("eth", 123))
	exp := suite.ctx.BlockTime().AddDate(1, 0, 0)

	allowance := &feegrant.BasicAllowance{
		SpendLimit: suite.atom,
		Expiration: &exp,
	}

	allowance1 := &feegrant.BasicAllowance{
		SpendLimit: eth,
		Expiration: &exp,
	}

	suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[0], suite.addrs[1], allowance)
	suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[2], suite.addrs[1], allowance1)

	suite.feegrantKeeper.IterateAllFeeAllowances(suite.ctx, func(grant feegrant.Grant) bool {
		suite.Require().Equal(suite.addrs[1].String(), grant.Grantee)
		suite.Require().Contains([]string{suite.addrs[0].String(), suite.addrs[2].String()}, grant.Granter)
		return true
	})
}

func (suite *KeeperTestSuite) TestPruneGrants() {
	eth := sdk.NewCoins(sdk.NewInt64Coin("eth", 123))
	now := suite.ctx.BlockTime()
	oneYearExpiry := now.AddDate(1, 0, 0)
	oneDay := now.AddDate(0, 0, 1)

	testCases := []struct {
		name      string
		ctx       sdk.Context
		granter   sdk.AccAddress
		grantee   sdk.AccAddress
		allowance feegrant.FeeAllowanceI
		expErrMsg string
		preRun    func()
		postRun   func()
	}{
		{
			name:    "grant not pruned from state",
			ctx:     suite.ctx,
			granter: suite.addrs[0],
			grantee: suite.addrs[1],
			allowance: &feegrant.BasicAllowance{
				SpendLimit: suite.atom,
				Expiration: &now,
			},
		},
		{
			name:      "grant pruned from state after a block: error",
			ctx:       suite.ctx.WithBlockTime(now.AddDate(0, 0, 1)),
			granter:   suite.addrs[2],
			grantee:   suite.addrs[1],
			expErrMsg: "not found",
			allowance: &feegrant.BasicAllowance{
				SpendLimit: eth,
				Expiration: &now,
			},
		},
		{
			name:    "grant not pruned from state after a day: no error",
			ctx:     suite.ctx.WithBlockTime(now.AddDate(0, 0, 1)),
			granter: suite.addrs[1],
			grantee: suite.addrs[0],
			allowance: &feegrant.BasicAllowance{
				SpendLimit: eth,
				Expiration: &oneYearExpiry,
			},
		},
		{
			name:      "grant pruned from state after a year: error",
			ctx:       suite.ctx.WithBlockTime(now.AddDate(1, 0, 1)),
			granter:   suite.addrs[1],
			grantee:   suite.addrs[2],
			expErrMsg: "not found",
			allowance: &feegrant.BasicAllowance{
				SpendLimit: eth,
				Expiration: &oneYearExpiry,
			},
		},
		{
			name:    "no expiry: no error",
			ctx:     suite.ctx.WithBlockTime(now.AddDate(1, 0, 0)),
			granter: suite.addrs[1],
			grantee: suite.addrs[2],
			allowance: &feegrant.BasicAllowance{
				SpendLimit: eth,
				Expiration: &oneYearExpiry,
			},
		},
		{
			name:    "grant created with a day expiry & overwritten with no expiry shouldn't be pruned: no error",
			ctx:     suite.ctx.WithBlockTime(now.AddDate(0, 0, 2)),
			granter: suite.addrs[2],
			grantee: suite.addrs[1],
			allowance: &feegrant.BasicAllowance{
				SpendLimit: eth,
			},
			preRun: func() {
				// create a grant with a day expiry.
				allowance := &feegrant.BasicAllowance{
					SpendLimit: suite.atom,
					Expiration: &oneDay,
				}
				err := suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[2], suite.addrs[1], allowance)
				suite.NoError(err)
			},
			postRun: func() {
				_, err := suite.msgSrvr.RevokeAllowance(suite.ctx, &feegrant.MsgRevokeAllowance{
					Granter: suite.addrs[2].String(),
					Grantee: suite.addrs[1].String(),
				})
				suite.NoError(err)
			},
		},
		{
			name:    "grant created with a day expiry & overwritten with a year expiry shouldn't be pruned: no error",
			ctx:     suite.ctx.WithBlockTime(now.AddDate(0, 0, 2)),
			granter: suite.addrs[2],
			grantee: suite.addrs[1],
			allowance: &feegrant.BasicAllowance{
				SpendLimit: eth,
				Expiration: &oneYearExpiry,
			},
			preRun: func() {
				// create a grant with a day expiry.
				allowance := &feegrant.BasicAllowance{
					SpendLimit: suite.atom,
					Expiration: &oneDay,
				}
				err := suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[2], suite.addrs[1], allowance)
				suite.NoError(err)
			},
			postRun: func() {
				_, err := suite.msgSrvr.RevokeAllowance(suite.ctx, &feegrant.MsgRevokeAllowance{
					Granter: suite.addrs[2].String(),
					Grantee: suite.addrs[1].String(),
				})
				suite.NoError(err)
			},
		},
		{
			name:    "grant created with a year expiry & overwritten with a day expiry should be pruned after a day: error",
			ctx:     suite.ctx.WithBlockTime(now.AddDate(0, 0, 2)),
			granter: suite.addrs[2],
			grantee: suite.addrs[1],
			allowance: &feegrant.BasicAllowance{
				SpendLimit: eth,
				Expiration: &oneDay,
			},
			preRun: func() {
				// create a grant with a year expiry.
				allowance := &feegrant.BasicAllowance{
					SpendLimit: suite.atom,
					Expiration: &oneYearExpiry,
				}
				err := suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[2], suite.addrs[1], allowance)
				suite.NoError(err)
			},
			postRun:   func() {},
			expErrMsg: "not found",
		},
		{
			name:    "grant created with no expiry & overwritten with a day expiry should be pruned after a day: error",
			ctx:     suite.ctx.WithBlockTime(now.AddDate(0, 0, 2)),
			granter: suite.addrs[2],
			grantee: suite.addrs[1],
			allowance: &feegrant.BasicAllowance{
				SpendLimit: eth,
				Expiration: &oneDay,
			},
			preRun: func() {
				// create a grant with no expiry.
				allowance := &feegrant.BasicAllowance{
					SpendLimit: suite.atom,
				}
				err := suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[2], suite.addrs[1], allowance)
				suite.NoError(err)
			},
			postRun:   func() {},
			expErrMsg: "not found",
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			if tc.preRun != nil {
				tc.preRun()
			}
			err := suite.feegrantKeeper.GrantAllowance(suite.ctx, tc.granter, tc.grantee, tc.allowance)
			suite.NoError(err)
			suite.feegrantKeeper.RemoveExpiredAllowances(tc.ctx)
			grant, err := suite.feegrantKeeper.GetAllowance(tc.ctx, tc.granter, tc.grantee)
			if tc.expErrMsg != "" {
				suite.Error(err)
				suite.Contains(err.Error(), tc.expErrMsg)
			} else {
				suite.NotNil(grant)
			}
			if tc.postRun != nil {
				tc.postRun()
			}
		})
	}
}
