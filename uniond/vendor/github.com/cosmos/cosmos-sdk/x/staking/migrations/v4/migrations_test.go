package v4_test

import (
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
	storetypes "github.com/cosmos/cosmos-sdk/store/types"
	"github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	paramtypes "github.com/cosmos/cosmos-sdk/x/params/types"
	"github.com/cosmos/cosmos-sdk/x/staking"
	v4 "github.com/cosmos/cosmos-sdk/x/staking/migrations/v4"
	"github.com/cosmos/cosmos-sdk/x/staking/types"
)

type mockSubspace struct {
	ps types.Params
}

func newMockSubspace(ps types.Params) mockSubspace {
	return mockSubspace{ps: ps}
}

func (ms mockSubspace) GetParamSet(ctx sdk.Context, ps paramtypes.ParamSet) {
	*ps.(*types.Params) = ms.ps
}

func TestMigrate(t *testing.T) {
	cdc := moduletestutil.MakeTestEncodingConfig(staking.AppModuleBasic{}).Codec

	storeKey := sdk.NewKVStoreKey(v4.ModuleName)
	tKey := sdk.NewTransientStoreKey("transient_test")
	ctx := testutil.DefaultContext(storeKey, tKey)
	store := ctx.KVStore(storeKey)
	duplicateCreationHeight := int64(1)

	accAddrs := sims.CreateIncrementalAccounts(1)
	accAddr := accAddrs[0]

	valAddrs := sims.ConvertAddrsToValAddrs(accAddrs)
	valAddr := valAddrs[0]

	// creating 10 ubdEntries with same height and 10 ubdEntries with different creation height
	err := createOldStateUnbonding(t, duplicateCreationHeight, valAddr, accAddr, cdc, store)
	require.NoError(t, err)

	legacySubspace := newMockSubspace(types.DefaultParams())

	testCases := []struct {
		name        string
		doMigration bool
	}{
		{
			name:        "without state migration",
			doMigration: false,
		},
		{
			name:        "with state migration",
			doMigration: true,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			if tc.doMigration {
				require.NoError(t, v4.MigrateStore(ctx, storeKey, cdc, legacySubspace))
			}

			ubd := getUBD(t, accAddr, valAddr, store, cdc)
			if tc.doMigration {
				var res types.Params
				bz := store.Get(v4.ParamsKey)
				require.NoError(t, cdc.Unmarshal(bz, &res))
				require.Equal(t, legacySubspace.ps, res)

				// checking the updated balance for duplicateCreationHeight
				for _, ubdEntry := range ubd.Entries {
					if ubdEntry.CreationHeight == duplicateCreationHeight {
						require.Equal(t, sdk.NewInt(100*10), ubdEntry.Balance)
						break
					}
				}
				require.Equal(t, 11, len(ubd.Entries))
			} else {
				require.Equal(t, true, true)
				require.Equal(t, 20, len(ubd.Entries))
			}
		})
	}
}

// createOldStateUnbonding will create the ubd entries with duplicate heights
// 10 duplicate heights and 10 unique ubd with creation height
func createOldStateUnbonding(t *testing.T, creationHeight int64, valAddr sdk.ValAddress, accAddr sdk.AccAddress, cdc codec.BinaryCodec, store storetypes.KVStore) error {
	unbondBalance := sdk.NewInt(100)
	completionTime := time.Now()
	ubdEntries := make([]types.UnbondingDelegationEntry, 0, 10)

	for i := int64(0); i < 10; i++ {
		ubdEntry := types.UnbondingDelegationEntry{
			CreationHeight: creationHeight,
			Balance:        unbondBalance,
			InitialBalance: unbondBalance,
			CompletionTime: completionTime,
		}
		ubdEntries = append(ubdEntries, ubdEntry)
		// creating more entries for testing the creation_heights
		ubdEntry.CreationHeight = i + 2
		ubdEntry.CompletionTime = completionTime.Add(time.Minute * 10)
		ubdEntries = append(ubdEntries, ubdEntry)
	}

	ubd := types.UnbondingDelegation{
		ValidatorAddress: valAddr.String(),
		DelegatorAddress: accAddr.String(),
		Entries:          ubdEntries,
	}

	// set the unbond delegation with validator and delegator
	bz := types.MustMarshalUBD(cdc, ubd)
	key := getUBDKey(accAddr, valAddr)
	store.Set(key, bz)
	return nil
}

func getUBD(t *testing.T, accAddr sdk.AccAddress, valAddr sdk.ValAddress, store storetypes.KVStore, cdc codec.BinaryCodec) types.UnbondingDelegation {
	// get the unbonding delegations
	var ubdRes types.UnbondingDelegation
	ubdbz := store.Get(getUBDKey(accAddr, valAddr))
	require.NoError(t, cdc.Unmarshal(ubdbz, &ubdRes))
	return ubdRes
}

func getUBDKey(accAddr sdk.AccAddress, valAddr sdk.ValAddress) []byte {
	return types.GetUBDKey(accAddr, valAddr)
}
