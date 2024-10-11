package keeper_test

import (
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/suite"

	"cosmossdk.io/core/header"
	coretesting "cosmossdk.io/core/testing"
	sdkmath "cosmossdk.io/math"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/feegrant"
	"cosmossdk.io/x/feegrant/keeper"
	"cosmossdk.io/x/feegrant/module"
	feegranttestutil "cosmossdk.io/x/feegrant/testutil"

	codecaddress "github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

type KeeperTestSuite struct {
	suite.Suite

	ctx            sdk.Context
	addrs          []sdk.AccAddress
	encodedAddrs   []string
	msgSrvr        feegrant.MsgServer
	atom           sdk.Coins
	feegrantKeeper keeper.Keeper
	accountKeeper  *feegranttestutil.MockAccountKeeper
}

func TestKeeperTestSuite(t *testing.T) {
	suite.Run(t, new(KeeperTestSuite))
}

func (suite *KeeperTestSuite) SetupTest() {
	suite.addrs = simtestutil.CreateIncrementalAccounts(20)
	key := storetypes.NewKVStoreKey(feegrant.StoreKey)
	testCtx := testutil.DefaultContextWithDB(suite.T(), key, storetypes.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, module.AppModule{})

	// setup gomock and initialize some globally expected executions
	ctrl := gomock.NewController(suite.T())
	suite.accountKeeper = feegranttestutil.NewMockAccountKeeper(ctrl)
	for i := 0; i < len(suite.addrs); i++ {
		suite.accountKeeper.EXPECT().GetAccount(gomock.Any(), suite.addrs[i]).Return(authtypes.NewBaseAccountWithAddress(suite.addrs[i])).AnyTimes()
	}

	ac := codecaddress.NewBech32Codec("cosmos")
	suite.accountKeeper.EXPECT().AddressCodec().Return(ac).AnyTimes()
	for _, addr := range suite.addrs {
		str, err := ac.BytesToString(addr)
		suite.Require().NoError(err)
		suite.encodedAddrs = append(suite.encodedAddrs, str)
	}

	suite.feegrantKeeper = keeper.NewKeeper(runtime.NewEnvironment(runtime.NewKVStoreService(key), coretesting.NewNopLogger()), encCfg.Codec, suite.accountKeeper)
	suite.ctx = testCtx.Ctx
	suite.msgSrvr = keeper.NewMsgServerImpl(suite.feegrantKeeper)
	suite.atom = sdk.NewCoins(sdk.NewCoin("atom", sdkmath.NewInt(555)))
}

func (suite *KeeperTestSuite) TestKeeperCrud() {
	// some helpers
	eth := sdk.NewCoins(sdk.NewInt64Coin("eth", 123))
	exp := suite.ctx.HeaderInfo().Time.AddDate(1, 0, 0)
	exp2 := suite.ctx.HeaderInfo().Time.AddDate(2, 0, 0)
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

	// addrs[0] -> addrs[1] (basic)
	err := suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[0], suite.addrs[1], basic)
	suite.Require().NoError(err)

	// addrs[0] -> addrs[2] (basic2)
	err = suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[0], suite.addrs[2], basic2)
	suite.Require().NoError(err)

	// addrs[1] -> addrs[2] (basic)
	err = suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[1], suite.addrs[2], basic)
	suite.Require().NoError(err)

	// addrs[1] -> addrs[3] (basic)
	err = suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[1], suite.addrs[3], basic)
	suite.Require().NoError(err)

	// addrs[3] -> addrs[0] (basic2)
	err = suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[3], suite.addrs[0], basic2)
	suite.Require().NoError(err)

	// addrs[3] -> addrs[0] (basic2) expect error with duplicate grant
	err = suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[3], suite.addrs[0], basic2)
	suite.Require().Error(err)

	// remove some, overwrite other
	_, err = suite.msgSrvr.RevokeAllowance(suite.ctx, &feegrant.MsgRevokeAllowance{Granter: suite.encodedAddrs[0], Grantee: suite.encodedAddrs[1]})
	suite.Require().NoError(err)

	_, err = suite.msgSrvr.RevokeAllowance(suite.ctx, &feegrant.MsgRevokeAllowance{Granter: suite.encodedAddrs[0], Grantee: suite.encodedAddrs[2]})
	suite.Require().NoError(err)

	// revoke non-exist fee allowance
	_, err = suite.msgSrvr.RevokeAllowance(suite.ctx, &feegrant.MsgRevokeAllowance{Granter: suite.encodedAddrs[0], Grantee: suite.encodedAddrs[2]})
	suite.Require().Error(err)

	err = suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[0], suite.addrs[2], basic)
	suite.Require().NoError(err)

	// revoke an existing grant and grant again with different allowance.
	_, err = suite.msgSrvr.RevokeAllowance(suite.ctx, &feegrant.MsgRevokeAllowance{Granter: suite.encodedAddrs[1], Grantee: suite.encodedAddrs[2]})
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
	address := "cosmos1rxr4mq58w3gtnx5tsc438mwjjafv3mja7k5pnu"
	accAddr, err := codecaddress.NewBech32Codec("cosmos").StringToBytes(address)
	suite.Require().NoError(err)
	suite.accountKeeper.EXPECT().GetAccount(gomock.Any(), accAddr).Return(authtypes.NewBaseAccountWithAddress(accAddr)).AnyTimes()

	// let's grant and revoke authorization to non existing account
	err = suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[3], accAddr, basic2)
	suite.Require().NoError(err)

	_, err = suite.feegrantKeeper.GetAllowance(suite.ctx, suite.addrs[3], accAddr)
	suite.Require().NoError(err)

	_, err = suite.msgSrvr.RevokeAllowance(suite.ctx, &feegrant.MsgRevokeAllowance{Granter: suite.encodedAddrs[3], Grantee: address})
	suite.Require().NoError(err)
}

func (suite *KeeperTestSuite) TestUseGrantedFee() {
	eth := sdk.NewCoins(sdk.NewInt64Coin("eth", 123))
	blockTime := suite.ctx.HeaderInfo().Time
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
		postRun func()
	}{
		"use entire pot": {
			granter: suite.addrs[0],
			grantee: suite.addrs[1],
			fee:     suite.atom,
			allowed: true,
			final:   nil,
			postRun: func() {},
		},
		"too high": {
			granter: suite.addrs[0],
			grantee: suite.addrs[1],
			fee:     hugeAtom,
			allowed: false,
			final:   future,
			postRun: func() {
				_, err := suite.msgSrvr.RevokeAllowance(suite.ctx, &feegrant.MsgRevokeAllowance{
					Granter: suite.encodedAddrs[0],
					Grantee: suite.encodedAddrs[1],
				})
				suite.Require().NoError(err)
			},
		},
		"use a little": {
			granter: suite.addrs[0],
			grantee: suite.addrs[1],
			fee:     smallAtom,
			allowed: true,
			final:   futureAfterSmall,
			postRun: func() {
				_, err := suite.msgSrvr.RevokeAllowance(suite.ctx, &feegrant.MsgRevokeAllowance{
					Granter: suite.encodedAddrs[0],
					Grantee: suite.encodedAddrs[1],
				})
				suite.Require().NoError(err)
			},
		},
	}

	for name, tc := range cases {
		suite.Run(name, func() {
			err := suite.feegrantKeeper.GrantAllowance(suite.ctx, tc.granter, tc.grantee, future)
			suite.Require().NoError(err)

			err = suite.feegrantKeeper.UseGrantedFees(suite.ctx, tc.granter, tc.grantee, tc.fee, []sdk.Msg{})
			if tc.allowed {
				suite.NoError(err)
			} else {
				suite.Error(err)
			}

			loaded, _ := suite.feegrantKeeper.GetAllowance(suite.ctx, tc.granter, tc.grantee)
			suite.Equal(tc.final, loaded)

			tc.postRun()
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
	ctx := suite.ctx.WithHeaderInfo(header.Info{Time: oneYear})

	// expect error: feegrant expired
	err = suite.feegrantKeeper.UseGrantedFees(ctx, suite.addrs[0], suite.addrs[2], eth, []sdk.Msg{})
	suite.Error(err)
	suite.Contains(err.Error(), "fee allowance expired")

	// verify: feegrant is not revoked
	// The expired feegrant is not automatically revoked when attempting to use it.
	// This is because the transaction using an expired feegrant would fail and be rolled back.
	// Expired feegrants are typically cleaned up by the ABCI EndBlocker, not by failed usage attempts.
	_, err = suite.feegrantKeeper.GetAllowance(ctx, suite.addrs[0], suite.addrs[2])
	suite.Require().NoError(err)
}

func (suite *KeeperTestSuite) TestIterateGrants() {
	eth := sdk.NewCoins(sdk.NewInt64Coin("eth", 123))
	exp := suite.ctx.HeaderInfo().Time.AddDate(1, 0, 0)

	allowance := &feegrant.BasicAllowance{
		SpendLimit: suite.atom,
		Expiration: &exp,
	}

	allowance1 := &feegrant.BasicAllowance{
		SpendLimit: eth,
		Expiration: &exp,
	}

	err := suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[0], suite.addrs[1], allowance)
	suite.Require().NoError(err)
	err = suite.feegrantKeeper.GrantAllowance(suite.ctx, suite.addrs[2], suite.addrs[1], allowance1)
	suite.Require().NoError(err)
	err = suite.feegrantKeeper.IterateAllFeeAllowances(suite.ctx, func(grant feegrant.Grant) bool {
		suite.Require().Equal(suite.encodedAddrs[1], grant.Grantee)
		suite.Require().Contains([]string{suite.encodedAddrs[0], suite.encodedAddrs[2]}, grant.Granter)
		return true
	})
	suite.Require().NoError(err)
}

func (suite *KeeperTestSuite) TestPruneGrants() {
	eth := sdk.NewCoins(sdk.NewInt64Coin("eth", 123))
	now := suite.ctx.HeaderInfo().Time
	oneDay := now.AddDate(0, 0, 1)
	oneYearExpiry := now.AddDate(1, 0, 0)

	testCases := []struct {
		name      string
		ctx       sdk.Context
		granter   sdk.AccAddress
		grantee   sdk.AccAddress
		allowance feegrant.FeeAllowanceI
		expErrMsg string
	}{
		{
			name:      "grant pruned from state after a block: error",
			ctx:       suite.ctx,
			granter:   suite.addrs[0],
			grantee:   suite.addrs[1],
			expErrMsg: "not found",
			allowance: &feegrant.BasicAllowance{
				SpendLimit: suite.atom,
				Expiration: &now,
			},
		},
		{
			name:    "grant not pruned from state before expiration: no error",
			ctx:     suite.ctx,
			granter: suite.addrs[2],
			grantee: suite.addrs[1],
			allowance: &feegrant.BasicAllowance{
				SpendLimit: eth,
				Expiration: &oneDay,
			},
		},
		{
			name:      "grant pruned from state after a day: error",
			ctx:       suite.ctx.WithHeaderInfo(header.Info{Time: now.AddDate(0, 0, 1)}),
			granter:   suite.addrs[1],
			grantee:   suite.addrs[0],
			expErrMsg: "not found",
			allowance: &feegrant.BasicAllowance{
				SpendLimit: eth,
				Expiration: &oneDay,
			},
		},
		{
			name:    "grant not pruned from state after a day: no error",
			ctx:     suite.ctx.WithHeaderInfo(header.Info{Time: now.AddDate(0, 0, 1)}),
			granter: suite.addrs[1],
			grantee: suite.addrs[0],
			allowance: &feegrant.BasicAllowance{
				SpendLimit: eth,
				Expiration: &oneYearExpiry,
			},
		},
		{
			name:      "grant pruned from state after a year: error",
			ctx:       suite.ctx.WithHeaderInfo(header.Info{Time: now.AddDate(1, 0, 0)}),
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
			ctx:     suite.ctx.WithHeaderInfo(header.Info{Time: now.AddDate(1, 0, 0)}),
			granter: suite.addrs[1],
			grantee: suite.addrs[2],
			allowance: &feegrant.BasicAllowance{
				SpendLimit: eth,
			},
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			err := suite.feegrantKeeper.GrantAllowance(suite.ctx, tc.granter, tc.grantee, tc.allowance)
			suite.NoError(err)
			err = suite.feegrantKeeper.RemoveExpiredAllowances(tc.ctx, 5)
			suite.NoError(err)

			grant, err := suite.feegrantKeeper.GetAllowance(tc.ctx, tc.granter, tc.grantee)
			if tc.expErrMsg != "" {
				suite.Error(err)
				suite.Contains(err.Error(), tc.expErrMsg)
			} else {
				suite.NoError(err)
				suite.NotNil(grant)
			}
		})
	}
}
