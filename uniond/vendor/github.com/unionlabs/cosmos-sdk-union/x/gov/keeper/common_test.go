package keeper_test

import (
	"errors"
	"fmt"
	"testing"
	"time"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/core/header"
	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/log"
	"cosmossdk.io/math"
	storetypes "cosmossdk.io/store/types"
	banktypes "cosmossdk.io/x/bank/types"
	"cosmossdk.io/x/gov/keeper"
	govtestutil "cosmossdk.io/x/gov/testutil"
	"cosmossdk.io/x/gov/types"
	v1 "cosmossdk.io/x/gov/types/v1"
	"cosmossdk.io/x/gov/types/v1beta1"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

var (
	_, _, addr   = testdata.KeyTestPubAddr()
	govAcct      = authtypes.NewModuleAddress(types.ModuleName)
	poolAcct     = authtypes.NewModuleAddress(protocolModuleName)
	govAcctStr   = "cosmos10d07y265gmmuvt4z0w9aw880jnsr700j6zn9kn"
	TestProposal = getTestProposal()
)

// mintModuleName duplicates the mint module's name to avoid a cyclic dependency with x/mint.
// It should be synced with the mint module's name if it is ever changed.
// See: https://github.com/cosmos/cosmos-sdk/blob/0e34478eb7420b69869ed50f129fc274a97a9b06/x/mint/types/keys.go#L13
const (
	mintModuleName     = "mint"
	protocolModuleName = "protocolpool"
)

// getTestProposal creates and returns a test proposal message.
func getTestProposal() []sdk.Msg {
	moduleAddr, err := codectestutil.CodecOptions{}.GetAddressCodec().BytesToString(authtypes.NewModuleAddress(types.ModuleName))
	if err != nil {
		panic(err)
	}

	legacyProposalMsg, err := v1.NewLegacyContent(v1beta1.NewTextProposal("Title", "description"), moduleAddr)
	if err != nil {
		panic(err)
	}

	addrStr, err := codectestutil.CodecOptions{}.GetAddressCodec().BytesToString(addr)
	if err != nil {
		panic(err)
	}

	return []sdk.Msg{
		banktypes.NewMsgSend(govAcctStr, addrStr, sdk.NewCoins(sdk.NewCoin("stake", math.NewInt(1000)))),
		legacyProposalMsg,
	}
}

type mocks struct {
	acctKeeper    *govtestutil.MockAccountKeeper
	bankKeeper    *govtestutil.MockBankKeeper
	stakingKeeper *govtestutil.MockStakingKeeper
	poolKeeper    *govtestutil.MockPoolKeeper
}

func mockAccountKeeperExpectations(ctx sdk.Context, m mocks) {
	m.acctKeeper.EXPECT().GetModuleAddress(types.ModuleName).DoAndReturn(func(name string) sdk.AccAddress {
		if name == types.ModuleName {
			return govAcct
		} else if name == protocolModuleName {
			return poolAcct
		}
		panic(fmt.Sprintf("unexpected module name: %s", name))
	}).AnyTimes()
	m.acctKeeper.EXPECT().GetModuleAddress(protocolModuleName).Return(poolAcct).AnyTimes()
	m.acctKeeper.EXPECT().GetModuleAccount(gomock.Any(), types.ModuleName).Return(authtypes.NewEmptyModuleAccount(types.ModuleName)).AnyTimes()
	m.acctKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec("cosmos")).AnyTimes()
}

func mockDefaultExpectations(ctx sdk.Context, m mocks) error {
	mockAccountKeeperExpectations(ctx, m)
	err := trackMockBalances(m.bankKeeper)
	if err != nil {
		return err
	}
	m.stakingKeeper.EXPECT().TokensFromConsensusPower(ctx, gomock.Any()).DoAndReturn(func(ctx sdk.Context, power int64) math.Int {
		return sdk.TokensFromConsensusPower(power, math.NewIntFromUint64(1000000))
	}).AnyTimes()

	m.stakingKeeper.EXPECT().BondDenom(ctx).Return("stake", nil).AnyTimes()
	m.stakingKeeper.EXPECT().IterateBondedValidatorsByPower(gomock.Any(), gomock.Any()).AnyTimes()
	m.stakingKeeper.EXPECT().IterateDelegations(gomock.Any(), gomock.Any(), gomock.Any()).AnyTimes()
	m.stakingKeeper.EXPECT().TotalBondedTokens(gomock.Any()).Return(math.NewInt(10000000), nil).AnyTimes()
	return nil
}

// setupGovKeeper creates a govKeeper as well as all its dependencies.
func setupGovKeeper(t *testing.T, expectations ...func(sdk.Context, mocks)) (
	*keeper.Keeper,
	mocks,
	moduletestutil.TestEncodingConfig,
	sdk.Context,
) {
	t.Helper()
	key := storetypes.NewKVStoreKey(types.StoreKey)
	storeService := runtime.NewKVStoreService(key)
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	ctx := testCtx.Ctx.WithHeaderInfo(header.Info{Time: time.Now()})
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{})
	v1.RegisterInterfaces(encCfg.InterfaceRegistry)
	v1beta1.RegisterInterfaces(encCfg.InterfaceRegistry)
	banktypes.RegisterInterfaces(encCfg.InterfaceRegistry)

	baseApp := baseapp.NewBaseApp(
		"authz",
		log.NewNopLogger(),
		testCtx.DB,
		encCfg.TxConfig.TxDecoder(),
	)
	baseApp.SetCMS(testCtx.CMS)
	baseApp.SetInterfaceRegistry(encCfg.InterfaceRegistry)

	environment := runtime.NewEnvironment(storeService, coretesting.NewNopLogger(), runtime.EnvWithQueryRouterService(baseApp.GRPCQueryRouter()), runtime.EnvWithMsgRouterService(baseApp.MsgServiceRouter()))

	// gomock initializations
	ctrl := gomock.NewController(t)
	m := mocks{
		acctKeeper:    govtestutil.NewMockAccountKeeper(ctrl),
		bankKeeper:    govtestutil.NewMockBankKeeper(ctrl),
		stakingKeeper: govtestutil.NewMockStakingKeeper(ctrl),
		poolKeeper:    govtestutil.NewMockPoolKeeper(ctrl),
	}
	if len(expectations) == 0 {
		err := mockDefaultExpectations(ctx, m)
		require.NoError(t, err)
	} else {
		for _, exp := range expectations {
			exp(ctx, m)
		}
	}

	govAddr, err := m.acctKeeper.AddressCodec().BytesToString(govAcct)
	require.NoError(t, err)

	// Gov keeper initializations
	govKeeper := keeper.NewKeeper(encCfg.Codec, environment, m.acctKeeper, m.bankKeeper, m.stakingKeeper, m.poolKeeper, keeper.DefaultConfig(), govAddr)
	require.NoError(t, govKeeper.ProposalID.Set(ctx, 1))
	govRouter := v1beta1.NewRouter() // Also register legacy gov handlers to test them too.
	govRouter.AddRoute(types.RouterKey, v1beta1.ProposalHandler)
	govKeeper.SetLegacyRouter(govRouter)
	err = govKeeper.Params.Set(ctx, v1.DefaultParams())
	require.NoError(t, err)
	err = govKeeper.Constitution.Set(ctx, "constitution")
	require.NoError(t, err)

	// Register all handlers for the MegServiceRouter.
	v1.RegisterMsgServer(baseApp.MsgServiceRouter(), keeper.NewMsgServerImpl(govKeeper))
	banktypes.RegisterMsgServer(baseApp.MsgServiceRouter(), nil) // Nil is fine here as long as we never execute the proposal's Msgs.

	return govKeeper, m, encCfg, ctx
}

// setupGovKeeperWithMaxVoteOptionsLen creates a govKeeper with a defined maxVoteOptionsLen, as well as all its dependencies.
func setupGovKeeperWithMaxVoteOptionsLen(t *testing.T, maxVoteOptionsLen uint64, expectations ...func(sdk.Context, mocks)) (
	*keeper.Keeper,
	mocks,
	moduletestutil.TestEncodingConfig,
	sdk.Context,
) {
	t.Helper()
	key := storetypes.NewKVStoreKey(types.StoreKey)
	storeService := runtime.NewKVStoreService(key)
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	ctx := testCtx.Ctx.WithHeaderInfo(header.Info{Time: time.Now()})
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{})
	v1.RegisterInterfaces(encCfg.InterfaceRegistry)
	v1beta1.RegisterInterfaces(encCfg.InterfaceRegistry)
	banktypes.RegisterInterfaces(encCfg.InterfaceRegistry)

	baseApp := baseapp.NewBaseApp(
		"authz",
		log.NewNopLogger(),
		testCtx.DB,
		encCfg.TxConfig.TxDecoder(),
	)
	baseApp.SetCMS(testCtx.CMS)
	baseApp.SetInterfaceRegistry(encCfg.InterfaceRegistry)

	environment := runtime.NewEnvironment(storeService, coretesting.NewNopLogger(), runtime.EnvWithQueryRouterService(baseApp.GRPCQueryRouter()), runtime.EnvWithMsgRouterService(baseApp.MsgServiceRouter()))

	// gomock initializations
	ctrl := gomock.NewController(t)
	m := mocks{
		acctKeeper:    govtestutil.NewMockAccountKeeper(ctrl),
		bankKeeper:    govtestutil.NewMockBankKeeper(ctrl),
		stakingKeeper: govtestutil.NewMockStakingKeeper(ctrl),
		poolKeeper:    govtestutil.NewMockPoolKeeper(ctrl),
	}
	if len(expectations) == 0 {
		err := mockDefaultExpectations(ctx, m)
		require.NoError(t, err)
	} else {
		for _, exp := range expectations {
			exp(ctx, m)
		}
	}

	govAddr, err := m.acctKeeper.AddressCodec().BytesToString(govAcct)
	require.NoError(t, err)

	config := keeper.DefaultConfig()
	config.MaxVoteOptionsLen = maxVoteOptionsLen

	// Gov keeper initializations
	govKeeper := keeper.NewKeeper(encCfg.Codec, environment, m.acctKeeper, m.bankKeeper, m.stakingKeeper, m.poolKeeper, config, govAddr)
	require.NoError(t, govKeeper.ProposalID.Set(ctx, 1))
	govRouter := v1beta1.NewRouter() // Also register legacy gov handlers to test them too.
	govRouter.AddRoute(types.RouterKey, v1beta1.ProposalHandler)
	govKeeper.SetLegacyRouter(govRouter)
	err = govKeeper.Params.Set(ctx, v1.DefaultParams())
	require.NoError(t, err)
	err = govKeeper.Constitution.Set(ctx, "constitution")
	require.NoError(t, err)

	// Register all handlers for the MegServiceRouter.
	v1.RegisterMsgServer(baseApp.MsgServiceRouter(), keeper.NewMsgServerImpl(govKeeper))
	banktypes.RegisterMsgServer(baseApp.MsgServiceRouter(), nil) // Nil is fine here as long as we never execute the proposal's Msgs.

	return govKeeper, m, encCfg, ctx
}

// trackMockBalances sets up expected calls on the Mock BankKeeper, and also
// locally tracks accounts balances (not modules balances).
func trackMockBalances(bankKeeper *govtestutil.MockBankKeeper) error {
	addressCdc := codectestutil.CodecOptions{}.GetAddressCodec()
	poolAcctStr, err := addressCdc.BytesToString(poolAcct)
	if err != nil {
		return err
	}
	balances := make(map[string]sdk.Coins)
	balances[poolAcctStr] = sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(0)))

	// We don't track module account balances.
	bankKeeper.EXPECT().MintCoins(gomock.Any(), mintModuleName, gomock.Any()).AnyTimes()
	bankKeeper.EXPECT().BurnCoins(gomock.Any(), authtypes.NewEmptyModuleAccount(types.ModuleName).GetAddress(), gomock.Any()).AnyTimes()
	bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), mintModuleName, types.ModuleName, gomock.Any()).AnyTimes()

	// But we do track normal account balances.
	bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), types.ModuleName, gomock.Any()).DoAndReturn(func(_ sdk.Context, sender sdk.AccAddress, _ string, coins sdk.Coins) error {
		senderAddr, err := addressCdc.BytesToString(sender)
		if err != nil {
			return err
		}
		newBalance, negative := balances[senderAddr].SafeSub(coins...)
		if negative {
			return errors.New("not enough balance")
		}
		balances[senderAddr] = newBalance
		return nil
	}).AnyTimes()
	bankKeeper.EXPECT().SendCoinsFromModuleToAccount(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).DoAndReturn(func(_ sdk.Context, module string, rcpt sdk.AccAddress, coins sdk.Coins) error {
		rcptAddr, err := addressCdc.BytesToString(rcpt)
		if err != nil {
			return err
		}
		balances[rcptAddr] = balances[rcptAddr].Add(coins...)
		return nil
	}).AnyTimes()
	bankKeeper.EXPECT().GetAllBalances(gomock.Any(), gomock.Any()).DoAndReturn(func(_ sdk.Context, addr sdk.AccAddress) (sdk.Coins, error) {
		addrStr, err := addressCdc.BytesToString(addr)
		if err != nil {
			return sdk.Coins{}, err
		}
		return balances[addrStr], nil
	}).AnyTimes()
	bankKeeper.EXPECT().GetBalance(gomock.Any(), gomock.Any(), sdk.DefaultBondDenom).DoAndReturn(func(_ sdk.Context, addr sdk.AccAddress, _ string) (sdk.Coin, error) {
		addrStr, err := addressCdc.BytesToString(addr)
		if err != nil {
			return sdk.Coin{}, err
		}
		balances := balances[addrStr]
		for _, balance := range balances {
			if balance.Denom == sdk.DefaultBondDenom {
				return balance, nil
			}
		}
		return sdk.NewCoin(sdk.DefaultBondDenom, math.NewInt(0)), nil
	}).AnyTimes()
	return nil
}
