package authz_test

import (
	"testing"
	"time"

	"github.com/cometbft/cometbft/libs/log"
	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/testutil"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	"github.com/cosmos/cosmos-sdk/x/authz"
	"github.com/cosmos/cosmos-sdk/x/authz/keeper"
	authzmodule "github.com/cosmos/cosmos-sdk/x/authz/module"
	authztestutil "github.com/cosmos/cosmos-sdk/x/authz/testutil"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/proto/tendermint/types"
)

func TestExpiredGrantsQueue(t *testing.T) {
	key := sdk.NewKVStoreKey(keeper.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, sdk.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(authzmodule.AppModuleBasic{})
	ctx := testCtx.Ctx.WithBlockHeader(types.Header{})

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
	expiration := ctx.BlockTime().AddDate(0, 1, 0)
	expiration2 := expiration.AddDate(1, 0, 0)
	smallCoins := sdk.NewCoins(sdk.NewInt64Coin("stake", 10))
	sendAuthz := banktypes.NewSendAuthorization(smallCoins, nil)

	ctrl := gomock.NewController(t)
	accountKeeper := authztestutil.NewMockAccountKeeper(ctrl)
	accountKeeper.EXPECT().GetAccount(gomock.Any(), granter).Return(authtypes.NewBaseAccountWithAddress(granter)).AnyTimes()
	accountKeeper.EXPECT().GetAccount(gomock.Any(), grantee1).Return(authtypes.NewBaseAccountWithAddress(grantee1)).AnyTimes()
	accountKeeper.EXPECT().GetAccount(gomock.Any(), grantee2).Return(authtypes.NewBaseAccountWithAddress(grantee2)).AnyTimes()
	accountKeeper.EXPECT().GetAccount(gomock.Any(), grantee3).Return(authtypes.NewBaseAccountWithAddress(grantee3)).AnyTimes()
	accountKeeper.EXPECT().GetAccount(gomock.Any(), grantee4).Return(authtypes.NewBaseAccountWithAddress(grantee4)).AnyTimes()

	authzKeeper := keeper.NewKeeper(key, encCfg.Codec, baseApp.MsgServiceRouter(), accountKeeper)

	save := func(grantee sdk.AccAddress, exp *time.Time) {
		err := authzKeeper.SaveGrant(ctx, grantee, granter, sendAuthz, exp)
		require.NoError(t, err, "Grant from %s", grantee.String())
	}
	save(grantee1, &expiration)
	save(grantee2, &expiration)
	save(grantee3, &expiration2)
	save(grantee4, nil)

	queryHelper := baseapp.NewQueryServerTestHelper(ctx, encCfg.InterfaceRegistry)
	authz.RegisterQueryServer(queryHelper, authzKeeper)
	queryClient := authz.NewQueryClient(queryHelper)

	checkGrants := func(ctx sdk.Context, expectedNum int) {
		authzmodule.BeginBlocker(ctx, authzKeeper)

		res, err := queryClient.GranterGrants(ctx.Context(), &authz.QueryGranterGrantsRequest{
			Granter: granter.String(),
		})
		require.NoError(t, err)
		require.NotNil(t, res)
		require.Equal(t, expectedNum, len(res.Grants))
	}

	checkGrants(ctx, 4)

	// expiration is exclusive!
	ctx = ctx.WithBlockTime(expiration)
	checkGrants(ctx, 4)

	ctx = ctx.WithBlockTime(expiration.AddDate(0, 0, 1))
	checkGrants(ctx, 2)

	ctx = ctx.WithBlockTime(expiration2.AddDate(0, 0, 1))
	checkGrants(ctx, 1)
}
