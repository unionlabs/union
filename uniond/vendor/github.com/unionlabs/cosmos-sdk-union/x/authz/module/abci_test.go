package module_test

import (
	"testing"
	"time"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/core/header"
	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/log"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/authz"
	"cosmossdk.io/x/authz/keeper"
	authzmodule "cosmossdk.io/x/authz/module"
	authztestutil "cosmossdk.io/x/authz/testutil"
	banktypes "cosmossdk.io/x/bank/types"

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

func TestExpiredGrantsQueue(t *testing.T) {
	key := storetypes.NewKVStoreKey(keeper.StoreKey)
	storeService := runtime.NewKVStoreService(key)
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, authzmodule.AppModule{})
	ctx := testCtx.Ctx

	baseApp := baseapp.NewBaseApp(
		"authz",
		log.NewNopLogger(),
		testCtx.DB,
		encCfg.TxConfig.TxDecoder(),
	)
	baseApp.SetCMS(testCtx.CMS)
	baseApp.SetInterfaceRegistry(encCfg.InterfaceRegistry)

	banktypes.RegisterInterfaces(encCfg.InterfaceRegistry)

	addrs := simtestutil.CreateIncrementalAccounts(5)
	granter := addrs[0]
	grantee1 := addrs[1]
	grantee2 := addrs[2]
	grantee3 := addrs[3]
	grantee4 := addrs[4]
	expiration := ctx.HeaderInfo().Time.AddDate(0, 1, 0)
	expiration2 := expiration.AddDate(1, 0, 0)
	smallCoins := sdk.NewCoins(sdk.NewInt64Coin("stake", 10))
	sendAuthz := banktypes.NewSendAuthorization(smallCoins, nil, codectestutil.CodecOptions{}.GetAddressCodec())

	ctrl := gomock.NewController(t)
	accountKeeper := authztestutil.NewMockAccountKeeper(ctrl)
	accountKeeper.EXPECT().GetAccount(gomock.Any(), granter).Return(authtypes.NewBaseAccountWithAddress(granter)).AnyTimes()
	accountKeeper.EXPECT().GetAccount(gomock.Any(), grantee1).Return(authtypes.NewBaseAccountWithAddress(grantee1)).AnyTimes()
	accountKeeper.EXPECT().GetAccount(gomock.Any(), grantee2).Return(authtypes.NewBaseAccountWithAddress(grantee2)).AnyTimes()
	accountKeeper.EXPECT().GetAccount(gomock.Any(), grantee3).Return(authtypes.NewBaseAccountWithAddress(grantee3)).AnyTimes()
	accountKeeper.EXPECT().GetAccount(gomock.Any(), grantee4).Return(authtypes.NewBaseAccountWithAddress(grantee4)).AnyTimes()

	accountKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec("cosmos")).AnyTimes()

	env := runtime.NewEnvironment(storeService, coretesting.NewNopLogger(), runtime.EnvWithQueryRouterService(baseApp.GRPCQueryRouter()), runtime.EnvWithMsgRouterService(baseApp.MsgServiceRouter()))
	authzKeeper := keeper.NewKeeper(env, encCfg.Codec, accountKeeper)

	save := func(grantee sdk.AccAddress, exp *time.Time) {
		err := authzKeeper.SaveGrant(ctx, grantee, granter, sendAuthz, exp)
		addr, _ := accountKeeper.AddressCodec().BytesToString(grantee)
		require.NoError(t, err, "Grant from %s", addr)
	}
	save(grantee1, &expiration)
	save(grantee2, &expiration)
	save(grantee3, &expiration2)
	save(grantee4, nil)

	queryHelper := baseapp.NewQueryServerTestHelper(ctx, encCfg.InterfaceRegistry)
	authz.RegisterQueryServer(queryHelper, authzKeeper)
	queryClient := authz.NewQueryClient(queryHelper)

	checkGrants := func(ctx sdk.Context, expectedNum int) {
		err := authzmodule.BeginBlocker(ctx, authzKeeper)
		require.NoError(t, err)

		addr, err := accountKeeper.AddressCodec().BytesToString(granter)
		require.NoError(t, err)
		res, err := queryClient.GranterGrants(ctx.Context(), &authz.QueryGranterGrantsRequest{
			Granter: addr,
		})
		require.NoError(t, err)
		require.NotNil(t, res)
		require.Equal(t, expectedNum, len(res.Grants))
	}

	checkGrants(ctx, 4)

	// expiration is exclusive!
	ctx = ctx.WithHeaderInfo(header.Info{Time: expiration})
	checkGrants(ctx, 4)

	ctx = ctx.WithHeaderInfo(header.Info{Time: expiration.AddDate(0, 0, 1)})
	checkGrants(ctx, 2)

	ctx = ctx.WithHeaderInfo(header.Info{Time: expiration2.AddDate(0, 0, 1)})
	checkGrants(ctx, 1)
}
