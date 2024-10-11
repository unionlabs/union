package module_test

import (
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/core/header"
	coretesting "cosmossdk.io/core/testing"
	sdkmath "cosmossdk.io/math"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/feegrant"
	"cosmossdk.io/x/feegrant/keeper"
	"cosmossdk.io/x/feegrant/module"
	feegranttestutil "cosmossdk.io/x/feegrant/testutil"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

func TestFeegrantPruning(t *testing.T) {
	key := storetypes.NewKVStoreKey(feegrant.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, module.AppModule{})

	addrs := simtestutil.CreateIncrementalAccounts(4)
	granter1 := addrs[0]
	granter2 := addrs[1]
	granter3 := addrs[2]
	grantee := addrs[3]
	spendLimit := sdk.NewCoins(sdk.NewCoin("stake", sdkmath.NewInt(1000)))
	now := testCtx.Ctx.HeaderInfo().Time
	oneDay := now.AddDate(0, 0, 1)

	ctrl := gomock.NewController(t)
	accountKeeper := feegranttestutil.NewMockAccountKeeper(ctrl)
	accountKeeper.EXPECT().GetAccount(gomock.Any(), grantee).Return(authtypes.NewBaseAccountWithAddress(grantee)).AnyTimes()
	accountKeeper.EXPECT().GetAccount(gomock.Any(), granter1).Return(authtypes.NewBaseAccountWithAddress(granter1)).AnyTimes()
	accountKeeper.EXPECT().GetAccount(gomock.Any(), granter2).Return(authtypes.NewBaseAccountWithAddress(granter2)).AnyTimes()
	accountKeeper.EXPECT().GetAccount(gomock.Any(), granter3).Return(authtypes.NewBaseAccountWithAddress(granter3)).AnyTimes()
	ac := address.NewBech32Codec("cosmos")
	accountKeeper.EXPECT().AddressCodec().Return(ac).AnyTimes()

	env := runtime.NewEnvironment(runtime.NewKVStoreService(key), coretesting.NewNopLogger())

	feegrantKeeper := keeper.NewKeeper(env, encCfg.Codec, accountKeeper)

	err := feegrantKeeper.GrantAllowance(
		testCtx.Ctx,
		granter1,
		grantee,
		&feegrant.BasicAllowance{
			Expiration: &now,
		},
	)
	require.NoError(t, err)

	err = feegrantKeeper.GrantAllowance(
		testCtx.Ctx,
		granter2,
		grantee,
		&feegrant.BasicAllowance{
			SpendLimit: spendLimit,
		},
	)
	require.NoError(t, err)

	err = feegrantKeeper.GrantAllowance(
		testCtx.Ctx,
		granter3,
		grantee,
		&feegrant.BasicAllowance{
			Expiration: &oneDay,
		},
	)
	require.NoError(t, err)

	queryHelper := baseapp.NewQueryServerTestHelper(testCtx.Ctx, encCfg.InterfaceRegistry)
	feegrant.RegisterQueryServer(queryHelper, feegrantKeeper)
	queryClient := feegrant.NewQueryClient(queryHelper)

	granteeStr, err := ac.BytesToString(grantee)
	require.NoError(t, err)
	queryRequest := &feegrant.QueryAllowancesRequest{
		Grantee: granteeStr,
	}

	res, err := queryClient.Allowances(testCtx.Ctx.Context(), queryRequest)
	require.NoError(t, err)
	require.NotNil(t, res)
	require.Len(t, res.Allowances, 3)

	require.NoError(t, module.EndBlocker(testCtx.Ctx, feegrantKeeper))

	res, err = queryClient.Allowances(testCtx.Ctx.Context(), queryRequest)
	require.NoError(t, err)
	require.NotNil(t, res)
	require.Len(t, res.Allowances, 2)

	testCtx.Ctx = testCtx.Ctx.WithHeaderInfo(header.Info{Time: now.AddDate(0, 0, 2)})
	require.NoError(t, module.EndBlocker(testCtx.Ctx, feegrantKeeper))

	res, err = queryClient.Allowances(testCtx.Ctx.Context(), queryRequest)
	require.NoError(t, err)
	require.NotNil(t, res)
	require.Len(t, res.Allowances, 1)
}
