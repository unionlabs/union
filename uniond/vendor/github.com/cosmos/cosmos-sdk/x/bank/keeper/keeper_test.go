package keeper_test

import (
	"fmt"
	"strings"
	"testing"
	"time"

	"cosmossdk.io/math"
	abci "github.com/cometbft/cometbft/abci/types"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	tmtime "github.com/cometbft/cometbft/types/time"
	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/types/query"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	vesting "github.com/cosmos/cosmos-sdk/x/auth/vesting/types"
	"github.com/cosmos/cosmos-sdk/x/bank/exported"
	"github.com/cosmos/cosmos-sdk/x/bank/keeper"
	banktestutil "github.com/cosmos/cosmos-sdk/x/bank/testutil"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types"
	minttypes "github.com/cosmos/cosmos-sdk/x/mint/types"
)

const (
	fooDenom     = "foo"
	barDenom     = "bar"
	initialPower = int64(100)
	holder       = "holder"
	multiPerm    = "multiple permissions account"
	randomPerm   = "random permission"
)

var (
	holderAcc    = authtypes.NewEmptyModuleAccount(holder)
	burnerAcc    = authtypes.NewEmptyModuleAccount(authtypes.Burner, authtypes.Burner)
	minterAcc    = authtypes.NewEmptyModuleAccount(authtypes.Minter, authtypes.Minter)
	mintAcc      = authtypes.NewEmptyModuleAccount(minttypes.ModuleName, authtypes.Minter)
	multiPermAcc = authtypes.NewEmptyModuleAccount(multiPerm, authtypes.Burner, authtypes.Minter, authtypes.Staking)

	baseAcc = authtypes.NewBaseAccountWithAddress(sdk.AccAddress([]byte("baseAcc")))

	accAddrs = []sdk.AccAddress{
		sdk.AccAddress([]byte("addr1_______________")),
		sdk.AccAddress([]byte("addr2_______________")),
		sdk.AccAddress([]byte("addr3_______________")),
		sdk.AccAddress([]byte("addr4_______________")),
		sdk.AccAddress([]byte("addr5_______________")),
	}

	// The default power validators are initialized to have within tests
	initTokens = sdk.TokensFromConsensusPower(initialPower, sdk.DefaultPowerReduction)
	initCoins  = sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, initTokens))
)

func newFooCoin(amt int64) sdk.Coin {
	return sdk.NewInt64Coin(fooDenom, amt)
}

func newBarCoin(amt int64) sdk.Coin {
	return sdk.NewInt64Coin(barDenom, amt)
}

type KeeperTestSuite struct {
	suite.Suite

	ctx        sdk.Context
	bankKeeper keeper.BaseKeeper
	authKeeper *banktestutil.MockAccountKeeper

	queryClient banktypes.QueryClient
	msgServer   banktypes.MsgServer

	encCfg moduletestutil.TestEncodingConfig
}

func TestKeeperTestSuite(t *testing.T) {
	suite.Run(t, new(KeeperTestSuite))
}

func (suite *KeeperTestSuite) SetupTest() {
	key := sdk.NewKVStoreKey(banktypes.StoreKey)
	testCtx := testutil.DefaultContextWithDB(suite.T(), key, sdk.NewTransientStoreKey("transient_test"))
	ctx := testCtx.Ctx.WithBlockHeader(tmproto.Header{Time: tmtime.Now()})
	encCfg := moduletestutil.MakeTestEncodingConfig()

	// gomock initializations
	ctrl := gomock.NewController(suite.T())
	authKeeper := banktestutil.NewMockAccountKeeper(ctrl)

	suite.ctx = ctx
	suite.authKeeper = authKeeper
	suite.bankKeeper = keeper.NewBaseKeeper(
		encCfg.Codec,
		key,
		suite.authKeeper,
		map[string]bool{accAddrs[4].String(): true},
		authtypes.NewModuleAddress(govtypes.ModuleName).String(),
	)

	banktypes.RegisterInterfaces(encCfg.InterfaceRegistry)

	queryHelper := baseapp.NewQueryServerTestHelper(ctx, encCfg.InterfaceRegistry)
	banktypes.RegisterQueryServer(queryHelper, suite.bankKeeper)
	queryClient := banktypes.NewQueryClient(queryHelper)

	suite.queryClient = queryClient
	suite.msgServer = keeper.NewMsgServerImpl(suite.bankKeeper)
	suite.encCfg = encCfg
}

func (suite *KeeperTestSuite) mockQueryClient(ctx sdk.Context) banktypes.QueryClient {
	queryHelper := baseapp.NewQueryServerTestHelper(ctx, suite.encCfg.InterfaceRegistry)
	banktypes.RegisterQueryServer(queryHelper, suite.bankKeeper)
	return banktypes.NewQueryClient(queryHelper)
}

func (suite *KeeperTestSuite) mockMintCoins(moduleAcc *authtypes.ModuleAccount) {
	suite.authKeeper.EXPECT().GetModuleAccount(suite.ctx, moduleAcc.Name).Return(moduleAcc)
}

func (suite *KeeperTestSuite) mockSendCoinsFromModuleToAccount(moduleAcc *authtypes.ModuleAccount, accAddr sdk.AccAddress) {
	suite.authKeeper.EXPECT().GetModuleAddress(moduleAcc.Name).Return(moduleAcc.GetAddress())
	suite.authKeeper.EXPECT().GetAccount(suite.ctx, moduleAcc.GetAddress()).Return(moduleAcc)
	suite.authKeeper.EXPECT().HasAccount(suite.ctx, accAddr).Return(true)
}

func (suite *KeeperTestSuite) mockBurnCoins(moduleAcc *authtypes.ModuleAccount) {
	suite.authKeeper.EXPECT().GetModuleAccount(suite.ctx, moduleAcc.Name).Return(moduleAcc)
	suite.authKeeper.EXPECT().GetAccount(suite.ctx, moduleAcc.GetAddress()).Return(moduleAcc)
}

func (suite *KeeperTestSuite) mockSendCoinsFromModuleToModule(sender *authtypes.ModuleAccount, receiver *authtypes.ModuleAccount) {
	suite.authKeeper.EXPECT().GetModuleAddress(sender.Name).Return(sender.GetAddress())
	suite.authKeeper.EXPECT().GetModuleAccount(suite.ctx, receiver.Name).Return(receiver)
	suite.authKeeper.EXPECT().GetAccount(suite.ctx, sender.GetAddress()).Return(sender)
	suite.authKeeper.EXPECT().HasAccount(suite.ctx, receiver.GetAddress()).Return(true)
}

func (suite *KeeperTestSuite) mockSendCoinsFromAccountToModule(acc *authtypes.BaseAccount, moduleAcc *authtypes.ModuleAccount) {
	suite.authKeeper.EXPECT().GetModuleAccount(suite.ctx, moduleAcc.Name).Return(moduleAcc)
	suite.authKeeper.EXPECT().GetAccount(suite.ctx, acc.GetAddress()).Return(acc)
	suite.authKeeper.EXPECT().HasAccount(suite.ctx, moduleAcc.GetAddress()).Return(true)
}

func (suite *KeeperTestSuite) mockSendCoins(ctx sdk.Context, sender authtypes.AccountI, receiver sdk.AccAddress) {
	suite.authKeeper.EXPECT().GetAccount(ctx, sender.GetAddress()).Return(sender)
	suite.authKeeper.EXPECT().HasAccount(ctx, receiver).Return(true)
}

func (suite *KeeperTestSuite) mockFundAccount(receiver sdk.AccAddress) {
	suite.mockMintCoins(mintAcc)
	suite.mockSendCoinsFromModuleToAccount(mintAcc, receiver)
}

func (suite *KeeperTestSuite) mockInputOutputCoins(inputs []authtypes.AccountI, outputs []sdk.AccAddress) {
	for _, input := range inputs {
		suite.authKeeper.EXPECT().GetAccount(suite.ctx, input.GetAddress()).Return(input)
	}
	for _, output := range outputs {
		suite.authKeeper.EXPECT().HasAccount(suite.ctx, output).Return(true)
	}
}

func (suite *KeeperTestSuite) mockValidateBalance(acc authtypes.AccountI) {
	suite.authKeeper.EXPECT().GetAccount(suite.ctx, acc.GetAddress()).Return(acc)
}

func (suite *KeeperTestSuite) mockSpendableCoins(ctx sdk.Context, acc authtypes.AccountI) {
	suite.authKeeper.EXPECT().GetAccount(ctx, acc.GetAddress()).Return(acc)
}

func (suite *KeeperTestSuite) mockDelegateCoins(ctx sdk.Context, acc authtypes.AccountI, mAcc authtypes.AccountI) {
	vacc, ok := acc.(banktypes.VestingAccount)
	if ok {
		suite.authKeeper.EXPECT().SetAccount(ctx, vacc)
	}
	suite.authKeeper.EXPECT().GetAccount(ctx, acc.GetAddress()).Return(acc)
	suite.authKeeper.EXPECT().GetAccount(ctx, mAcc.GetAddress()).Return(mAcc)
}

func (suite *KeeperTestSuite) mockUnDelegateCoins(ctx sdk.Context, acc authtypes.AccountI, mAcc authtypes.AccountI) {
	vacc, ok := acc.(banktypes.VestingAccount)
	if ok {
		suite.authKeeper.EXPECT().SetAccount(ctx, vacc)
	}
	suite.authKeeper.EXPECT().GetAccount(ctx, acc.GetAddress()).Return(acc)
	suite.authKeeper.EXPECT().GetAccount(ctx, mAcc.GetAddress()).Return(mAcc)
	suite.authKeeper.EXPECT().GetAccount(ctx, mAcc.GetAddress()).Return(mAcc)
}

func (suite *KeeperTestSuite) TestGetAuthority() {
	NewKeeperWithAuthority := func(authority string) keeper.BaseKeeper {
		return keeper.NewBaseKeeper(
			moduletestutil.MakeTestEncodingConfig().Codec,
			sdk.NewKVStoreKey(banktypes.StoreKey),
			nil,
			nil,
			authority,
		)
	}

	tests := map[string]string{
		"some random account":    "cosmos139f7kncmglres2nf3h4hc4tade85ekfr8sulz5",
		"gov module account":     authtypes.NewModuleAddress(govtypes.ModuleName).String(),
		"another module account": authtypes.NewModuleAddress(minttypes.ModuleName).String(),
	}

	for name, expected := range tests {
		suite.T().Run(name, func(t *testing.T) {
			kpr := NewKeeperWithAuthority(expected)
			actual := kpr.GetAuthority()
			suite.Require().Equal(expected, actual)
		})
	}
}

func (suite *KeeperTestSuite) TestSupply() {
	ctx := suite.ctx
	require := suite.Require()
	keeper := suite.bankKeeper

	// add module accounts to supply keeper
	genesisSupply, _, err := keeper.GetPaginatedTotalSupply(ctx, &query.PageRequest{})
	require.NoError(err)

	initialPower := int64(100)
	initTokens := sdk.TokensFromConsensusPower(initialPower, sdk.DefaultPowerReduction)
	initCoins := sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, initTokens))

	// set burnerAcc balance
	suite.mockMintCoins(minterAcc)
	require.NoError(keeper.MintCoins(ctx, authtypes.Minter, initCoins))

	suite.mockSendCoinsFromModuleToAccount(minterAcc, burnerAcc.GetAddress())
	require.NoError(keeper.SendCoinsFromModuleToAccount(ctx, authtypes.Minter, burnerAcc.GetAddress(), initCoins))

	total, _, err := keeper.GetPaginatedTotalSupply(ctx, &query.PageRequest{})
	require.NoError(err)

	expTotalSupply := initCoins.Add(genesisSupply...)
	require.Equal(expTotalSupply, total)

	// burning all supplied tokens
	suite.mockBurnCoins(burnerAcc)
	require.NoError(keeper.BurnCoins(ctx, authtypes.Burner, initCoins))

	total, _, err = keeper.GetPaginatedTotalSupply(ctx, &query.PageRequest{})
	require.NoError(err)
	require.Equal(total, genesisSupply)
}

func (suite *KeeperTestSuite) TestSendCoinsFromModuleToAccount_Blocklist() {
	ctx := suite.ctx
	require := suite.Require()
	keeper := suite.bankKeeper

	suite.mockMintCoins(mintAcc)
	require.NoError(keeper.MintCoins(ctx, minttypes.ModuleName, initCoins))

	suite.authKeeper.EXPECT().GetModuleAddress(mintAcc.Name).Return(mintAcc.GetAddress())
	require.Error(keeper.SendCoinsFromModuleToAccount(
		ctx, minttypes.ModuleName, accAddrs[4], initCoins,
	))
}

func (suite *KeeperTestSuite) TestSupply_SendCoins() {
	ctx := suite.ctx
	require := suite.Require()
	authKeeper, keeper := suite.authKeeper, suite.bankKeeper

	// set initial balances
	suite.mockMintCoins(mintAcc)
	require.NoError(keeper.MintCoins(ctx, minttypes.ModuleName, initCoins))

	suite.mockSendCoinsFromModuleToAccount(mintAcc, holderAcc.GetAddress())
	require.NoError(keeper.SendCoinsFromModuleToAccount(ctx, minttypes.ModuleName, holderAcc.GetAddress(), initCoins))

	authKeeper.EXPECT().GetModuleAddress("").Return(nil)
	require.Panics(func() {
		_ = keeper.SendCoinsFromModuleToModule(ctx, "", holderAcc.GetName(), initCoins) // nolint:errcheck
	})

	authKeeper.EXPECT().GetModuleAddress(burnerAcc.Name).Return(burnerAcc.GetAddress())
	authKeeper.EXPECT().GetModuleAccount(ctx, "").Return(nil)
	require.Panics(func() {
		_ = keeper.SendCoinsFromModuleToModule(ctx, authtypes.Burner, "", initCoins) // nolint:errcheck
	})

	authKeeper.EXPECT().GetModuleAddress("").Return(nil)
	require.Panics(func() {
		_ = keeper.SendCoinsFromModuleToAccount(ctx, "", baseAcc.GetAddress(), initCoins) // nolint:errcheck
	})

	authKeeper.EXPECT().GetModuleAddress(holderAcc.Name).Return(holderAcc.GetAddress())
	authKeeper.EXPECT().GetAccount(suite.ctx, holderAcc.GetAddress()).Return(holderAcc)
	require.Error(
		keeper.SendCoinsFromModuleToAccount(ctx, holderAcc.GetName(), baseAcc.GetAddress(), initCoins.Add(initCoins...)),
	)

	suite.mockSendCoinsFromModuleToModule(holderAcc, burnerAcc)
	require.NoError(
		keeper.SendCoinsFromModuleToModule(ctx, holderAcc.GetName(), authtypes.Burner, initCoins),
	)

	require.Equal(sdk.NewCoins(), keeper.GetAllBalances(ctx, holderAcc.GetAddress()))
	require.Equal(initCoins, keeper.GetAllBalances(ctx, burnerAcc.GetAddress()))

	suite.mockSendCoinsFromModuleToAccount(burnerAcc, baseAcc.GetAddress())
	require.NoError(
		keeper.SendCoinsFromModuleToAccount(ctx, authtypes.Burner, baseAcc.GetAddress(), initCoins),
	)
	require.Equal(sdk.NewCoins(), keeper.GetAllBalances(ctx, burnerAcc.GetAddress()))
	require.Equal(initCoins, keeper.GetAllBalances(ctx, baseAcc.GetAddress()))

	suite.mockSendCoinsFromAccountToModule(baseAcc, burnerAcc)
	require.NoError(keeper.SendCoinsFromAccountToModule(ctx, baseAcc.GetAddress(), authtypes.Burner, initCoins))
	require.Equal(sdk.NewCoins(), keeper.GetAllBalances(ctx, baseAcc.GetAddress()))
	require.Equal(initCoins, keeper.GetAllBalances(ctx, burnerAcc.GetAddress()))
}

func (suite *KeeperTestSuite) TestSupply_MintCoins() {
	ctx := suite.ctx
	require := suite.Require()
	authKeeper, keeper := suite.authKeeper, suite.bankKeeper

	initialSupply, _, err := keeper.GetPaginatedTotalSupply(ctx, &query.PageRequest{})
	require.NoError(err)

	authKeeper.EXPECT().GetModuleAccount(ctx, "").Return(nil)
	require.Panics(func() { _ = keeper.MintCoins(ctx, "", initCoins) }, "no module account") // nolint:errcheck

	suite.mockMintCoins(burnerAcc)
	require.Panics(func() { _ = keeper.MintCoins(ctx, authtypes.Burner, initCoins) }, "invalid permission") // nolint:errcheck

	suite.mockMintCoins(minterAcc)
	require.Error(keeper.MintCoins(ctx, authtypes.Minter, sdk.Coins{sdk.Coin{Denom: "denom", Amount: sdk.NewInt(-10)}}), "insufficient coins")

	authKeeper.EXPECT().GetModuleAccount(ctx, randomPerm).Return(nil)
	require.Panics(func() { _ = keeper.MintCoins(ctx, randomPerm, initCoins) }) // nolint:errcheck

	suite.mockMintCoins(minterAcc)
	require.NoError(keeper.MintCoins(ctx, authtypes.Minter, initCoins))

	require.Equal(initCoins, keeper.GetAllBalances(ctx, minterAcc.GetAddress()))
	totalSupply, _, err := keeper.GetPaginatedTotalSupply(ctx, &query.PageRequest{})
	require.NoError(err)

	require.Equal(initialSupply.Add(initCoins...), totalSupply)

	// test same functionality on module account with multiple permissions
	initialSupply, _, err = keeper.GetPaginatedTotalSupply(ctx, &query.PageRequest{})
	require.NoError(err)

	suite.mockMintCoins(multiPermAcc)
	require.NoError(keeper.MintCoins(ctx, multiPermAcc.GetName(), initCoins))

	totalSupply, _, err = keeper.GetPaginatedTotalSupply(ctx, &query.PageRequest{})
	require.NoError(err)
	require.Equal(initCoins, keeper.GetAllBalances(ctx, multiPermAcc.GetAddress()))
	require.Equal(initialSupply.Add(initCoins...), totalSupply)
}

func (suite *KeeperTestSuite) TestSupply_BurnCoins() {
	ctx := suite.ctx
	require := suite.Require()
	authKeeper, keeper := suite.authKeeper, suite.bankKeeper

	// set burnerAcc balance
	suite.mockMintCoins(minterAcc)
	require.NoError(keeper.MintCoins(ctx, authtypes.Minter, initCoins))

	suite.mockSendCoinsFromModuleToAccount(minterAcc, burnerAcc.GetAddress())
	require.NoError(keeper.SendCoinsFromModuleToAccount(ctx, authtypes.Minter, burnerAcc.GetAddress(), initCoins))

	// inflate supply
	suite.mockMintCoins(minterAcc)
	require.NoError(keeper.MintCoins(ctx, authtypes.Minter, initCoins))

	supplyAfterInflation, _, err := keeper.GetPaginatedTotalSupply(ctx, &query.PageRequest{})
	require.NoError(err)

	authKeeper.EXPECT().GetModuleAccount(ctx, "").Return(nil)
	require.Panics(func() { _ = keeper.BurnCoins(ctx, "", initCoins) }, "no module account") // nolint:errcheck

	authKeeper.EXPECT().GetModuleAccount(ctx, minterAcc.Name).Return(nil)
	require.Panics(func() { _ = keeper.BurnCoins(ctx, authtypes.Minter, initCoins) }, "invalid permission") // nolint:errcheck

	authKeeper.EXPECT().GetModuleAccount(ctx, randomPerm).Return(nil)
	require.Panics(func() { _ = keeper.BurnCoins(ctx, randomPerm, supplyAfterInflation) }, "random permission") // nolint:errcheck

	suite.mockBurnCoins(burnerAcc)
	require.Error(keeper.BurnCoins(ctx, authtypes.Burner, supplyAfterInflation), "insufficient coins")

	suite.mockBurnCoins(burnerAcc)
	require.NoError(keeper.BurnCoins(ctx, authtypes.Burner, initCoins))

	supplyAfterBurn, _, err := keeper.GetPaginatedTotalSupply(ctx, &query.PageRequest{})
	require.NoError(err)
	require.Equal(sdk.NewCoins(), keeper.GetAllBalances(ctx, burnerAcc.GetAddress()))
	require.Equal(supplyAfterInflation.Sub(initCoins...), supplyAfterBurn)

	// test same functionality on module account with multiple permissions
	suite.mockMintCoins(minterAcc)
	require.NoError(keeper.MintCoins(ctx, authtypes.Minter, initCoins))

	supplyAfterInflation, _, err = keeper.GetPaginatedTotalSupply(ctx, &query.PageRequest{})
	require.NoError(err)

	suite.mockSendCoins(ctx, minterAcc, multiPermAcc.GetAddress())
	require.NoError(keeper.SendCoins(ctx, minterAcc.GetAddress(), multiPermAcc.GetAddress(), initCoins))

	suite.mockBurnCoins(multiPermAcc)
	require.NoError(keeper.BurnCoins(ctx, multiPermAcc.GetName(), initCoins))

	supplyAfterBurn, _, err = keeper.GetPaginatedTotalSupply(ctx, &query.PageRequest{})
	require.NoError(err)
	require.Equal(sdk.NewCoins(), keeper.GetAllBalances(ctx, multiPermAcc.GetAddress()))
	require.Equal(supplyAfterInflation.Sub(initCoins...), supplyAfterBurn)
}

func (suite *KeeperTestSuite) TestSendCoinsNewAccount() {
	ctx := suite.ctx
	require := suite.Require()
	balances := sdk.NewCoins(newFooCoin(100), newBarCoin(50))

	suite.mockFundAccount(accAddrs[0])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[0], balances))

	acc0 := authtypes.NewBaseAccountWithAddress(accAddrs[0])
	acc1Balances := suite.bankKeeper.GetAllBalances(ctx, accAddrs[0])
	require.Equal(balances, acc1Balances)

	suite.bankKeeper.GetAllBalances(ctx, accAddrs[1])
	require.Empty(suite.bankKeeper.GetAllBalances(ctx, accAddrs[1]))

	sendAmt := sdk.NewCoins(newFooCoin(50), newBarCoin(50))
	suite.mockSendCoins(ctx, acc0, accAddrs[1])
	require.NoError(suite.bankKeeper.SendCoins(ctx, accAddrs[0], accAddrs[1], sendAmt))

	acc2Balances := suite.bankKeeper.GetAllBalances(ctx, accAddrs[1])
	acc1Balances = suite.bankKeeper.GetAllBalances(ctx, accAddrs[0])
	require.Equal(sendAmt, acc2Balances)
	updatedAcc1Bal := balances.Sub(sendAmt...)
	require.Len(acc1Balances, len(updatedAcc1Bal))
	require.Equal(acc1Balances, updatedAcc1Bal)
}

func (suite *KeeperTestSuite) TestInputOutputNewAccount() {
	ctx := suite.ctx
	require := suite.Require()

	balances := sdk.NewCoins(newFooCoin(100), newBarCoin(50))

	suite.mockFundAccount(accAddrs[0])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[0], balances))

	acc1Balances := suite.bankKeeper.GetAllBalances(ctx, accAddrs[0])
	require.Equal(balances, acc1Balances)

	require.Empty(suite.bankKeeper.GetAllBalances(ctx, accAddrs[1]))

	suite.mockInputOutputCoins([]authtypes.AccountI{authtypes.NewBaseAccountWithAddress(accAddrs[0])}, []sdk.AccAddress{accAddrs[1]})
	inputs := []banktypes.Input{
		{Address: accAddrs[0].String(), Coins: sdk.NewCoins(newFooCoin(30), newBarCoin(10))},
	}
	outputs := []banktypes.Output{
		{Address: accAddrs[1].String(), Coins: sdk.NewCoins(newFooCoin(30), newBarCoin(10))},
	}

	require.NoError(suite.bankKeeper.InputOutputCoins(ctx, inputs, outputs))

	expected := sdk.NewCoins(newFooCoin(30), newBarCoin(10))
	acc2Balances := suite.bankKeeper.GetAllBalances(ctx, accAddrs[1])
	require.Equal(expected, acc2Balances)
}

func (suite *KeeperTestSuite) TestInputOutputCoins() {
	ctx := suite.ctx
	require := suite.Require()
	balances := sdk.NewCoins(newFooCoin(90), newBarCoin(30))

	acc0 := authtypes.NewBaseAccountWithAddress(accAddrs[0])
	inputs := []banktypes.Input{
		{Address: accAddrs[0].String(), Coins: sdk.NewCoins(newFooCoin(60), newBarCoin(20))},
	}
	outputs := []banktypes.Output{
		{Address: accAddrs[1].String(), Coins: sdk.NewCoins(newFooCoin(30), newBarCoin(10))},
		{Address: accAddrs[2].String(), Coins: sdk.NewCoins(newFooCoin(30), newBarCoin(10))},
	}

	require.Error(suite.bankKeeper.InputOutputCoins(ctx, inputs, []banktypes.Output{}))

	suite.authKeeper.EXPECT().GetAccount(suite.ctx, accAddrs[0]).Return(acc0)
	require.Error(suite.bankKeeper.InputOutputCoins(ctx, inputs, outputs))

	suite.mockFundAccount(accAddrs[0])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[0], balances))

	insufficientInputs := []banktypes.Input{
		{
			Address: accAddrs[0].String(),
			Coins:   sdk.NewCoins(newFooCoin(300), newBarCoin(100)),
		},
	}
	insufficientOutputs := []banktypes.Output{
		{Address: accAddrs[1].String(), Coins: sdk.NewCoins(newFooCoin(300), newBarCoin(100))},
		{Address: accAddrs[2].String(), Coins: sdk.NewCoins(newFooCoin(300), newBarCoin(100))},
	}

	require.Error(suite.bankKeeper.InputOutputCoins(ctx, insufficientInputs, insufficientOutputs))

	suite.mockInputOutputCoins([]authtypes.AccountI{acc0}, accAddrs[1:3])
	require.NoError(suite.bankKeeper.InputOutputCoins(ctx, inputs, outputs))

	acc1Balances := suite.bankKeeper.GetAllBalances(ctx, accAddrs[0])
	expected := sdk.NewCoins(newFooCoin(30), newBarCoin(10))
	require.Equal(expected, acc1Balances)

	acc2Balances := suite.bankKeeper.GetAllBalances(ctx, accAddrs[1])
	require.Equal(expected, acc2Balances)

	acc3Balances := suite.bankKeeper.GetAllBalances(ctx, accAddrs[2])
	require.Equal(expected, acc3Balances)
}

func (suite *KeeperTestSuite) TestSendCoins() {
	ctx := suite.ctx
	require := suite.Require()
	balances := sdk.NewCoins(newFooCoin(100), newBarCoin(50))

	acc0 := authtypes.NewBaseAccountWithAddress(accAddrs[0])

	suite.mockFundAccount(accAddrs[1])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[1], balances))

	sendAmt := sdk.NewCoins(newFooCoin(50), newBarCoin(25))
	suite.authKeeper.EXPECT().GetAccount(suite.ctx, accAddrs[0]).Return(acc0)
	require.Error(suite.bankKeeper.SendCoins(ctx, accAddrs[0], accAddrs[1], sendAmt))

	suite.mockFundAccount(accAddrs[0])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[0], balances))
	suite.mockSendCoins(ctx, acc0, accAddrs[1])
	require.NoError(suite.bankKeeper.SendCoins(ctx, accAddrs[0], accAddrs[1], sendAmt))

	acc1Balances := suite.bankKeeper.GetAllBalances(ctx, accAddrs[0])
	expected := sdk.NewCoins(newFooCoin(50), newBarCoin(25))
	require.Equal(expected, acc1Balances)

	acc2Balances := suite.bankKeeper.GetAllBalances(ctx, accAddrs[1])
	expected = sdk.NewCoins(newFooCoin(150), newBarCoin(75))
	require.Equal(expected, acc2Balances)

	// we sent all foo coins to acc2, so foo balance should be deleted for acc1 and bar should be still there
	var coins []sdk.Coin
	suite.bankKeeper.IterateAccountBalances(ctx, accAddrs[0], func(c sdk.Coin) (stop bool) {
		coins = append(coins, c)
		return true
	})
	require.Len(coins, 1)
	require.Equal(newBarCoin(25), coins[0], "expected only bar coins in the account balance, got: %v", coins)
}

func (suite *KeeperTestSuite) TestSendCoins_Invalid_SendLockedCoins() {
	balances := sdk.NewCoins(newFooCoin(50))

	now := tmtime.Now()
	endTime := now.Add(24 * time.Hour)

	origCoins := sdk.NewCoins(sdk.NewInt64Coin("stake", 100))
	sendCoins := sdk.NewCoins(sdk.NewInt64Coin("stake", 50))

	acc0 := authtypes.NewBaseAccountWithAddress(accAddrs[0])
	vacc := vesting.NewContinuousVestingAccount(acc0, origCoins, now.Unix(), endTime.Unix())

	suite.mockFundAccount(accAddrs[1])
	suite.Require().NoError(banktestutil.FundAccount(suite.bankKeeper, suite.ctx, accAddrs[1], balances))

	suite.authKeeper.EXPECT().GetAccount(suite.ctx, accAddrs[0]).Return(vacc)
	suite.Require().Error(suite.bankKeeper.SendCoins(suite.ctx, accAddrs[0], accAddrs[1], sendCoins))
}

func (suite *KeeperTestSuite) TestValidateBalance() {
	ctx := suite.ctx
	require := suite.Require()
	now := tmtime.Now()
	endTime := now.Add(24 * time.Hour)

	acc0 := authtypes.NewBaseAccountWithAddress(accAddrs[0])
	suite.authKeeper.EXPECT().GetAccount(suite.ctx, accAddrs[0]).Return(nil)
	require.Error(suite.bankKeeper.ValidateBalance(ctx, accAddrs[0]))

	balances := sdk.NewCoins(newFooCoin(100))
	suite.mockFundAccount(accAddrs[0])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[0], balances))

	suite.mockValidateBalance(acc0)
	require.NoError(suite.bankKeeper.ValidateBalance(ctx, accAddrs[0]))

	acc1 := authtypes.NewBaseAccountWithAddress(accAddrs[1])
	vacc := vesting.NewContinuousVestingAccount(acc1, balances.Add(balances...), now.Unix(), endTime.Unix())

	suite.mockFundAccount(accAddrs[1])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[1], balances))

	suite.mockValidateBalance(vacc)
	require.Error(suite.bankKeeper.ValidateBalance(ctx, accAddrs[1]))
}

func (suite *KeeperTestSuite) TestSendEnabled() {
	ctx := suite.ctx
	require := suite.Require()
	enabled := true
	params := banktypes.DefaultParams()
	require.Equal(enabled, params.DefaultSendEnabled)

	require.NoError(suite.bankKeeper.SetParams(ctx, params))

	bondCoin := sdk.NewCoin(sdk.DefaultBondDenom, math.OneInt())
	fooCoin := sdk.NewCoin("foocoin", math.OneInt())
	barCoin := sdk.NewCoin("barcoin", math.OneInt())

	// assert with default (all denom) send enabled both Bar and Bond Denom are enabled
	require.Equal(enabled, suite.bankKeeper.IsSendEnabledCoin(ctx, barCoin))
	require.Equal(enabled, suite.bankKeeper.IsSendEnabledCoin(ctx, bondCoin))

	// Both coins should be send enabled.
	err := suite.bankKeeper.IsSendEnabledCoins(ctx, fooCoin, bondCoin)
	require.NoError(err)

	// Set default send_enabled to !enabled, add a foodenom that overrides default as enabled
	params.DefaultSendEnabled = !enabled
	require.NoError(suite.bankKeeper.SetParams(ctx, params))
	suite.bankKeeper.SetSendEnabled(ctx, fooCoin.Denom, enabled)

	// Expect our specific override to be enabled, others to be !enabled.
	require.Equal(enabled, suite.bankKeeper.IsSendEnabledCoin(ctx, fooCoin))
	require.Equal(!enabled, suite.bankKeeper.IsSendEnabledCoin(ctx, barCoin))
	require.Equal(!enabled, suite.bankKeeper.IsSendEnabledCoin(ctx, bondCoin))

	// Foo coin should be send enabled.
	err = suite.bankKeeper.IsSendEnabledCoins(ctx, fooCoin)
	require.NoError(err)

	// Expect an error when one coin is not send enabled.
	err = suite.bankKeeper.IsSendEnabledCoins(ctx, fooCoin, bondCoin)
	require.Error(err)

	// Expect an error when all coins are not send enabled.
	err = suite.bankKeeper.IsSendEnabledCoins(ctx, bondCoin, barCoin)
	require.Error(err)
}

func (suite *KeeperTestSuite) TestHasBalance() {
	ctx := suite.ctx
	require := suite.Require()

	balances := sdk.NewCoins(newFooCoin(100))
	require.False(suite.bankKeeper.HasBalance(ctx, accAddrs[0], newFooCoin(99)))

	suite.mockFundAccount(accAddrs[0])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[0], balances))
	require.False(suite.bankKeeper.HasBalance(ctx, accAddrs[0], newFooCoin(101)))
	require.True(suite.bankKeeper.HasBalance(ctx, accAddrs[0], newFooCoin(100)))
	require.True(suite.bankKeeper.HasBalance(ctx, accAddrs[0], newFooCoin(1)))
}

func (suite *KeeperTestSuite) TestMsgSendEvents() {
	ctx := suite.ctx
	require := suite.Require()

	acc0 := authtypes.NewBaseAccountWithAddress(accAddrs[0])
	newCoins := sdk.NewCoins(sdk.NewInt64Coin(fooDenom, 50))
	suite.mockFundAccount(accAddrs[0])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[0], newCoins))

	suite.mockSendCoins(ctx, acc0, accAddrs[1])
	require.NoError(suite.bankKeeper.SendCoins(ctx, accAddrs[0], accAddrs[1], newCoins))
	event1 := sdk.Event{
		Type:       banktypes.EventTypeTransfer,
		Attributes: []abci.EventAttribute{},
	}
	event1.Attributes = append(
		event1.Attributes,
		abci.EventAttribute{Key: banktypes.AttributeKeyRecipient, Value: accAddrs[1].String()},
	)
	event1.Attributes = append(
		event1.Attributes,
		abci.EventAttribute{Key: banktypes.AttributeKeySender, Value: accAddrs[0].String()},
	)
	event1.Attributes = append(
		event1.Attributes,
		abci.EventAttribute{Key: sdk.AttributeKeyAmount, Value: newCoins.String()},
	)

	event2 := sdk.Event{
		Type:       sdk.EventTypeMessage,
		Attributes: []abci.EventAttribute{},
	}
	event2.Attributes = append(
		event2.Attributes,
		abci.EventAttribute{Key: banktypes.AttributeKeySender, Value: accAddrs[0].String()},
	)

	// events are shifted due to the funding account events
	events := ctx.EventManager().ABCIEvents()
	require.Equal(10, len(events))
	require.Equal(abci.Event(event1), events[8])
	require.Equal(abci.Event(event2), events[9])
}

func (suite *KeeperTestSuite) TestMsgMultiSendEvents() {
	ctx := suite.ctx
	require := suite.Require()
	acc0 := authtypes.NewBaseAccountWithAddress(accAddrs[0])

	require.NoError(suite.bankKeeper.SetParams(ctx, banktypes.DefaultParams()))

	coins := sdk.NewCoins(sdk.NewInt64Coin(fooDenom, 50), sdk.NewInt64Coin(barDenom, 100))
	newCoins := sdk.NewCoins(sdk.NewInt64Coin(fooDenom, 50))
	newCoins2 := sdk.NewCoins(sdk.NewInt64Coin(barDenom, 100))
	inputs := []banktypes.Input{
		{
			Address: accAddrs[0].String(),
			Coins:   coins,
		},
	}
	outputs := []banktypes.Output{
		{Address: accAddrs[2].String(), Coins: newCoins},
		{Address: accAddrs[3].String(), Coins: newCoins2},
	}

	suite.authKeeper.EXPECT().GetAccount(suite.ctx, accAddrs[0]).Return(acc0)
	require.Error(suite.bankKeeper.InputOutputCoins(ctx, inputs, outputs))

	events := ctx.EventManager().ABCIEvents()
	require.Equal(0, len(events))

	// Set addr's coins but not accAddrs[1]'s coins
	suite.mockFundAccount(accAddrs[0])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[0], sdk.NewCoins(sdk.NewInt64Coin(fooDenom, 50), sdk.NewInt64Coin(barDenom, 100))))

	suite.mockInputOutputCoins([]authtypes.AccountI{acc0}, accAddrs[2:4])
	require.NoError(suite.bankKeeper.InputOutputCoins(ctx, inputs, outputs))

	events = ctx.EventManager().ABCIEvents()
	require.Equal(12, len(events)) // 12 events because account funding causes extra minting + coin_spent + coin_recv events

	event1 := sdk.Event{
		Type:       sdk.EventTypeMessage,
		Attributes: []abci.EventAttribute{},
	}
	event1.Attributes = append(
		event1.Attributes,
		abci.EventAttribute{Key: banktypes.AttributeKeySender, Value: accAddrs[0].String()},
	)
	require.Equal(abci.Event(event1), events[7])

	// Set addr's coins and accAddrs[1]'s coins
	suite.mockFundAccount(accAddrs[0])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[0], sdk.NewCoins(sdk.NewInt64Coin(fooDenom, 50))))
	newCoins = sdk.NewCoins(sdk.NewInt64Coin(fooDenom, 50))

	suite.mockFundAccount(accAddrs[0])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[0], sdk.NewCoins(sdk.NewInt64Coin(barDenom, 100))))
	newCoins2 = sdk.NewCoins(sdk.NewInt64Coin(barDenom, 100))

	suite.mockInputOutputCoins([]authtypes.AccountI{acc0}, accAddrs[2:4])
	require.NoError(suite.bankKeeper.InputOutputCoins(ctx, inputs, outputs))

	events = ctx.EventManager().ABCIEvents()
	require.Equal(30, len(events)) // 27 due to account funding + coin_spent + coin_recv events

	event2 := sdk.Event{
		Type:       banktypes.EventTypeTransfer,
		Attributes: []abci.EventAttribute{},
	}
	event2.Attributes = append(
		event2.Attributes,
		abci.EventAttribute{Key: banktypes.AttributeKeyRecipient, Value: accAddrs[2].String()},
	)
	event2.Attributes = append(
		event2.Attributes,
		abci.EventAttribute{Key: sdk.AttributeKeyAmount, Value: newCoins.String()})
	event3 := sdk.Event{
		Type:       banktypes.EventTypeTransfer,
		Attributes: []abci.EventAttribute{},
	}
	event3.Attributes = append(
		event3.Attributes,
		abci.EventAttribute{Key: banktypes.AttributeKeyRecipient, Value: accAddrs[3].String()},
	)
	event3.Attributes = append(
		event3.Attributes,
		abci.EventAttribute{Key: sdk.AttributeKeyAmount, Value: newCoins2.String()},
	)
	// events are shifted due to the funding account events
	require.Equal(abci.Event(event1), events[25])
	require.Equal(abci.Event(event2), events[27])
	require.Equal(abci.Event(event3), events[29])
}

func (suite *KeeperTestSuite) TestSpendableCoins() {
	ctx := suite.ctx
	require := suite.Require()
	now := tmtime.Now()
	endTime := now.Add(24 * time.Hour)

	origCoins := sdk.NewCoins(sdk.NewInt64Coin("stake", 100))
	lockedCoins := sdk.NewCoins(sdk.NewInt64Coin("stake", 50))

	acc0 := authtypes.NewBaseAccountWithAddress(accAddrs[0])
	acc1 := authtypes.NewBaseAccountWithAddress(accAddrs[1])
	vacc := vesting.NewContinuousVestingAccount(acc0, origCoins, ctx.BlockHeader().Time.Unix(), endTime.Unix())

	suite.mockFundAccount(accAddrs[0])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[0], origCoins))

	suite.mockFundAccount(accAddrs[1])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[1], origCoins))

	suite.mockSpendableCoins(ctx, acc1)
	require.Equal(origCoins, suite.bankKeeper.SpendableCoins(ctx, accAddrs[1]))

	suite.mockSpendableCoins(ctx, acc1)
	require.Equal(origCoins[0], suite.bankKeeper.SpendableCoin(ctx, accAddrs[1], "stake"))

	ctx = ctx.WithBlockTime(now.Add(12 * time.Hour))
	suite.mockSpendableCoins(ctx, vacc)
	require.Equal(origCoins.Sub(lockedCoins...), suite.bankKeeper.SpendableCoins(ctx, accAddrs[0]))

	suite.mockSpendableCoins(ctx, vacc)
	require.Equal(origCoins.Sub(lockedCoins...)[0], suite.bankKeeper.SpendableCoin(ctx, accAddrs[0], "stake"))
}

func (suite *KeeperTestSuite) TestVestingAccountSend() {
	ctx := suite.ctx
	require := suite.Require()
	now := tmtime.Now()
	endTime := now.Add(24 * time.Hour)

	origCoins := sdk.NewCoins(sdk.NewInt64Coin("stake", 100))
	sendCoins := sdk.NewCoins(sdk.NewInt64Coin("stake", 50))

	acc0 := authtypes.NewBaseAccountWithAddress(accAddrs[0])
	vacc := vesting.NewContinuousVestingAccount(acc0, origCoins, now.Unix(), endTime.Unix())

	suite.mockFundAccount(accAddrs[0])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[0], origCoins))

	// require that no coins be sendable at the beginning of the vesting schedule
	suite.authKeeper.EXPECT().GetAccount(ctx, accAddrs[0]).Return(vacc)
	require.Error(suite.bankKeeper.SendCoins(ctx, accAddrs[0], accAddrs[1], sendCoins))

	// receive some coins
	suite.mockFundAccount(accAddrs[0])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[0], sendCoins))
	// require that all vested coins are spendable plus any received
	ctx = ctx.WithBlockTime(now.Add(12 * time.Hour))
	suite.mockSendCoins(ctx, vacc, accAddrs[1])
	require.NoError(suite.bankKeeper.SendCoins(ctx, accAddrs[0], accAddrs[1], sendCoins))
	require.Equal(origCoins, suite.bankKeeper.GetAllBalances(ctx, accAddrs[0]))
}

func (suite *KeeperTestSuite) TestPeriodicVestingAccountSend() {
	ctx := suite.ctx
	require := suite.Require()
	now := tmtime.Now()
	origCoins := sdk.NewCoins(sdk.NewInt64Coin("stake", 100))
	sendCoins := sdk.NewCoins(sdk.NewInt64Coin("stake", 50))

	periods := vesting.Periods{
		vesting.Period{Length: int64(12 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin("stake", 50)}},
		vesting.Period{Length: int64(6 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin("stake", 25)}},
		vesting.Period{Length: int64(6 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin("stake", 25)}},
	}

	acc0 := authtypes.NewBaseAccountWithAddress(accAddrs[0])
	vacc := vesting.NewPeriodicVestingAccount(acc0, origCoins, ctx.BlockHeader().Time.Unix(), periods)

	suite.mockFundAccount(accAddrs[0])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[0], origCoins))

	// require that no coins be sendable at the beginning of the vesting schedule
	suite.authKeeper.EXPECT().GetAccount(ctx, accAddrs[0]).Return(vacc)
	require.Error(suite.bankKeeper.SendCoins(ctx, accAddrs[0], accAddrs[1], sendCoins))

	// receive some coins
	suite.mockFundAccount(accAddrs[0])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[0], sendCoins))

	// require that all vested coins are spendable plus any received
	ctx = ctx.WithBlockTime(now.Add(12 * time.Hour))
	suite.mockSendCoins(ctx, vacc, accAddrs[1])
	require.NoError(suite.bankKeeper.SendCoins(ctx, accAddrs[0], accAddrs[1], sendCoins))
	require.Equal(origCoins, suite.bankKeeper.GetAllBalances(ctx, accAddrs[0]))
}

func (suite *KeeperTestSuite) TestVestingAccountReceive() {
	ctx := suite.ctx
	require := suite.Require()
	now := tmtime.Now()
	endTime := now.Add(24 * time.Hour)

	origCoins := sdk.NewCoins(sdk.NewInt64Coin("stake", 100))
	sendCoins := sdk.NewCoins(sdk.NewInt64Coin("stake", 50))

	acc0 := authtypes.NewBaseAccountWithAddress(accAddrs[0])
	acc1 := authtypes.NewBaseAccountWithAddress(accAddrs[1])
	vacc := vesting.NewContinuousVestingAccount(acc0, origCoins, ctx.BlockHeader().Time.Unix(), endTime.Unix())

	suite.mockFundAccount(accAddrs[0])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[0], origCoins))

	suite.mockFundAccount(accAddrs[1])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[1], origCoins))

	// send some coins to the vesting account
	suite.mockSendCoins(ctx, acc1, accAddrs[0])
	require.NoError(suite.bankKeeper.SendCoins(ctx, accAddrs[1], accAddrs[0], sendCoins))

	// require the coins are spendable
	balances := suite.bankKeeper.GetAllBalances(ctx, accAddrs[0])
	require.Equal(origCoins.Add(sendCoins...), balances)
	require.Equal(balances.Sub(vacc.LockedCoins(now)...), sendCoins)

	// require coins are spendable plus any that have vested
	require.Equal(balances.Sub(vacc.LockedCoins(now.Add(12*time.Hour))...), origCoins)
}

func (suite *KeeperTestSuite) TestPeriodicVestingAccountReceive() {
	ctx := suite.ctx
	require := suite.Require()
	now := tmtime.Now()

	origCoins := sdk.NewCoins(sdk.NewInt64Coin("stake", 100))
	sendCoins := sdk.NewCoins(sdk.NewInt64Coin("stake", 50))

	acc0 := authtypes.NewBaseAccountWithAddress(accAddrs[0])
	acc1 := authtypes.NewBaseAccountWithAddress(accAddrs[1])
	periods := vesting.Periods{
		vesting.Period{Length: int64(12 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin("stake", 50)}},
		vesting.Period{Length: int64(6 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin("stake", 25)}},
		vesting.Period{Length: int64(6 * 60 * 60), Amount: sdk.Coins{sdk.NewInt64Coin("stake", 25)}},
	}

	vacc := vesting.NewPeriodicVestingAccount(acc0, origCoins, ctx.BlockHeader().Time.Unix(), periods)

	suite.mockFundAccount(accAddrs[0])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[0], origCoins))

	suite.mockFundAccount(accAddrs[1])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[1], origCoins))

	// send some coins to the vesting account
	suite.mockSendCoins(ctx, acc1, accAddrs[0])
	require.NoError(suite.bankKeeper.SendCoins(ctx, accAddrs[1], accAddrs[0], sendCoins))

	// require the coins are spendable
	balances := suite.bankKeeper.GetAllBalances(ctx, accAddrs[0])
	require.Equal(origCoins.Add(sendCoins...), balances)
	require.Equal(balances.Sub(vacc.LockedCoins(now)...), sendCoins)

	// require coins are spendable plus any that have vested
	require.Equal(balances.Sub(vacc.LockedCoins(now.Add(12*time.Hour))...), origCoins)
}

func (suite *KeeperTestSuite) TestDelegateCoins() {
	ctx := suite.ctx
	require := suite.Require()
	now := tmtime.Now()
	endTime := now.Add(24 * time.Hour)

	origCoins := sdk.NewCoins(sdk.NewInt64Coin("stake", 100))
	delCoins := sdk.NewCoins(sdk.NewInt64Coin("stake", 50))

	acc0 := authtypes.NewBaseAccountWithAddress(accAddrs[0])
	acc1 := authtypes.NewBaseAccountWithAddress(accAddrs[1])
	vacc := vesting.NewContinuousVestingAccount(acc0, origCoins, ctx.BlockHeader().Time.Unix(), endTime.Unix())

	suite.mockFundAccount(accAddrs[0])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[0], origCoins))

	suite.mockFundAccount(accAddrs[1])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[1], origCoins))

	ctx = ctx.WithBlockTime(now.Add(12 * time.Hour))

	// require the ability for a non-vesting account to delegate
	suite.mockDelegateCoins(ctx, acc1, holderAcc)
	require.NoError(suite.bankKeeper.DelegateCoins(ctx, accAddrs[1], holderAcc.GetAddress(), delCoins))
	require.Equal(origCoins.Sub(delCoins...), suite.bankKeeper.GetAllBalances(ctx, accAddrs[1]))
	require.Equal(delCoins, suite.bankKeeper.GetAllBalances(ctx, holderAcc.GetAddress()))

	// require the ability for a vesting account to delegate
	suite.mockDelegateCoins(ctx, vacc, holderAcc)
	require.NoError(suite.bankKeeper.DelegateCoins(ctx, accAddrs[0], holderAcc.GetAddress(), delCoins))
	require.Equal(delCoins, suite.bankKeeper.GetAllBalances(ctx, accAddrs[0]))

	// require that delegated vesting amount is equal to what was delegated with DelegateCoins
	require.Equal(delCoins, vacc.GetDelegatedVesting())
}

func (suite *KeeperTestSuite) TestDelegateCoins_Invalid() {
	ctx := suite.ctx
	require := suite.Require()

	origCoins := sdk.NewCoins(newFooCoin(100))
	delCoins := sdk.NewCoins(newFooCoin(50))

	suite.authKeeper.EXPECT().GetAccount(ctx, holderAcc.GetAddress()).Return(nil)
	require.Error(suite.bankKeeper.DelegateCoins(ctx, accAddrs[0], holderAcc.GetAddress(), delCoins))

	suite.authKeeper.EXPECT().GetAccount(ctx, holderAcc.GetAddress()).Return(holderAcc)
	invalidCoins := sdk.Coins{sdk.Coin{Denom: "fooDenom", Amount: sdk.NewInt(-50)}}
	require.Error(suite.bankKeeper.DelegateCoins(ctx, accAddrs[0], holderAcc.GetAddress(), invalidCoins))

	suite.authKeeper.EXPECT().GetAccount(ctx, holderAcc.GetAddress()).Return(holderAcc)
	require.Error(suite.bankKeeper.DelegateCoins(ctx, accAddrs[0], holderAcc.GetAddress(), delCoins))

	suite.authKeeper.EXPECT().GetAccount(ctx, holderAcc.GetAddress()).Return(holderAcc)
	require.Error(suite.bankKeeper.DelegateCoins(ctx, accAddrs[0], holderAcc.GetAddress(), origCoins.Add(origCoins...)))
}

func (suite *KeeperTestSuite) TestUndelegateCoins() {
	ctx := suite.ctx
	require := suite.Require()
	now := tmtime.Now()
	endTime := now.Add(24 * time.Hour)

	origCoins := sdk.NewCoins(sdk.NewInt64Coin("stake", 100))
	delCoins := sdk.NewCoins(sdk.NewInt64Coin("stake", 50))

	acc0 := authtypes.NewBaseAccountWithAddress(accAddrs[0])
	acc1 := authtypes.NewBaseAccountWithAddress(accAddrs[1])
	vacc := vesting.NewContinuousVestingAccount(acc0, origCoins, ctx.BlockHeader().Time.Unix(), endTime.Unix())

	suite.mockFundAccount(accAddrs[0])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[0], origCoins))

	suite.mockFundAccount(accAddrs[1])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[1], origCoins))

	ctx = ctx.WithBlockTime(now.Add(12 * time.Hour))

	// require the ability for a non-vesting account to delegate
	suite.mockDelegateCoins(ctx, acc1, holderAcc)
	require.NoError(suite.bankKeeper.DelegateCoins(ctx, accAddrs[1], holderAcc.GetAddress(), delCoins))

	require.Equal(origCoins.Sub(delCoins...), suite.bankKeeper.GetAllBalances(ctx, accAddrs[1]))
	require.Equal(delCoins, suite.bankKeeper.GetAllBalances(ctx, holderAcc.GetAddress()))

	// require the ability for a non-vesting account to undelegate
	suite.mockUnDelegateCoins(ctx, acc1, holderAcc)
	require.NoError(suite.bankKeeper.UndelegateCoins(ctx, holderAcc.GetAddress(), accAddrs[1], delCoins))

	require.Equal(origCoins, suite.bankKeeper.GetAllBalances(ctx, accAddrs[1]))
	require.True(suite.bankKeeper.GetAllBalances(ctx, holderAcc.GetAddress()).Empty())

	// require the ability for a vesting account to delegate
	suite.mockDelegateCoins(ctx, acc0, holderAcc)
	require.NoError(suite.bankKeeper.DelegateCoins(ctx, accAddrs[0], holderAcc.GetAddress(), delCoins))

	require.Equal(origCoins.Sub(delCoins...), suite.bankKeeper.GetAllBalances(ctx, accAddrs[0]))
	require.Equal(delCoins, suite.bankKeeper.GetAllBalances(ctx, holderAcc.GetAddress()))

	// require the ability for a vesting account to undelegate
	suite.mockUnDelegateCoins(ctx, vacc, holderAcc)
	require.NoError(suite.bankKeeper.UndelegateCoins(ctx, holderAcc.GetAddress(), accAddrs[0], delCoins))

	require.Equal(origCoins, suite.bankKeeper.GetAllBalances(ctx, accAddrs[0]))
	require.True(suite.bankKeeper.GetAllBalances(ctx, holderAcc.GetAddress()).Empty())

	// require that delegated vesting amount is completely empty, since they were completely undelegated
	require.Empty(vacc.GetDelegatedVesting())
}

func (suite *KeeperTestSuite) TestUndelegateCoins_Invalid() {
	ctx := suite.ctx
	require := suite.Require()

	origCoins := sdk.NewCoins(newFooCoin(100))
	delCoins := sdk.NewCoins(newFooCoin(50))

	acc0 := authtypes.NewBaseAccountWithAddress(accAddrs[0])

	suite.authKeeper.EXPECT().GetAccount(ctx, holderAcc.GetAddress()).Return(nil)
	require.Error(suite.bankKeeper.UndelegateCoins(ctx, holderAcc.GetAddress(), accAddrs[0], delCoins))

	suite.mockFundAccount(accAddrs[0])
	require.NoError(banktestutil.FundAccount(suite.bankKeeper, ctx, accAddrs[0], origCoins))

	suite.mockDelegateCoins(ctx, acc0, holderAcc)
	require.NoError(suite.bankKeeper.DelegateCoins(ctx, accAddrs[0], holderAcc.GetAddress(), delCoins))

	suite.authKeeper.EXPECT().GetAccount(ctx, holderAcc.GetAddress()).Return(holderAcc)
	suite.authKeeper.EXPECT().GetAccount(ctx, holderAcc.GetAddress()).Return(holderAcc)
	suite.authKeeper.EXPECT().GetAccount(ctx, acc0.GetAddress()).Return(nil)
	require.Error(suite.bankKeeper.UndelegateCoins(ctx, holderAcc.GetAddress(), accAddrs[0], delCoins))

	suite.authKeeper.EXPECT().GetAccount(ctx, holderAcc.GetAddress()).Return(holderAcc)
	suite.authKeeper.EXPECT().GetAccount(ctx, holderAcc.GetAddress()).Return(holderAcc)
	require.Error(suite.bankKeeper.UndelegateCoins(ctx, holderAcc.GetAddress(), accAddrs[0], origCoins))
}

func (suite *KeeperTestSuite) TestSetDenomMetaData() {
	ctx := suite.ctx
	require := suite.Require()

	metadata := suite.getTestMetadata()

	for i := range []int{1, 2} {
		suite.bankKeeper.SetDenomMetaData(ctx, metadata[i])
	}

	actualMetadata, found := suite.bankKeeper.GetDenomMetaData(ctx, metadata[1].Base)
	require.True(found)
	found = suite.bankKeeper.HasDenomMetaData(ctx, metadata[1].Base)
	require.True(found)
	require.Equal(metadata[1].GetBase(), actualMetadata.GetBase())
	require.Equal(metadata[1].GetDisplay(), actualMetadata.GetDisplay())
	require.Equal(metadata[1].GetDescription(), actualMetadata.GetDescription())
	require.Equal(metadata[1].GetDenomUnits()[1].GetDenom(), actualMetadata.GetDenomUnits()[1].GetDenom())
	require.Equal(metadata[1].GetDenomUnits()[1].GetExponent(), actualMetadata.GetDenomUnits()[1].GetExponent())
	require.Equal(metadata[1].GetDenomUnits()[1].GetAliases(), actualMetadata.GetDenomUnits()[1].GetAliases())
}

func (suite *KeeperTestSuite) TestIterateAllDenomMetaData() {
	ctx := suite.ctx
	require := suite.Require()

	expectedMetadata := suite.getTestMetadata()
	// set metadata
	for i := range []int{1, 2} {
		suite.bankKeeper.SetDenomMetaData(ctx, expectedMetadata[i])
	}
	// retrieve metadata
	actualMetadata := make([]banktypes.Metadata, 0)
	suite.bankKeeper.IterateAllDenomMetaData(ctx, func(metadata banktypes.Metadata) bool {
		actualMetadata = append(actualMetadata, metadata)
		return false
	})
	// execute checks
	for i := range []int{1, 2} {
		require.Equal(expectedMetadata[i].GetBase(), actualMetadata[i].GetBase())
		require.Equal(expectedMetadata[i].GetDisplay(), actualMetadata[i].GetDisplay())
		require.Equal(expectedMetadata[i].GetDescription(), actualMetadata[i].GetDescription())
		require.Equal(expectedMetadata[i].GetDenomUnits()[1].GetDenom(), actualMetadata[i].GetDenomUnits()[1].GetDenom())
		require.Equal(expectedMetadata[i].GetDenomUnits()[1].GetExponent(), actualMetadata[i].GetDenomUnits()[1].GetExponent())
		require.Equal(expectedMetadata[i].GetDenomUnits()[1].GetAliases(), actualMetadata[i].GetDenomUnits()[1].GetAliases())
	}
}

func (suite *KeeperTestSuite) TestBalanceTrackingEvents() {
	require := suite.Require()

	// mint coins
	suite.mockMintCoins(multiPermAcc)
	require.NoError(
		suite.bankKeeper.MintCoins(
			suite.ctx,
			multiPermAcc.Name,
			sdk.NewCoins(sdk.NewCoin("utxo", sdk.NewInt(100000)))),
	)
	// send coins to address
	suite.mockSendCoinsFromModuleToAccount(multiPermAcc, accAddrs[0])
	require.NoError(
		suite.bankKeeper.SendCoinsFromModuleToAccount(
			suite.ctx,
			multiPermAcc.Name,
			accAddrs[0],
			sdk.NewCoins(sdk.NewCoin("utxo", sdk.NewInt(50000))),
		),
	)

	// burn coins from module account
	suite.mockBurnCoins(multiPermAcc)
	require.NoError(
		suite.bankKeeper.BurnCoins(
			suite.ctx,
			multiPermAcc.Name,
			sdk.NewCoins(sdk.NewInt64Coin("utxo", 1000)),
		),
	)

	// process balances and supply from events
	supply := sdk.NewCoins()

	balances := make(map[string]sdk.Coins)

	for _, e := range suite.ctx.EventManager().ABCIEvents() {
		switch e.Type {
		case banktypes.EventTypeCoinBurn:
			burnedCoins, err := sdk.ParseCoinsNormalized((string)(e.Attributes[1].Value))
			require.NoError(err)
			supply = supply.Sub(burnedCoins...)

		case banktypes.EventTypeCoinMint:
			mintedCoins, err := sdk.ParseCoinsNormalized((string)(e.Attributes[1].Value))
			require.NoError(err)
			supply = supply.Add(mintedCoins...)

		case banktypes.EventTypeCoinSpent:
			coinsSpent, err := sdk.ParseCoinsNormalized((string)(e.Attributes[1].Value))
			require.NoError(err)
			spender, err := sdk.AccAddressFromBech32((string)(e.Attributes[0].Value))
			require.NoError(err)
			balances[spender.String()] = balances[spender.String()].Sub(coinsSpent...)

		case banktypes.EventTypeCoinReceived:
			coinsRecv, err := sdk.ParseCoinsNormalized((string)(e.Attributes[1].Value))
			require.NoError(err)
			receiver, err := sdk.AccAddressFromBech32((string)(e.Attributes[0].Value))
			require.NoError(err)
			balances[receiver.String()] = balances[receiver.String()].Add(coinsRecv...)
		}
	}

	// check balance and supply tracking
	require.True(suite.bankKeeper.HasSupply(suite.ctx, "utxo"))
	savedSupply := suite.bankKeeper.GetSupply(suite.ctx, "utxo")
	utxoSupply := savedSupply
	require.Equal(utxoSupply.Amount, supply.AmountOf("utxo"))
	// iterate accounts and check balances
	suite.bankKeeper.IterateAllBalances(suite.ctx, func(address sdk.AccAddress, coin sdk.Coin) (stop bool) {
		// if it's not utxo coin then skip
		if coin.Denom != "utxo" {
			return false
		}

		balance, exists := balances[address.String()]
		require.True(exists)

		expectedUtxo := sdk.NewCoin("utxo", balance.AmountOf(coin.Denom))
		require.Equal(expectedUtxo.String(), coin.String())
		return false
	})
}

func (suite *KeeperTestSuite) getTestMetadata() []banktypes.Metadata {
	return []banktypes.Metadata{
		{
			Name:        "Cosmos Hub Atom",
			Symbol:      "ATOM",
			Description: "The native staking token of the Cosmos Hub.",
			DenomUnits: []*banktypes.DenomUnit{
				{Denom: "uatom", Exponent: uint32(0), Aliases: []string{"microatom"}},
				{Denom: "matom", Exponent: uint32(3), Aliases: []string{"milliatom"}},
				{Denom: "atom", Exponent: uint32(6), Aliases: nil},
			},
			Base:    "uatom",
			Display: "atom",
		},
		{
			Name:        "Token",
			Symbol:      "TOKEN",
			Description: "The native staking token of the Token Hub.",
			DenomUnits: []*banktypes.DenomUnit{
				{Denom: "1token", Exponent: uint32(5), Aliases: []string{"decitoken"}},
				{Denom: "2token", Exponent: uint32(4), Aliases: []string{"centitoken"}},
				{Denom: "3token", Exponent: uint32(7), Aliases: []string{"dekatoken"}},
			},
			Base:    "utoken",
			Display: "token",
		},
	}
}

func (suite *KeeperTestSuite) TestMintCoinRestrictions() {
	type BankMintingRestrictionFn func(ctx sdk.Context, coins sdk.Coins) error
	require := suite.Require()

	type testCase struct {
		coinsToTry sdk.Coin
		expectPass bool
	}

	tests := []struct {
		name          string
		restrictionFn BankMintingRestrictionFn
		testCases     []testCase
	}{
		{
			"restriction",
			func(_ sdk.Context, coins sdk.Coins) error {
				for _, coin := range coins {
					if coin.Denom != fooDenom {
						return fmt.Errorf("Module %s only has perms for minting %s coins, tried minting %s coins", banktypes.ModuleName, fooDenom, coin.Denom)
					}
				}
				return nil
			},
			[]testCase{
				{
					coinsToTry: newFooCoin(100),
					expectPass: true,
				},
				{
					coinsToTry: newBarCoin(100),
					expectPass: false,
				},
			},
		},
	}

	for _, test := range tests {
		keeper := suite.bankKeeper.WithMintCoinsRestriction(keeper.MintingRestrictionFn(test.restrictionFn))
		for _, testCase := range test.testCases {
			if testCase.expectPass {
				suite.mockMintCoins(multiPermAcc)
				require.NoError(
					keeper.MintCoins(
						suite.ctx,
						multiPermAcc.Name,
						sdk.NewCoins(testCase.coinsToTry),
					),
				)
			} else {
				require.Error(
					keeper.MintCoins(
						suite.ctx,
						multiPermAcc.Name,
						sdk.NewCoins(testCase.coinsToTry),
					),
				)
			}
		}
	}
}

func (suite *KeeperTestSuite) TestIsSendEnabledDenom() {
	ctx, bankKeeper := suite.ctx, suite.bankKeeper
	require := suite.Require()

	defaultCoin := "defaultCoin"
	enabledCoin := "enabledCoin"
	disabledCoin := "disabledCoin"
	bankKeeper.DeleteSendEnabled(ctx, defaultCoin)
	bankKeeper.SetSendEnabled(ctx, enabledCoin, true)
	bankKeeper.SetSendEnabled(ctx, disabledCoin, false)

	tests := []struct {
		denom  string
		exp    bool
		expDef bool
	}{
		{
			denom:  "defaultCoin",
			expDef: true,
		},
		{
			denom: enabledCoin,
			exp:   true,
		},
		{
			denom: disabledCoin,
			exp:   false,
		},
	}

	for _, def := range []bool{true, false} {
		params := banktypes.Params{DefaultSendEnabled: def}
		require.NoError(bankKeeper.SetParams(ctx, params))

		for _, tc := range tests {
			suite.T().Run(fmt.Sprintf("%s default %t", tc.denom, def), func(t *testing.T) {
				actual := suite.bankKeeper.IsSendEnabledDenom(suite.ctx, tc.denom)
				exp := tc.exp
				if tc.expDef {
					exp = def
				}

				require.Equal(exp, actual)
			})
		}
	}
}

func (suite *KeeperTestSuite) TestGetSendEnabledEntry() {
	ctx, bankKeeper := suite.ctx, suite.bankKeeper
	require := suite.Require()

	bankKeeper.SetAllSendEnabled(ctx, []*banktypes.SendEnabled{
		{Denom: "gettruecoin", Enabled: true},
		{Denom: "getfalsecoin", Enabled: false},
	})

	tests := []struct {
		denom string
		expSE banktypes.SendEnabled
		expF  bool
	}{
		{
			denom: "missing",
			expSE: banktypes.SendEnabled{},
			expF:  false,
		},
		{
			denom: "gettruecoin",
			expSE: banktypes.SendEnabled{Denom: "gettruecoin", Enabled: true},
			expF:  true,
		},
		{
			denom: "getfalsecoin",
			expSE: banktypes.SendEnabled{Denom: "getfalsecoin", Enabled: false},
			expF:  true,
		},
	}

	for _, tc := range tests {
		suite.T().Run(tc.denom, func(t *testing.T) {
			actualSE, actualF := bankKeeper.GetSendEnabledEntry(ctx, tc.denom)
			require.Equal(tc.expF, actualF, "found")
			require.Equal(tc.expSE, actualSE, "SendEnabled")
		})
	}
}

func (suite *KeeperTestSuite) TestSetSendEnabled() {
	ctx, bankKeeper := suite.ctx, suite.bankKeeper
	require := suite.Require()

	tests := []struct {
		name  string
		denom string
		value bool
	}{
		{
			name:  "very short denom true",
			denom: "f",
			value: true,
		},
		{
			name:  "very short denom false",
			denom: "f",
			value: true,
		},
		{
			name:  "falseFirstCoin false",
			denom: "falseFirstCoin",
			value: false,
		},
		{
			name:  "falseFirstCoin true",
			denom: "falseFirstCoin",
			value: true,
		},
		{
			name:  "falseFirstCoin true again",
			denom: "falseFirstCoin",
			value: true,
		},
		{
			name:  "trueFirstCoin true",
			denom: "falseFirstCoin",
			value: false,
		},
		{
			name:  "trueFirstCoin false",
			denom: "falseFirstCoin",
			value: false,
		},
		{
			name:  "trueFirstCoin false again",
			denom: "falseFirstCoin",
			value: false,
		},
	}

	for _, def := range []bool{true, false} {
		params := banktypes.Params{DefaultSendEnabled: def}
		require.NoError(bankKeeper.SetParams(ctx, params))

		for _, tc := range tests {
			suite.T().Run(fmt.Sprintf("%s default %t", tc.name, def), func(t *testing.T) {
				bankKeeper.SetSendEnabled(ctx, tc.denom, tc.value)
				actual := bankKeeper.IsSendEnabledDenom(ctx, tc.denom)
				require.Equal(tc.value, actual)
			})
		}
	}
}

func (suite *KeeperTestSuite) TestSetAllSendEnabled() {
	ctx, bankKeeper := suite.ctx, suite.bankKeeper
	require := suite.Require()

	tests := []struct {
		name         string
		sendEnableds []*banktypes.SendEnabled
	}{
		{
			name:         "nil",
			sendEnableds: nil,
		},
		{
			name:         "empty",
			sendEnableds: []*banktypes.SendEnabled{},
		},
		{
			name: "one true",
			sendEnableds: []*banktypes.SendEnabled{
				{Denom: "aonecoin", Enabled: true},
			},
		},
		{
			name: "one false",
			sendEnableds: []*banktypes.SendEnabled{
				{Denom: "bonecoin", Enabled: false},
			},
		},
		{
			name: "two true",
			sendEnableds: []*banktypes.SendEnabled{
				{Denom: "conecoin", Enabled: true},
				{Denom: "ctwocoin", Enabled: true},
			},
		},
		{
			name: "two true false",
			sendEnableds: []*banktypes.SendEnabled{
				{Denom: "donecoin", Enabled: true},
				{Denom: "dtwocoin", Enabled: false},
			},
		},
		{
			name: "two false true",
			sendEnableds: []*banktypes.SendEnabled{
				{Denom: "eonecoin", Enabled: false},
				{Denom: "etwocoin", Enabled: true},
			},
		},
		{
			name: "two false",
			sendEnableds: []*banktypes.SendEnabled{
				{Denom: "fonecoin", Enabled: false},
				{Denom: "ftwocoin", Enabled: false},
			},
		},
	}

	for _, def := range []bool{true, false} {
		params := banktypes.Params{DefaultSendEnabled: def}
		require.NoError(bankKeeper.SetParams(ctx, params))

		for _, tc := range tests {
			suite.T().Run(fmt.Sprintf("%s default %t", tc.name, def), func(t *testing.T) {
				bankKeeper.SetAllSendEnabled(ctx, tc.sendEnableds)

				for _, se := range tc.sendEnableds {
					actual := bankKeeper.IsSendEnabledDenom(ctx, se.Denom)
					require.Equal(se.Enabled, actual, se.Denom)
				}
			})
		}
	}
}

func (suite *KeeperTestSuite) TestDeleteSendEnabled() {
	ctx, bankKeeper := suite.ctx, suite.bankKeeper
	require := suite.Require()

	for _, def := range []bool{true, false} {
		params := banktypes.Params{DefaultSendEnabled: def}
		require.NoError(bankKeeper.SetParams(ctx, params))
		suite.T().Run(fmt.Sprintf("default %t", def), func(t *testing.T) {
			denom := fmt.Sprintf("somerand%tcoin", !def)
			bankKeeper.SetSendEnabled(ctx, denom, !def)
			require.Equal(!def, bankKeeper.IsSendEnabledDenom(ctx, denom))
			bankKeeper.DeleteSendEnabled(ctx, denom)
			require.Equal(def, bankKeeper.IsSendEnabledDenom(ctx, denom))
		})
	}
}

func (suite *KeeperTestSuite) TestIterateSendEnabledEntries() {
	ctx, bankKeeper := suite.ctx, suite.bankKeeper
	require := suite.Require()

	suite.T().Run("no entries to iterate", func(t *testing.T) {
		count := 0
		bankKeeper.IterateSendEnabledEntries(ctx, func(_ string, _ bool) (stop bool) {
			count++
			return false
		})

		require.Equal(0, count)
	})

	alpha := strings.Split("abcdefghijklmnopqrstuvwxyz", "")
	denoms := make([]string, len(alpha)*2)
	for i, l := range alpha {
		denoms[i*2] = fmt.Sprintf("%sitercointrue", l)
		denoms[i*2+1] = fmt.Sprintf("%sitercoinfalse", l)
		bankKeeper.SetSendEnabled(ctx, denoms[i*2], true)
		bankKeeper.SetSendEnabled(ctx, denoms[i*2+1], false)
	}

	for _, def := range []bool{true, false} {
		params := banktypes.Params{DefaultSendEnabled: def}
		require.NoError(bankKeeper.SetParams(ctx, params))

		var seen []string
		suite.T().Run(fmt.Sprintf("all denoms have expected values default %t", def), func(t *testing.T) {
			bankKeeper.IterateSendEnabledEntries(ctx, func(denom string, sendEnabled bool) (stop bool) {
				seen = append(seen, denom)
				exp := true
				if strings.HasSuffix(denom, "false") {
					exp = false
				}

				require.Equal(exp, sendEnabled, denom)
				return false
			})
		})

		suite.T().Run(fmt.Sprintf("all denoms were seen default %t", def), func(t *testing.T) {
			require.ElementsMatch(denoms, seen)
		})
	}

	bankKeeper.DeleteSendEnabled(ctx, denoms...)

	suite.T().Run("no entries to iterate again after deleting all of them", func(t *testing.T) {
		count := 0
		bankKeeper.IterateSendEnabledEntries(ctx, func(_ string, _ bool) (stop bool) {
			count++
			return false
		})

		require.Equal(0, count)
	})
}

func (suite *KeeperTestSuite) TestGetAllSendEnabledEntries() {
	ctx, bankKeeper := suite.ctx, suite.bankKeeper
	require := suite.Require()

	suite.T().Run("no entries", func(t *testing.T) {
		actual := bankKeeper.GetAllSendEnabledEntries(ctx)
		require.Len(actual, 0)
	})

	alpha := strings.Split("abcdefghijklmnopqrstuvwxyz", "")
	denoms := make([]string, len(alpha)*2)
	for i, l := range alpha {
		denoms[i*2] = fmt.Sprintf("%sitercointrue", l)
		denoms[i*2+1] = fmt.Sprintf("%sitercoinfalse", l)
		bankKeeper.SetSendEnabled(ctx, denoms[i*2], true)
		bankKeeper.SetSendEnabled(ctx, denoms[i*2+1], false)
	}

	for _, def := range []bool{true, false} {
		params := banktypes.Params{DefaultSendEnabled: def}
		require.NoError(bankKeeper.SetParams(ctx, params))

		var seen []string
		suite.T().Run(fmt.Sprintf("all denoms have expected values default %t", def), func(t *testing.T) {
			actual := bankKeeper.GetAllSendEnabledEntries(ctx)
			for _, se := range actual {
				seen = append(seen, se.Denom)
				exp := true
				if strings.HasSuffix(se.Denom, "false") {
					exp = false
				}

				require.Equal(exp, se.Enabled, se.Denom)
			}
		})

		suite.T().Run(fmt.Sprintf("all denoms were seen default %t", def), func(t *testing.T) {
			require.ElementsMatch(denoms, seen)
		})
	}

	for _, denom := range denoms {
		bankKeeper.DeleteSendEnabled(ctx, denom)
	}

	suite.T().Run("no entries again after deleting all of them", func(t *testing.T) {
		actual := bankKeeper.GetAllSendEnabledEntries(ctx)
		require.Len(actual, 0)
	})
}

type mockSubspace struct {
	ps banktypes.Params
}

func (ms mockSubspace) GetParamSet(ctx sdk.Context, ps exported.ParamSet) {
	*ps.(*banktypes.Params) = ms.ps
}

func (suite *KeeperTestSuite) TestMigrator_Migrate3to4() {
	ctx, bankKeeper := suite.ctx, suite.bankKeeper
	require := suite.Require()

	for _, def := range []bool{true, false} {
		params := banktypes.Params{DefaultSendEnabled: def}
		require.NoError(bankKeeper.SetParams(ctx, params))

		suite.T().Run(fmt.Sprintf("default %t does not change", def), func(t *testing.T) {
			legacySubspace := func(ps banktypes.Params) mockSubspace {
				return mockSubspace{ps: ps}
			}(banktypes.NewParams(def))

			migrator := keeper.NewMigrator(bankKeeper, legacySubspace)
			require.NoError(migrator.Migrate3to4(ctx))

			actual := bankKeeper.GetParams(ctx)
			require.Equal(params.DefaultSendEnabled, actual.DefaultSendEnabled)
		})
	}

	for _, def := range []bool{true, false} {
		params := banktypes.Params{
			SendEnabled: []*banktypes.SendEnabled{
				{Denom: fmt.Sprintf("truecoin%t", def), Enabled: true},
				{Denom: fmt.Sprintf("falsecoin%t", def), Enabled: false},
			},
		}

		require.NoError(bankKeeper.SetParams(ctx, params))

		suite.T().Run(fmt.Sprintf("default %t send enabled info moved to store", def), func(t *testing.T) {
			legacySubspace := func(ps banktypes.Params) mockSubspace {
				return mockSubspace{ps: ps}
			}(banktypes.NewParams(def))

			migrator := keeper.NewMigrator(bankKeeper, legacySubspace)
			require.NoError(migrator.Migrate3to4(ctx))

			newParams := bankKeeper.GetParams(ctx)
			require.Len(newParams.SendEnabled, 0)
			require.Equal(def, newParams.DefaultSendEnabled)

			for _, se := range params.SendEnabled {
				actual := bankKeeper.IsSendEnabledDenom(ctx, se.Denom)
				require.Equal(se.Enabled, actual, se.Denom)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestSetParams() {
	ctx, bankKeeper := suite.ctx, suite.bankKeeper
	require := suite.Require()

	params := banktypes.NewParams(true)
	params.SendEnabled = []*banktypes.SendEnabled{
		{Denom: "paramscointrue", Enabled: true},
		{Denom: "paramscoinfalse", Enabled: false},
	}
	require.NoError(bankKeeper.SetParams(ctx, params))

	suite.Run("stored params are as expected", func() {
		actual := bankKeeper.GetParams(ctx)
		require.True(actual.DefaultSendEnabled, "DefaultSendEnabled")
		require.Len(actual.SendEnabled, 0, "SendEnabled")
	})

	suite.Run("send enabled params converted to store", func() {
		actual := bankKeeper.GetAllSendEnabledEntries(ctx)
		if suite.Assert().Len(actual, 2) {
			require.Equal("paramscoinfalse", actual[0].Denom, "actual[0].Denom")
			require.False(actual[0].Enabled, "actual[0].Enabled")
			require.Equal("paramscointrue", actual[1].Denom, "actual[1].Denom")
			require.True(actual[1].Enabled, "actual[1].Enabled")
		}
	})
}
