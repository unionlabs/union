package runtime_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	bankv1beta1 "cosmossdk.io/api/cosmos/bank/v1beta1"
	coretesting "cosmossdk.io/core/testing"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/baseapp"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	counterkeeper "github.com/cosmos/cosmos-sdk/testutil/x/counter/keeper"
	countertypes "github.com/cosmos/cosmos-sdk/testutil/x/counter/types"
)

func TestRouterService(t *testing.T) {
	interfaceRegistry := codectypes.NewInterfaceRegistry()
	msgRouter := baseapp.NewMsgServiceRouter()
	msgRouter.SetInterfaceRegistry(interfaceRegistry)
	queryRouter := baseapp.NewGRPCQueryRouter()
	queryRouter.SetInterfaceRegistry(interfaceRegistry)
	key := storetypes.NewKVStoreKey(countertypes.StoreKey)
	storeService := runtime.NewKVStoreService(key)
	counterKeeper := counterkeeper.NewKeeper(runtime.NewEnvironment(storeService, coretesting.NewNopLogger()))
	countertypes.RegisterInterfaces(interfaceRegistry)
	countertypes.RegisterMsgServer(msgRouter, counterKeeper)
	countertypes.RegisterQueryServer(queryRouter, counterKeeper)

	messageRouterService := runtime.NewMsgRouterService(msgRouter)
	queryRouterService := runtime.NewQueryRouterService(queryRouter)
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))

	// Messages

	t.Run("invalid msg", func(t *testing.T) {
		_, err := messageRouterService.Invoke(testCtx.Ctx, &bankv1beta1.MsgSend{})
		require.ErrorContains(t, err, "could not find response type for message cosmos.bank.v1beta1.MsgSend")
	})

	t.Run("invoke: valid msg (proto v1)", func(t *testing.T) {
		resp, err := messageRouterService.Invoke(testCtx.Ctx, &countertypes.MsgIncreaseCounter{
			Signer: "cosmos1",
			Count:  42,
		})
		require.NoError(t, err)
		require.NotNil(t, resp)
	})

	// Queries

	t.Run("invalid query", func(t *testing.T) {
		_, err := queryRouterService.Invoke(testCtx.Ctx, &bankv1beta1.QueryBalanceRequest{})
		require.ErrorContains(t, err, "could not find response type for request cosmos.bank.v1beta1.QueryBalanceRequest")
	})

	t.Run("invoke: valid query (proto v1)", func(t *testing.T) {
		_ = counterKeeper.CountStore.Set(testCtx.Ctx, 42)

		resp, err := queryRouterService.Invoke(testCtx.Ctx, &countertypes.QueryGetCountRequest{})
		require.NoError(t, err)
		require.NotNil(t, resp)
		respVal, ok := resp.(*countertypes.QueryGetCountResponse)
		require.True(t, ok)
		require.Equal(t, int64(42), respVal.TotalCount)
	})
}
