package keeper_test

import (
	"fmt"
	"testing"

	"cosmossdk.io/math"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	tmtime "github.com/cometbft/cometbft/types/time"
	"github.com/golang/mock/gomock"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	"github.com/cosmos/cosmos-sdk/x/gov/keeper"
	govtestutil "github.com/cosmos/cosmos-sdk/x/gov/testutil"
	"github.com/cosmos/cosmos-sdk/x/gov/types"
	v1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1"
	"github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"
	minttypes "github.com/cosmos/cosmos-sdk/x/mint/types"
)

var (
	_, _, addr   = testdata.KeyTestPubAddr()
	govAcct      = authtypes.NewModuleAddress(types.ModuleName)
	TestProposal = getTestProposal()
)

// getTestProposal creates and returns a test proposal message.
func getTestProposal() []sdk.Msg {
	legacyProposalMsg, err := v1.NewLegacyContent(v1beta1.NewTextProposal("Title", "description"), authtypes.NewModuleAddress(types.ModuleName).String())
	if err != nil {
		panic(err)
	}

	return []sdk.Msg{
		banktypes.NewMsgSend(govAcct, addr, sdk.NewCoins(sdk.NewCoin("stake", sdk.NewInt(1000)))),
		legacyProposalMsg,
	}
}

// setupGovKeeper creates a govKeeper as well as all its dependencies.
func setupGovKeeper(t *testing.T) (
	*keeper.Keeper,
	*govtestutil.MockAccountKeeper,
	*govtestutil.MockBankKeeper,
	*govtestutil.MockStakingKeeper,
	moduletestutil.TestEncodingConfig,
	sdk.Context,
) {
	key := sdk.NewKVStoreKey(types.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, sdk.NewTransientStoreKey("transient_test"))
	ctx := testCtx.Ctx.WithBlockHeader(tmproto.Header{Time: tmtime.Now()})
	encCfg := moduletestutil.MakeTestEncodingConfig()
	v1.RegisterInterfaces(encCfg.InterfaceRegistry)
	v1beta1.RegisterInterfaces(encCfg.InterfaceRegistry)
	banktypes.RegisterInterfaces(encCfg.InterfaceRegistry)

	// Create MsgServiceRouter, but don't populate it before creating the gov
	// keeper.
	msr := baseapp.NewMsgServiceRouter()

	// gomock initializations
	ctrl := gomock.NewController(t)
	acctKeeper := govtestutil.NewMockAccountKeeper(ctrl)
	bankKeeper := govtestutil.NewMockBankKeeper(ctrl)
	stakingKeeper := govtestutil.NewMockStakingKeeper(ctrl)
	acctKeeper.EXPECT().GetModuleAddress(types.ModuleName).Return(govAcct).AnyTimes()
	acctKeeper.EXPECT().GetModuleAccount(gomock.Any(), types.ModuleName).Return(authtypes.NewEmptyModuleAccount(types.ModuleName)).AnyTimes()
	trackMockBalances(bankKeeper)
	stakingKeeper.EXPECT().TokensFromConsensusPower(ctx, gomock.Any()).DoAndReturn(func(ctx sdk.Context, power int64) math.Int {
		return sdk.TokensFromConsensusPower(power, math.NewIntFromUint64(1000000))
	}).AnyTimes()
	stakingKeeper.EXPECT().BondDenom(ctx).Return("stake").AnyTimes()
	stakingKeeper.EXPECT().IterateBondedValidatorsByPower(gomock.Any(), gomock.Any()).AnyTimes()
	stakingKeeper.EXPECT().IterateDelegations(gomock.Any(), gomock.Any(), gomock.Any()).AnyTimes()
	stakingKeeper.EXPECT().TotalBondedTokens(gomock.Any()).Return(math.NewInt(10000000)).AnyTimes()

	// Gov keeper initializations
	govKeeper := keeper.NewKeeper(encCfg.Codec, key, acctKeeper, bankKeeper, stakingKeeper, msr, types.DefaultConfig(), govAcct.String())
	govKeeper.SetProposalID(ctx, 1)
	govRouter := v1beta1.NewRouter() // Also register legacy gov handlers to test them too.
	govRouter.AddRoute(types.RouterKey, v1beta1.ProposalHandler)
	govKeeper.SetLegacyRouter(govRouter)
	govKeeper.SetParams(ctx, v1.DefaultParams())

	// Register all handlers for the MegServiceRouter.
	msr.SetInterfaceRegistry(encCfg.InterfaceRegistry)
	v1.RegisterMsgServer(msr, keeper.NewMsgServerImpl(govKeeper))
	banktypes.RegisterMsgServer(msr, nil) // Nil is fine here as long as we never execute the proposal's Msgs.

	return govKeeper, acctKeeper, bankKeeper, stakingKeeper, encCfg, ctx
}

// trackMockBalances sets up expected calls on the Mock BankKeeper, and also
// locally tracks accounts balances (not modules balances).
func trackMockBalances(bankKeeper *govtestutil.MockBankKeeper) {
	balances := make(map[string]sdk.Coins)

	// We don't track module account balances.
	bankKeeper.EXPECT().MintCoins(gomock.Any(), minttypes.ModuleName, gomock.Any()).AnyTimes()
	bankKeeper.EXPECT().BurnCoins(gomock.Any(), types.ModuleName, gomock.Any()).AnyTimes()
	bankKeeper.EXPECT().SendCoinsFromModuleToModule(gomock.Any(), minttypes.ModuleName, types.ModuleName, gomock.Any()).AnyTimes()

	// But we do track normal account balances.
	bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), types.ModuleName, gomock.Any()).DoAndReturn(func(_ sdk.Context, sender sdk.AccAddress, _ string, coins sdk.Coins) error {
		newBalance, negative := balances[sender.String()].SafeSub(coins...)
		if negative {
			return fmt.Errorf("not enough balance")
		}
		balances[sender.String()] = newBalance
		return nil
	}).AnyTimes()
	bankKeeper.EXPECT().SendCoinsFromModuleToAccount(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).DoAndReturn(func(_ sdk.Context, module string, rcpt sdk.AccAddress, coins sdk.Coins) error {
		balances[rcpt.String()] = balances[rcpt.String()].Add(coins...)
		return nil
	}).AnyTimes()
	bankKeeper.EXPECT().GetAllBalances(gomock.Any(), gomock.Any()).DoAndReturn(func(_ sdk.Context, addr sdk.AccAddress) sdk.Coins {
		return balances[addr.String()]
	}).AnyTimes()
}
