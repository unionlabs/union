package cometbls

import (
	"testing"
)

// 	cosmosstore "cosmossdk.io/store"
// 	storetypes "cosmossdk.io/store/types"
// 	cmtproto "github.com/cometbft/cometbft/proto/tendermint/types"
// 	"github.com/cosmos/cosmos-sdk/codec"
// 	sdk "github.com/cosmos/cosmos-sdk/types"

// 	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
// 	"github.com/stretchr/testify/require"
// 	"github.com/tendermint/tendermint/libs/db"
// 	"github.com/tendermint/tendermint/libs/log"
// )

// func createTestKeeper() (sdk.Context, Keeper) {
// 	// Initialize in-memory database and context
// 	db := db.NewMemDB()
// 	// ms := store.NewCommitMultiStore(db)
// 	ms := cosmosstore.NewCommitMultiStore(db)
// 	key := storetypes.NewKVStoreKey(StoreKey)

// 	ms.MountStoreWithDB(key, storetypes.StoreTypeIAVL, db)
// 	ms.LoadLatestVersion()

// 	registry := codectypes.NewInterfaceRegistry()
// 	cdc := codec.NewProtoCodec(registry)

// 	ctx := sdk.NewContext(ms, cmtproto.Header{}, false, log.NewNopLogger())
// 	authority := "cosmos1h5p39slkj9gkye32ccu0dzgn9lcyffwnfk88pt" // Use an appropriate mock address

// 	// Initialize the Keeper
// 	keeper := NewKeeper(cdc, key, authority)

// 	return ctx, keeper
// }

// func TestSetAndGetVerifyingKey(t *testing.T) {
// 	ctx, keeper := createTestKeeper()

// 	// Setup a sender that matches the authority
// 	sender, _ := sdk.AccAddressFromBech32("cosmos1h5p39slkj9gkye32ccu0dzgn9lcyffwnfk88pt")

// 	// Verifying key to set
// 	vkHex := "8967072901cc7ab63357f1ddc4196c7c1feda50540d8026d7f6f0167c118a899d923def15f75234f2a6d53b566a2528441e98050b38803673e9179b834fc39a499355fd270b7601d5d88408b7e9e53d260512e2180cd260017dc941f2fc96d65153f0344c6bf2d8a891b979bc61d39a98fb11155fcd57418f30ea018ea842874a0e76be91a3148e2f8ef644222b3ce5b939a73bd2e0a40814f7f92a79c483acf2216bbe0c289e07936b4d9653b91521a24c570c808fa46dfd12ec4429e71b61999fcfb245459d63a4923b8f8c488d1e6af7ca358867b88eb0cdefe896c221f09e95e4c18d1e0475de4549b2547611d8301e1afff1047a6f5a288c9314af0b9fc05d403c8c91820a385a72c18d6a4962cef41a3ab93daa7ed289b1e95db4d04eb00000003e71843e52743864f4bb67ce94a2ce8fe82c8f61042c4c1ced8531d94305392818b0dbe71f4d60e02e9160ec2b015cae3a09cbe4f437226e2c02e1a5e5d124bcac29e93d5f47c0c7671350398ed8c40f5bc5c2f5b00363c7e2eb18a91a1c490c70000000100000000a57df6f8132cb0037f7dfdf1a29b04c1ff92ba082eda513996ba2bfa9fbd198713f0d8d8879885ca567ef99298c30c397e6fba584658f4127713a814c06de55aefbfe141a7555cf7e3e86b092660b81cfb68a025ad817e45cec0b0f2e2ca636802a104df1c015f2307fa2859627098cdf9fdb521d61d323943343a12304e5baf"

// 	// Set the verifying key
// 	err := keeper.SetVerifyingKey(ctx, sender, vkHex)
// 	require.NoError(t, err)

// 	// Retrieve the verifying key and check if it matches
// 	retrievedVkHex, err := keeper.GetVerifyingKey(ctx)
// 	require.NoError(t, err)
// 	require.Equal(t, vkHex, retrievedVkHex)
// }

// func TestUnauthorizedSetVerifyingKey(t *testing.T) {
// 	ctx, keeper := createTestKeeper()

// 	// Setup a sender that is NOT the authority
// 	sender, _ := sdk.AccAddressFromBech32("cosmos1xyz9gkye32ccu0dzgn9lcyffwnfk88pt")

// 	// Verifying key to set
// 	vkHex := "8967072901cc7ab63357f1ddc4196c7c1feda50540d8026d7f6f0167c118a899d923def15f75234f2a6d53b566a2528441e98050b38803673e9179b834fc39a499355fd270b7601d5d88408b7e9e53d260512e2180cd260017dc941f2fc96d65153f0344c6bf2d8a891b979bc61d39a98fb11155fcd57418f30ea018ea842874a0e76be91a3148e2f8ef644222b3ce5b939a73bd2e0a40814f7f92a79c483acf2216bbe0c289e07936b4d9653b91521a24c570c808fa46dfd12ec4429e71b61999fcfb245459d63a4923b8f8c488d1e6af7ca358867b88eb0cdefe896c221f09e95e4c18d1e0475de4549b2547611d8301e1afff1047a6f5a288c9314af0b9fc05d403c8c91820a385a72c18d6a4962cef41a3ab93daa7ed289b1e95db4d04eb00000003e71843e52743864f4bb67ce94a2ce8fe82c8f61042c4c1ced8531d94305392818b0dbe71f4d60e02e9160ec2b015cae3a09cbe4f437226e2c02e1a5e5d124bcac29e93d5f47c0c7671350398ed8c40f5bc5c2f5b00363c7e2eb18a91a1c490c70000000100000000a57df6f8132cb0037f7dfdf1a29b04c1ff92ba082eda513996ba2bfa9fbd198713f0d8d8879885ca567ef99298c30c397e6fba584658f4127713a814c06de55aefbfe141a7555cf7e3e86b092660b81cfb68a025ad817e45cec0b0f2e2ca636802a104df1c015f2307fa2859627098cdf9fdb521d61d323943343a12304e5baf"

// 	// Try to set the verifying key as an unauthorized user
// 	err := keeper.SetVerifyingKey(ctx, sender, vkHex)
// 	require.Error(t, err)
// }
// func TestGetVerifyingKey_NotSet(t *testing.T) {
// 	ctx, keeper := createTestKeeper()

// 	// Try to get the verifying key before it's set
// 	vkHex, err := keeper.GetVerifyingKey(ctx)
// 	require.Error(t, err)
// 	require.Equal(t, "", vkHex)
// }
// func TestSetAndInitVerifyingKey(t *testing.T) {
// 	ctx, keeper := createTestKeeper()

// 	// Setup a sender that matches the authority
// 	sender, _ := sdk.AccAddressFromBech32("cosmos1h5p39slkj9gkye32ccu0dzgn9lcyffwnfk88pt")

// 	// Verifying key to set
// 	vkHex := "abcd1234"

// 	// Set the verifying key
// 	err := keeper.SetVerifyingKey(ctx, sender, vkHex)
// 	require.NoError(t, err)

// 	// Call InitVerifyingKey (should not overwrite the existing key)
// 	keeper.InitVerifyingKey(ctx)

// 	// Retrieve the verifying key and check if it matches the one set earlier
// 	retrievedVkHex, err := keeper.GetVerifyingKey(ctx)
// 	require.NoError(t, err)
// 	require.Equal(t, vkHex, retrievedVkHex)
// }

func TestVerifier(t *testing.T) {
	// rawZKP, _ := hex.DecodeString("294A48A750D5C2CF926516752FF484EEBE55FF26CF8A8A7536D98794CF062DB6214D0C9E5C6B164111927A1630889619DBBB40149D8E2D32898E7ACB765542CD0EB8A8E04CCC254C3BFDC2FCE627D59C3C05E2AC76E03977855DD889C1C9BA432FF7FF4DEFCB5286555D36D22DD073A859140508AF9B977F38EB9A604E99A5F6109D43A4AFA0AB161DA2B261DED80FBC0C36E57DE2001338941C834E3262CF751BC1BFC6EC27BB8E106BAAB976285BAC1D4AC38D1B759C8A2852D65CE239974F1275CC6765B3D174FD1122EFDE86137D19F07483FEF5244B1D74B2D9DC598AC32A5CA10E8837FBC89703F4D0D46912CF4AF82341C30C2A1F3941849CC011A56E18AD2162EEB71289B8821CC01875BC1E35E5FC1EBD9114C0B2C0F0D9A96C394001468C70A1716CA98EBE82B1E614D4D9B07292EBAD5B60E0C76FD1D58B485E7D1FB1E07F51A0C68E4CA59A399FCF0634D9585BE478E37480423681B984E96C0A1698D8FCB1DF51CAE023B045E114EED9CB233A5742D9E60E1097206EB20A5058")

	// zkp, err := ParseZKP(rawZKP)

	// assert.NoError(t, err)

	// trustedValHash, _ := hex.DecodeString("1B7EA0F1B3E574F8D50A12827CCEA43CFF858C2716AE05370CC40AE8EC521FD8")
	// nextValHash, _ := hex.DecodeString("1B7EA0F1B3E574F8D50A12827CCEA43CFF858C2716AE05370CC40AE8EC521FD8")
	// valHash, _ := hex.DecodeString("1B7EA0F1B3E574F8D50A12827CCEA43CFF858C2716AE05370CC40AE8EC521FD8")
	// appHash, _ := hex.DecodeString("3A34FC963EEFAAE9B7C0D3DFF89180D91F3E31073E654F732340CEEDD77DD25B")
	// err = zkp.Verify(

	// 	trustedValHash,
	// 	LightHeader{
	// 		ChainId:            "union-devnet-1337",
	// 		Height:             3405691582,
	// 		Time:               time.Unix(1710783278, 499600406),
	// 		ValidatorsHash:     valHash,
	// 		NextValidatorsHash: nextValHash,
	// 		AppHash:            appHash,
	// 	},
	// )

	// assert.NoError(t, err)
}
