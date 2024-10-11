package keeper_test

import (
	"testing"
	"time"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/core/header"
	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/math"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/distribution"
	"cosmossdk.io/x/distribution/keeper"
	distrtestutil "cosmossdk.io/x/distribution/testutil"
	"cosmossdk.io/x/distribution/types"

	"github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

type dep struct {
	bankKeeper    *distrtestutil.MockBankKeeper
	stakingKeeper *distrtestutil.MockStakingKeeper
	accountKeeper *distrtestutil.MockAccountKeeper
}

func initFixture(t *testing.T) (sdk.Context, []sdk.AccAddress, keeper.Keeper, dep) {
	t.Helper()

	ctrl := gomock.NewController(t)
	key := storetypes.NewKVStoreKey(types.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	cdcOpts := codectestutil.CodecOptions{}
	encCfg := moduletestutil.MakeTestEncodingConfig(cdcOpts, distribution.AppModule{})
	ctx := testCtx.Ctx.WithHeaderInfo(header.Info{Time: time.Now()})
	addrs := simtestutil.CreateIncrementalAccounts(2)

	bankKeeper := distrtestutil.NewMockBankKeeper(ctrl)
	stakingKeeper := distrtestutil.NewMockStakingKeeper(ctrl)
	accountKeeper := distrtestutil.NewMockAccountKeeper(ctrl)

	accountKeeper.EXPECT().GetModuleAddress("distribution").Return(distrAcc.GetAddress())
	accountKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec("cosmos")).AnyTimes()

	stakingKeeper.EXPECT().ValidatorAddressCodec().Return(address.NewBech32Codec("cosmosvaloper")).AnyTimes()

	withdrawAddr := addrs[1]
	bankKeeper.EXPECT().BlockedAddr(withdrawAddr).Return(false).AnyTimes()
	bankKeeper.EXPECT().BlockedAddr(distrAcc.GetAddress()).Return(true).AnyTimes()

	env := runtime.NewEnvironment(runtime.NewKVStoreService(key), coretesting.NewNopLogger())

	authorityAddr, err := cdcOpts.GetAddressCodec().BytesToString(authtypes.NewModuleAddress("gov"))
	require.NoError(t, err)

	distrKeeper := keeper.NewKeeper(
		encCfg.Codec,
		env,
		accountKeeper,
		bankKeeper,
		stakingKeeper,
		testCometService,
		"fee_collector",
		authorityAddr,
	)

	params := types.DefaultParams()
	require.NoError(t, distrKeeper.Params.Set(ctx, params))

	return ctx, addrs, distrKeeper, dep{bankKeeper, stakingKeeper, accountKeeper}
}

func TestSetWithdrawAddr(t *testing.T) {
	ctx, addrs, distrKeeper, _ := initFixture(t)

	params := types.DefaultParams()
	params.WithdrawAddrEnabled = false
	require.NoError(t, distrKeeper.Params.Set(ctx, params))

	delegatorAddr := addrs[0]
	withdrawAddr := addrs[1]

	err := distrKeeper.SetWithdrawAddr(ctx, delegatorAddr, withdrawAddr)
	require.NotNil(t, err)

	params.WithdrawAddrEnabled = true
	require.NoError(t, distrKeeper.Params.Set(ctx, params))

	err = distrKeeper.SetWithdrawAddr(ctx, delegatorAddr, withdrawAddr)
	require.Nil(t, err)
	addr, err := distrKeeper.GetDelegatorWithdrawAddr(ctx, delegatorAddr)
	require.NoError(t, err)
	require.Equal(t, addr, withdrawAddr)

	require.Error(t, distrKeeper.SetWithdrawAddr(ctx, delegatorAddr, distrAcc.GetAddress()))
}

func TestWithdrawValidatorCommission(t *testing.T) {
	ctx, addrs, distrKeeper, dep := initFixture(t)

	valAddr := sdk.ValAddress(addrs[0])
	valCommission := sdk.DecCoins{
		sdk.NewDecCoinFromDec("mytoken", math.LegacyNewDec(5).Quo(math.LegacyNewDec(4))),
		sdk.NewDecCoinFromDec("stake", math.LegacyNewDec(3).Quo(math.LegacyNewDec(2))),
	}

	// set outstanding rewards
	require.NoError(t, distrKeeper.ValidatorOutstandingRewards.Set(ctx, valAddr, types.ValidatorOutstandingRewards{Rewards: valCommission}))

	// set commission
	require.NoError(t, distrKeeper.ValidatorsAccumulatedCommission.Set(ctx, valAddr, types.ValidatorAccumulatedCommission{Commission: valCommission}))

	// withdraw commission
	coins := sdk.NewCoins(sdk.NewCoin("mytoken", math.NewInt(1)), sdk.NewCoin("stake", math.NewInt(1)))
	// if SendCoinsFromModuleToAccount is called, we know that the withdraw was successful
	dep.bankKeeper.EXPECT().SendCoinsFromModuleToAccount(gomock.Any(), "distribution", addrs[0], coins).Return(nil)

	_, err := distrKeeper.WithdrawValidatorCommission(ctx, valAddr)
	require.NoError(t, err)

	// check remainder
	remainderValCommission, err := distrKeeper.ValidatorsAccumulatedCommission.Get(ctx, valAddr)
	require.NoError(t, err)
	remainder := remainderValCommission.Commission
	require.Equal(t, sdk.DecCoins{
		sdk.NewDecCoinFromDec("mytoken", math.LegacyNewDec(1).Quo(math.LegacyNewDec(4))),
		sdk.NewDecCoinFromDec("stake", math.LegacyNewDec(1).Quo(math.LegacyNewDec(2))),
	}, remainder)
}

func TestGetTotalRewards(t *testing.T) {
	ctx, addrs, distrKeeper, _ := initFixture(t)

	valCommission := sdk.DecCoins{
		sdk.NewDecCoinFromDec("mytoken", math.LegacyNewDec(5).Quo(math.LegacyNewDec(4))),
		sdk.NewDecCoinFromDec("stake", math.LegacyNewDec(3).Quo(math.LegacyNewDec(2))),
	}

	require.NoError(t, distrKeeper.ValidatorOutstandingRewards.Set(ctx, sdk.ValAddress(addrs[0]), types.ValidatorOutstandingRewards{Rewards: valCommission}))
	require.NoError(t, distrKeeper.ValidatorOutstandingRewards.Set(ctx, sdk.ValAddress(addrs[1]), types.ValidatorOutstandingRewards{Rewards: valCommission}))

	expectedRewards := valCommission.MulDec(math.LegacyNewDec(2))
	totalRewards := distrKeeper.GetTotalRewards(ctx)

	require.Equal(t, expectedRewards, totalRewards)
}
