package keeper_test

import (
	"testing"
	"time"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/collections"
	"cosmossdk.io/core/header"
	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/math"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/bank/keeper"
	banktestutil "cosmossdk.io/x/bank/testutil"
	banktypes "cosmossdk.io/x/bank/types"

	"github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

func TestBankStateCompatibility(t *testing.T) {
	key := storetypes.NewKVStoreKey(banktypes.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	ctx := testCtx.Ctx.WithHeaderInfo(header.Info{Time: time.Now()})
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{})

	env := runtime.NewEnvironment(runtime.NewKVStoreService(key), coretesting.NewNopLogger())

	// gomock initializations
	ctrl := gomock.NewController(t)
	authKeeper := banktestutil.NewMockAccountKeeper(ctrl)
	authKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec("cosmos")).AnyTimes()

	ac := codectestutil.CodecOptions{}.GetAddressCodec()
	addr, err := ac.BytesToString(accAddrs[4])
	require.NoError(t, err)
	authority, err := ac.BytesToString(authtypes.NewModuleAddress(banktypes.GovModuleName))
	require.NoError(t, err)

	k := keeper.NewBaseKeeper(
		env,
		encCfg.Codec,
		authKeeper,
		map[string]bool{addr: true},
		authority,
	)

	// test we can decode balances without problems
	// using the old value format of the denom to address index
	bankDenomAddressLegacyIndexValue := []byte{0} // taken from: https://github.com/cosmos/cosmos-sdk/blob/v0.47.3/x/bank/keeper/send.go#L361
	rawKey, err := collections.EncodeKeyWithPrefix(
		banktypes.DenomAddressPrefix,
		k.Balances.Indexes.Denom.KeyCodec(),
		collections.Join("atom", sdk.AccAddress("test")),
	)
	require.NoError(t, err)
	// we set the index key to the old value.
	require.NoError(t, env.KVStoreService.OpenKVStore(ctx).Set(rawKey, bankDenomAddressLegacyIndexValue))

	// test walking is ok
	err = k.Balances.Indexes.Denom.Walk(ctx, nil, func(indexingKey string, indexedKey sdk.AccAddress) (stop bool, err error) {
		require.Equal(t, indexedKey, sdk.AccAddress("test"))
		require.Equal(t, indexingKey, "atom")
		return true, nil
	})
	require.NoError(t, err)

	// test matching is also ok
	iter, err := k.Balances.Indexes.Denom.MatchExact(ctx, "atom")
	require.NoError(t, err)
	defer iter.Close()
	pks, err := iter.PrimaryKeys()
	require.NoError(t, err)
	require.Len(t, pks, 1)
	require.Equal(t, pks[0], collections.Join(sdk.AccAddress("test"), "atom"))

	// assert the index value will be updated to the new format
	err = k.Balances.Indexes.Denom.Reference(ctx, collections.Join(sdk.AccAddress("test"), "atom"), math.ZeroInt(), nil)
	require.NoError(t, err)

	newRawValue, err := env.KVStoreService.OpenKVStore(ctx).Get(rawKey)
	require.NoError(t, err)
	require.Equal(t, []byte{}, newRawValue)
}
