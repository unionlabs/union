package keeper_test

import (
	"testing"
	"time"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/suite"

	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/header"
	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/math"
	storetypes "cosmossdk.io/store/types"
	poolkeeper "cosmossdk.io/x/protocolpool/keeper"
	pooltestutil "cosmossdk.io/x/protocolpool/testutil"
	"cosmossdk.io/x/protocolpool/types"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

var (
	poolAcc      = authtypes.NewEmptyModuleAccount(types.ModuleName)
	streamAcc    = authtypes.NewEmptyModuleAccount(types.StreamAccount)
	poolDistrAcc = authtypes.NewEmptyModuleAccount(types.ProtocolPoolDistrAccount)
)

type KeeperTestSuite struct {
	suite.Suite

	ctx           sdk.Context
	environment   appmodule.Environment
	poolKeeper    poolkeeper.Keeper
	authKeeper    *pooltestutil.MockAccountKeeper
	bankKeeper    *pooltestutil.MockBankKeeper
	stakingKeeper *pooltestutil.MockStakingKeeper

	msgServer   types.MsgServer
	queryServer types.QueryServer
}

func (s *KeeperTestSuite) SetupTest() {
	key := storetypes.NewKVStoreKey(types.StoreKey)
	storeService := runtime.NewKVStoreService(key)
	environment := runtime.NewEnvironment(storeService, coretesting.NewNopLogger())
	testCtx := testutil.DefaultContextWithDB(s.T(), key, storetypes.NewTransientStoreKey("transient_test"))
	ctx := testCtx.Ctx.WithHeaderInfo(header.Info{Time: time.Now()})
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{})

	// gomock initializations
	ctrl := gomock.NewController(s.T())
	accountKeeper := pooltestutil.NewMockAccountKeeper(ctrl)
	accountKeeper.EXPECT().GetModuleAddress(types.ModuleName).Return(poolAcc.GetAddress())
	accountKeeper.EXPECT().GetModuleAddress(types.ProtocolPoolDistrAccount).Return(poolDistrAcc.GetAddress())
	accountKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec("cosmos")).AnyTimes()
	accountKeeper.EXPECT().GetModuleAddress(types.StreamAccount).Return(streamAcc.GetAddress())
	s.authKeeper = accountKeeper

	bankKeeper := pooltestutil.NewMockBankKeeper(ctrl)
	s.bankKeeper = bankKeeper

	stakingKeeper := pooltestutil.NewMockStakingKeeper(ctrl)
	stakingKeeper.EXPECT().BondDenom(ctx).Return("stake", nil).AnyTimes()
	s.stakingKeeper = stakingKeeper

	authority, err := accountKeeper.AddressCodec().BytesToString(authtypes.NewModuleAddress(types.GovModuleName))
	s.Require().NoError(err)

	poolKeeper := poolkeeper.NewKeeper(
		encCfg.Codec,
		environment,
		accountKeeper,
		bankKeeper,
		stakingKeeper,
		authority,
	)
	s.ctx = ctx
	s.poolKeeper = poolKeeper
	s.environment = environment

	types.RegisterInterfaces(encCfg.InterfaceRegistry)
	queryHelper := baseapp.NewQueryServerTestHelper(ctx, encCfg.InterfaceRegistry)
	types.RegisterQueryServer(queryHelper, poolkeeper.Querier{Keeper: poolKeeper})
	s.msgServer = poolkeeper.NewMsgServerImpl(poolKeeper)
	s.queryServer = poolkeeper.NewQuerier(poolKeeper)
}

func (s *KeeperTestSuite) mockSendCoinsFromModuleToAccount(accAddr sdk.AccAddress) {
	s.bankKeeper.EXPECT().SendCoinsFromModuleToAccount(s.ctx, types.ModuleName, accAddr, gomock.Any()).AnyTimes()
}

func (s *KeeperTestSuite) mockWithdrawContinuousFund() {
	s.authKeeper.EXPECT().GetModuleAccount(gomock.Any(), types.ModuleName).Return(poolAcc).AnyTimes()
	distrBal := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(100000)))
	s.bankKeeper.EXPECT().GetAllBalances(gomock.Any(), gomock.Any()).Return(distrBal).AnyTimes()
	s.bankKeeper.EXPECT().SendCoinsFromModuleToAccount(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).AnyTimes()
	s.stakingKeeper.EXPECT().BondDenom(gomock.Any()).Return("stake", nil).AnyTimes()
}

func (s *KeeperTestSuite) mockStreamFunds(distributed math.Int) {
	s.authKeeper.EXPECT().GetModuleAccount(s.ctx, types.ModuleName).Return(poolAcc).AnyTimes()
	s.authKeeper.EXPECT().GetModuleAccount(s.ctx, types.ProtocolPoolDistrAccount).Return(poolDistrAcc).AnyTimes()
	s.authKeeper.EXPECT().GetModuleAddress(types.StreamAccount).Return(streamAcc.GetAddress()).AnyTimes()
	distrBal := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, distributed))
	s.bankKeeper.EXPECT().GetAllBalances(s.ctx, poolDistrAcc.GetAddress()).Return(distrBal).AnyTimes()
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(s.ctx, poolDistrAcc.GetName(), streamAcc.GetName(), gomock.Any()).AnyTimes()
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(s.ctx, poolDistrAcc.GetName(), poolAcc.GetName(), gomock.Any()).AnyTimes()
}

func TestKeeperTestSuite(t *testing.T) {
	suite.Run(t, new(KeeperTestSuite))
}

func (s *KeeperTestSuite) TestIterateAndUpdateFundsDistribution() {
	// We'll create 2 continuous funds of 30% each, and the total pool is 1000000, meaning each fund should get 300000

	s.SetupTest()
	s.authKeeper.EXPECT().GetModuleAccount(s.ctx, types.ProtocolPoolDistrAccount).Return(poolAcc).AnyTimes()
	distrBal := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(1000000)))
	s.bankKeeper.EXPECT().GetAllBalances(s.ctx, poolAcc.GetAddress()).Return(distrBal).AnyTimes()
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(s.ctx, poolDistrAcc.GetName(), streamAcc.GetName(), sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(600000))))
	s.bankKeeper.EXPECT().SendCoinsFromModuleToModule(s.ctx, poolDistrAcc.GetName(), poolAcc.GetName(), sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(400000))))

	_, err := s.msgServer.CreateContinuousFund(s.ctx, &types.MsgCreateContinuousFund{
		Authority:  s.poolKeeper.GetAuthority(),
		Recipient:  "cosmos1qypq2q2l8z4wz2z2l8z4wz2z2l8z4wz2srklj6",
		Percentage: math.LegacyMustNewDecFromStr("0.3"),
	})
	s.Require().NoError(err)

	_, err = s.msgServer.CreateContinuousFund(s.ctx, &types.MsgCreateContinuousFund{
		Authority:  s.poolKeeper.GetAuthority(),
		Recipient:  "cosmos1tygms3xhhs3yv487phx3dw4a95jn7t7lpm470r",
		Percentage: math.LegacyMustNewDecFromStr("0.3"),
	})
	s.Require().NoError(err)

	_ = s.poolKeeper.SetToDistribute(s.ctx)

	err = s.poolKeeper.IterateAndUpdateFundsDistribution(s.ctx)
	s.Require().NoError(err)

	err = s.poolKeeper.RecipientFundDistribution.Walk(s.ctx, nil, func(key sdk.AccAddress, value math.Int) (stop bool, err error) {
		strAddr, err := s.authKeeper.AddressCodec().BytesToString(key)
		s.Require().NoError(err)

		if strAddr == "cosmos1qypq2q2l8z4wz2z2l8z4wz2z2l8z4wz2srklj6" {
			s.Require().Equal(value, math.NewInt(300000))
		} else if strAddr == "cosmos1tygms3xhhs3yv487phx3dw4a95jn7t7lpm470r" {
			s.Require().Equal(value, math.NewInt(300000))
		}
		return false, nil
	})
	s.Require().NoError(err)
}

func (suite *KeeperTestSuite) TestGetCommunityPool() {
	suite.SetupTest()

	expectedBalance := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(1000000)))
	suite.authKeeper.EXPECT().GetModuleAccount(suite.ctx, types.ModuleName).Return(poolAcc).Times(1)
	suite.bankKeeper.EXPECT().GetAllBalances(suite.ctx, poolAcc.GetAddress()).Return(expectedBalance).Times(1)

	balance, err := suite.poolKeeper.GetCommunityPool(suite.ctx)
	suite.Require().NoError(err)
	suite.Require().Equal(expectedBalance, balance)

	// Test error case when module account doesn't exist
	suite.authKeeper.EXPECT().GetModuleAccount(suite.ctx, types.ModuleName).Return(nil).Times(1)
	_, err = suite.poolKeeper.GetCommunityPool(suite.ctx)
	suite.Require().Error(err)
	suite.Require().Contains(err.Error(), "module account protocolpool does not exist")
}

func (suite *KeeperTestSuite) TestSetToDistribute() {
	suite.SetupTest()

	suite.authKeeper.EXPECT().GetModuleAccount(suite.ctx, types.ProtocolPoolDistrAccount).Return(poolDistrAcc).AnyTimes()
	distrBal := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(1000000)))
	suite.bankKeeper.EXPECT().GetAllBalances(suite.ctx, poolDistrAcc.GetAddress()).Return(distrBal).AnyTimes()

	// because there are no continuous funds, all are going to the community pool
	suite.bankKeeper.EXPECT().SendCoinsFromModuleToModule(suite.ctx, poolDistrAcc.GetName(), poolAcc.GetName(), distrBal)

	err := suite.poolKeeper.SetToDistribute(suite.ctx)
	suite.Require().NoError(err)

	// Verify that LastBalance was not set (zero balance)
	_, err = suite.poolKeeper.LastBalance.Get(suite.ctx)
	suite.Require().ErrorContains(err, "not found")

	// create new continuous fund and distribute again
	addrCdc := address.NewBech32Codec("cosmos")
	addrStr := "cosmos1qypq2q2l8z4wz2z2l8z4wz2z2l8z4wz2srklj6"
	addrBz, err := addrCdc.StringToBytes(addrStr)
	suite.Require().NoError(err)

	err = suite.poolKeeper.ContinuousFund.Set(suite.ctx, addrBz, types.ContinuousFund{
		Recipient:  addrStr,
		Percentage: math.LegacyMustNewDecFromStr("0.3"),
		Expiry:     nil,
	})
	suite.Require().NoError(err)

	err = suite.poolKeeper.SetToDistribute(suite.ctx)
	suite.Require().NoError(err)

	// Verify that LastBalance was set correctly
	lastBalance, err := suite.poolKeeper.LastBalance.Get(suite.ctx)
	suite.Require().NoError(err)
	suite.Require().Equal(math.NewInt(1000000), lastBalance)

	// Verify that a distribution was set
	var distribution math.Int
	err = suite.poolKeeper.Distributions.Walk(suite.ctx, nil, func(key time.Time, value math.Int) (bool, error) {
		distribution = value
		return true, nil
	})
	suite.Require().NoError(err)
	suite.Require().Equal(math.NewInt(1000000), distribution)

	// Test case when balance is zero
	zeroBal := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, math.ZeroInt()))
	suite.bankKeeper.EXPECT().GetAllBalances(suite.ctx, poolDistrAcc.GetAddress()).Return(zeroBal).AnyTimes()

	err = suite.poolKeeper.SetToDistribute(suite.ctx)
	suite.Require().NoError(err)

	// Verify that no new distribution was set
	count := 0
	err = suite.poolKeeper.Distributions.Walk(suite.ctx, nil, func(key time.Time, value math.Int) (bool, error) {
		count++
		return false, nil
	})
	suite.Require().NoError(err)
	suite.Require().Equal(1, count) // Only the previous distribution should exist
}
