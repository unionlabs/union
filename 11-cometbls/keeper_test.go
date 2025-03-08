package cometbls

import (
	"bytes"
	"encoding/hex"
	"testing"

	"cosmossdk.io/log"
	"cosmossdk.io/store"
	"cosmossdk.io/store/metrics"
	storetypes "cosmossdk.io/store/types"
	cmtproto "github.com/cometbft/cometbft/proto/tendermint/types"
	dbm "github.com/cosmos/cosmos-db"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	"github.com/stretchr/testify/require"
)

var (
	verifyingKeyHex = "8967072901cc7ab63357f1ddc4196c7c1feda50540d8026d7f6f0167c118a899d923def15f75234f2a6d53b566a2528441e98050b38803673e9179b834fc39a499355fd270b7601d5d88408b7e9e53d260512e2180cd260017dc941f2fc96d65153f0344c6bf2d8a891b979bc61d39a98fb11155fcd57418f30ea018ea842874a0e76be91a3148e2f8ef644222b3ce5b939a73bd2e0a40814f7f92a79c483acf2216bbe0c289e07936b4d9653b91521a24c570c808fa46dfd12ec4429e71b61999fcfb245459d63a4923b8f8c488d1e6af7ca358867b88eb0cdefe896c221f09e95e4c18d1e0475de4549b2547611d8301e1afff1047a6f5a288c9314af0b9fc05d403c8c91820a385a72c18d6a4962cef41a3ab93daa7ed289b1e95db4d04eb00000003e71843e52743864f4bb67ce94a2ce8fe82c8f61042c4c1ced8531d94305392818b0dbe71f4d60e02e9160ec2b015cae3a09cbe4f437226e2c02e1a5e5d124bcac29e93d5f47c0c7671350398ed8c40f5bc5c2f5b00363c7e2eb18a91a1c490c70000000100000000a57df6f8132cb0037f7dfdf1a29b04c1ff92ba082eda513996ba2bfa9fbd198713f0d8d8879885ca567ef99298c30c397e6fba584658f4127713a814c06de55aefbfe141a7555cf7e3e86b092660b81cfb68a025ad817e45cec0b0f2e2ca636802a104df1c015f2307fa2859627098cdf9fdb521d61d323943343a12304e5baf"
)

func setupKeeper(t *testing.T) (sdk.Context, *Keeper, sdk.AccAddress) {
	key := storetypes.NewKVStoreKey("cometbls")

	db := dbm.NewMemDB()

	cms := store.NewCommitMultiStore(db, log.NewNopLogger(), metrics.NewNoOpMetrics())

	cms.MountStoreWithDB(key, storetypes.StoreTypeIAVL, db)

	err := cms.LoadLatestVersion()
	require.NoError(t, err)

	registry := codectypes.NewInterfaceRegistry()
	registry.RegisterInterface(sdk.MsgInterfaceProtoName, (*sdk.Msg)(nil))
	RegisterInterfaces(registry)
	cdc := codec.NewProtoCodec(registry)

	authority := sdk.AccAddress(address.Module("authority"))
	keeper := NewKeeper(cdc, key, authority.String())

	ctx := sdk.NewContext(cms, cmtproto.Header{}, false, log.NewNopLogger())

	keeper.SetAuthority(ctx, authority)

	return ctx, &keeper, authority
}

func TestSetVerifyingKeyUnauthorized(t *testing.T) {
	ctx, keeper, _ := setupKeeper(t)

	unauthorizedSender := sdk.AccAddress(address.Module("unauthorized"))

	param := NewMsgMyFunction(unauthorizedSender, verifyingKeyHex)
	err := keeper.SetVerifyingKey(ctx, &param)

	require.Error(t, err)
	require.Contains(t, err.Error(), "unauthorized")
}

func TestGetVerifyingKeyNotSet(t *testing.T) {
	ctx, keeper, _ := setupKeeper(t)

	_, err := keeper.GetVerifyingKey(ctx)

	require.Error(t, err)
	require.Contains(t, err.Error(), "verifying key not found")
}

func TestSetAndGetVerifyingKey(t *testing.T) {
	ctx, keeper, authority := setupKeeper(t)

	param := NewMsgMyFunction(authority, verifyingKeyHex)

	err := keeper.SetVerifyingKey(ctx, &param)
	require.NoError(t, err)

	retrievedKey, err := keeper.GetVerifyingKey(ctx)
	require.NoError(t, err)
	var keyBuffer bytes.Buffer
	_, err = retrievedKey.WriteTo(&keyBuffer)
	require.NoError(t, err)

	retrievedKeyHex := hex.EncodeToString(keyBuffer.Bytes())

	require.Equal(t, verifyingKeyHex, retrievedKeyHex)
}
