package keeper_test

import (
	"encoding/hex"

	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	govtypes "cosmossdk.io/x/gov/types"

	wasmtesting "github.com/cosmos/ibc-go/modules/light-clients/08-wasm/testing"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
)

func (suite *KeeperTestSuite) TestInitGenesis() {
	var (
		genesisState types.GenesisState
		expChecksums []string
	)

	testCases := []struct {
		name     string
		malleate func()
	}{
		{
			"success",
			func() {
				checksum := "b3a49b2914f5e6a673215e74325c1d153bb6776e079774e52c5b7e674d9ad3ab" //nolint:gosec // these are not hard-coded credentials

				genesisState = *types.NewGenesisState(
					[]types.Contract{
						{
							CodeBytes: wasmtesting.Code,
						},
					},
				)

				expChecksums = []string{checksum}
			},
		},
		{
			"success with empty genesis contract",
			func() {
				genesisState = *types.NewGenesisState([]types.Contract{})
				expChecksums = []string{}
			},
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			suite.SetupWasmWithMockVM()

			ctx := suite.chainA.GetContext()
			tc.malleate()

			err := GetSimApp(suite.chainA).WasmClientKeeper.InitGenesis(ctx, genesisState)
			suite.Require().NoError(err)

			var storedHashes []string
			checksums, err := types.GetAllChecksums(suite.chainA.GetContext())
			suite.Require().NoError(err)

			for _, hash := range checksums {
				storedHashes = append(storedHashes, hex.EncodeToString(hash))
			}

			suite.Require().Equal(len(expChecksums), len(storedHashes))
			suite.Require().ElementsMatch(expChecksums, storedHashes)
		})
	}
}

func (suite *KeeperTestSuite) TestExportGenesis() {
	suite.SetupWasmWithMockVM()

	ctx := suite.chainA.GetContext()

	expChecksum := "b3a49b2914f5e6a673215e74325c1d153bb6776e079774e52c5b7e674d9ad3ab" //nolint:gosec // these are not hard-coded credentials

	signer := authtypes.NewModuleAddress(govtypes.ModuleName).String()

	msg := types.NewMsgStoreCode(signer, wasmtesting.Code)
	res, err := GetSimApp(suite.chainA).WasmClientKeeper.StoreCode(ctx, msg)
	suite.Require().NoError(err)
	suite.Require().Equal(expChecksum, hex.EncodeToString(res.Checksum))

	genesisState := GetSimApp(suite.chainA).WasmClientKeeper.ExportGenesis(ctx)
	suite.Require().Len(genesisState.Contracts, 1)
	suite.Require().NotEmpty(genesisState.Contracts[0].CodeBytes)
}
