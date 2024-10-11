package v5_test

import (
	"math"
	"math/rand"
	"testing"
	"time"

	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	coretesting "cosmossdk.io/core/testing"
	sdkmath "cosmossdk.io/math"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/staking"
	v5 "cosmossdk.io/x/staking/migrations/v5"
	stakingtypes "cosmossdk.io/x/staking/types"

	"github.com/cosmos/cosmos-sdk/codec"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
)

func TestHistoricalKeysMigration(t *testing.T) {
	storeKey := storetypes.NewKVStoreKey("staking")
	tKey := storetypes.NewTransientStoreKey("transient_test")
	ctx := testutil.DefaultContext(storeKey, tKey)
	store := ctx.KVStore(storeKey)
	logger := coretesting.NewNopLogger()

	type testCase struct {
		oldKey, newKey, historicalInfo []byte
	}

	testCases := make(map[int64]testCase)

	// edge cases
	testCases[0], testCases[1], testCases[math.MaxInt32] = testCase{}, testCase{}, testCase{}

	// random cases
	seed := time.Now().UnixNano()
	r := rand.New(rand.NewSource(seed))
	for i := 0; i < 10; i++ {
		height := r.Intn(math.MaxInt32-2) + 2

		testCases[int64(height)] = testCase{}
	}

	cdc := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}).Codec
	for height := range testCases {
		testCases[height] = testCase{
			oldKey:         v5.GetLegacyHistoricalInfoKey(height),
			newKey:         v5.GetHistoricalInfoKey(height),
			historicalInfo: cdc.MustMarshal(createHistoricalInfo(height, "testChainID")),
		}
	}

	// populate store using old key format
	for _, tc := range testCases {
		store.Set(tc.oldKey, tc.historicalInfo)
	}

	// migrate store to new key format
	require.NoErrorf(t, v5.MigrateStore(ctx, store, cdc, logger), "v5.MigrateStore failed, seed: %d", seed)

	// check results
	for _, tc := range testCases {
		require.Nilf(t, store.Get(tc.oldKey), "old key should be deleted, seed: %d", seed)
		require.NotNilf(t, store.Get(tc.newKey), "new key should be created, seed: %d", seed)
		require.Equalf(t, tc.historicalInfo, store.Get(tc.newKey), "seed: %d", seed)
	}
}

func createHistoricalInfo(height int64, chainID string) *stakingtypes.HistoricalInfo {
	return &stakingtypes.HistoricalInfo{Header: cmtproto.Header{ChainID: chainID, Height: height}}
}

func TestDelegationsByValidatorMigrations(t *testing.T) {
	codecOpts := codectestutil.CodecOptions{}
	cdc := moduletestutil.MakeTestEncodingConfig(codecOpts, staking.AppModule{}).Codec
	storeKey := storetypes.NewKVStoreKey(v5.ModuleName)
	tKey := storetypes.NewTransientStoreKey("transient_test")
	ctx := testutil.DefaultContext(storeKey, tKey)
	store := ctx.KVStore(storeKey)
	logger := coretesting.NewNopLogger()

	accAddrs := sims.CreateIncrementalAccounts(11)
	valAddrs := sims.ConvertAddrsToValAddrs(accAddrs[0:1])
	var addedDels []stakingtypes.Delegation

	valAddr, err := codecOpts.GetValidatorCodec().BytesToString(valAddrs[0])
	assert.NoError(t, err)

	for i := 1; i < 11; i++ {
		accAddr, err := codecOpts.GetAddressCodec().BytesToString(accAddrs[i])
		assert.NoError(t, err)
		del1 := stakingtypes.NewDelegation(accAddr, valAddr, sdkmath.LegacyNewDec(100))
		store.Set(v5.GetDelegationKey(accAddrs[i], valAddrs[0]), stakingtypes.MustMarshalDelegation(cdc, del1))
		addedDels = append(addedDels, del1)
	}

	// before migration the state of delegations by val index should be empty
	dels := getValDelegations(ctx, cdc, storeKey, valAddrs[0])
	assert.Len(t, dels, 0)

	err = v5.MigrateStore(ctx, store, cdc, logger)
	assert.NoError(t, err)

	// after migration the state of delegations by val index should not be empty
	dels = getValDelegations(ctx, cdc, storeKey, valAddrs[0])
	assert.Len(t, dels, len(addedDels))
	assert.Equal(t, addedDels, dels)
}

func getValDelegations(ctx sdk.Context, cdc codec.Codec, storeKey storetypes.StoreKey, valAddr sdk.ValAddress) []stakingtypes.Delegation {
	var delegations []stakingtypes.Delegation

	store := ctx.KVStore(storeKey)
	iterator := storetypes.KVStorePrefixIterator(store, v5.GetDelegationsByValPrefixKey(valAddr))
	for ; iterator.Valid(); iterator.Next() {
		var delegation stakingtypes.Delegation
		valAddr, delAddr, err := v5.ParseDelegationsByValKey(iterator.Key())
		if err != nil {
			panic(err)
		}

		bz := store.Get(v5.GetDelegationKey(delAddr, valAddr))

		cdc.MustUnmarshal(bz, &delegation)

		delegations = append(delegations, delegation)
	}

	return delegations
}
