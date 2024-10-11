package keeper_test

import (
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/internal/ibcwasm"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/keeper"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
)

func (suite *KeeperTestSuite) TestMigrateWasmStore() {
	testCases := []struct {
		name      string
		checksums [][]byte
	}{
		{
			"success: empty checksums",
			[][]byte{},
		},
		{
			"success: multiple checksums",
			[][]byte{[]byte("hash1"), []byte("hash2"), []byte("hash3")},
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			suite.storeChecksums(tc.checksums)

			// run the migration
			wasmKeeper := GetSimApp(suite.chainA).WasmClientKeeper
			m := keeper.NewMigrator(wasmKeeper)

			err := m.MigrateChecksums(suite.chainA.GetContext())
			suite.Require().NoError(err)

			// check that they were stored in KeySet
			for _, hash := range tc.checksums {
				suite.Require().True(ibcwasm.Checksums.Has(suite.chainA.GetContext(), hash))
			}

			// check that the data under the old key was deleted
			store := suite.chainA.GetContext().KVStore(GetSimApp(suite.chainA).GetKey(types.StoreKey))
			suite.Require().Nil(store.Get([]byte(types.KeyChecksums)))
		})
	}
}

// storeChecksums stores the given checksums under the KeyChecksums key, it runs
// each time on an empty store so we don't need to read the previous checksums.
func (suite *KeeperTestSuite) storeChecksums(checksums [][]byte) {
	ctx := suite.chainA.GetContext()

	store := ctx.KVStore(GetSimApp(suite.chainA).GetKey(types.StoreKey))
	checksum := types.Checksums{Checksums: checksums}
	bz, err := GetSimApp(suite.chainA).AppCodec().Marshal(&checksum)
	suite.Require().NoError(err)

	store.Set([]byte(types.KeyChecksums), bz)
}
