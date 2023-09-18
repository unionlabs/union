package v3_test

import (
	"math/rand"
	"testing"
	"time"

	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/testutil"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/x/auth"
	authexported "github.com/cosmos/cosmos-sdk/x/auth/exported"
	"github.com/cosmos/cosmos-sdk/x/auth/keeper"
	v1 "github.com/cosmos/cosmos-sdk/x/auth/migrations/v1"
	v4 "github.com/cosmos/cosmos-sdk/x/auth/migrations/v4"
	authtestutil "github.com/cosmos/cosmos-sdk/x/auth/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

type mockSubspace struct {
	ps authtypes.Params
}

func newMockSubspace(ps authtypes.Params) mockSubspace {
	return mockSubspace{ps: ps}
}

func (ms mockSubspace) GetParamSet(ctx sdk.Context, ps authexported.ParamSet) {
	*ps.(*authtypes.Params) = ms.ps
}

// TestMigrateMapAccAddressToAccNumberKey test cases for state migration of map to accAddr to accNum
func TestMigrateMapAccAddressToAccNumberKey(t *testing.T) {
	encCfg := moduletestutil.MakeTestEncodingConfig(auth.AppModuleBasic{})
	cdc := encCfg.Codec

	storeKey := sdk.NewKVStoreKey(v1.ModuleName)
	tKey := sdk.NewTransientStoreKey("transient_test")
	ctx := testutil.DefaultContext(storeKey, tKey)
	store := ctx.KVStore(storeKey)

	var accountKeeper keeper.AccountKeeper

	app, err := simtestutil.Setup(
		authtestutil.AppConfig,
		&accountKeeper,
	)
	require.NoError(t, err)

	legacySubspace := newMockSubspace(authtypes.DefaultParams())
	require.NoError(t, v4.Migrate(ctx, store, legacySubspace, cdc))

	// new base account
	senderPrivKey := secp256k1.GenPrivKey()
	randAccNumber := uint64(rand.Intn(100000-10000) + 10000)
	acc := authtypes.NewBaseAccount(senderPrivKey.PubKey().Address().Bytes(), senderPrivKey.PubKey(), randAccNumber, 0)

	ctx = app.BaseApp.NewContext(false, tmproto.Header{Time: time.Now()})

	// migrator
	m := keeper.NewMigrator(accountKeeper, app.GRPCQueryRouter(), legacySubspace)
	// set the account to store with map acc addr to acc number
	require.NoError(t, m.V45_SetAccount(ctx, acc))

	testCases := []struct {
		name        string
		doMigration bool
		accNum      uint64
	}{
		{
			name:        "without state migration",
			doMigration: false,
			accNum:      acc.AccountNumber,
		},
		{
			name:        "with state migration",
			doMigration: true,
			accNum:      acc.AccountNumber,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			if tc.doMigration {
				require.NoError(t, m.Migrate2to3(ctx))
			}

			//  get the account address by acc id
			accAddr := accountKeeper.GetAccountAddressByID(ctx, tc.accNum)

			if tc.doMigration {
				require.Equal(t, accAddr, acc.Address)
			} else {
				require.Equal(t, len(accAddr), 0)
			}
		})
	}
}
