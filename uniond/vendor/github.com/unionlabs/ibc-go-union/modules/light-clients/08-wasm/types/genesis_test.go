package types_test

import (
	"encoding/json"

	wasmvm "github.com/CosmWasm/wasmvm/v2"
	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"

	errorsmod "cosmossdk.io/errors"

	wasmtesting "github.com/cosmos/ibc-go/modules/light-clients/08-wasm/testing"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

func (suite *TypesTestSuite) TestValidateGenesis() {
	testCases := []struct {
		name     string
		genState *types.GenesisState
		expPass  bool
	}{
		{
			"valid genesis",
			&types.GenesisState{
				Contracts: []types.Contract{{CodeBytes: []byte{1}}},
			},
			true,
		},
		{
			"invalid genesis",
			&types.GenesisState{
				Contracts: []types.Contract{{CodeBytes: []byte{}}},
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		err := tc.genState.Validate()
		if tc.expPass {
			suite.Require().NoError(err)
		} else {
			suite.Require().Error(err)
		}
	}
}

func (suite *TypesTestSuite) TestExportMetatada() {
	mockMetadata := clienttypes.NewGenesisMetadata([]byte("key"), []byte("value"))

	testCases := []struct {
		name        string
		malleate    func()
		expPanic    error
		expMetadata []exported.GenesisMetadata
	}{
		{
			"success",
			func() {
				suite.mockVM.RegisterQueryCallback(types.ExportMetadataMsg{}, func(_ wasmvm.Checksum, _ wasmvmtypes.Env, queryMsg []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.QueryResult, uint64, error) {
					var msg types.QueryMsg

					err := json.Unmarshal(queryMsg, &msg)
					suite.Require().NoError(err)

					suite.Require().NotNil(msg.ExportMetadata)
					suite.Require().Nil(msg.VerifyClientMessage)
					suite.Require().Nil(msg.Status)
					suite.Require().Nil(msg.CheckForMisbehaviour)
					suite.Require().Nil(msg.TimestampAtHeight)

					resp, err := json.Marshal(types.ExportMetadataResult{
						GenesisMetadata: []clienttypes.GenesisMetadata{mockMetadata},
					})
					suite.Require().NoError(err)

					return &wasmvmtypes.QueryResult{Ok: resp}, wasmtesting.DefaultGasUsed, nil
				})
			},
			nil,
			[]exported.GenesisMetadata{mockMetadata},
		},
		{
			"failure: contract returns an error",
			func() {
				suite.mockVM.RegisterQueryCallback(types.ExportMetadataMsg{}, func(_ wasmvm.Checksum, _ wasmvmtypes.Env, queryMsg []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.QueryResult, uint64, error) {
					return nil, 0, wasmtesting.ErrMockContract
				})
			},
			errorsmod.Wrapf(types.ErrVMError, wasmtesting.ErrMockContract.Error()),
			nil,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupWasmWithMockVM()

			endpoint := wasmtesting.NewWasmEndpoint(suite.chainA)
			err := endpoint.CreateClient()
			suite.Require().NoError(err)

			clientState := endpoint.GetClientState()

			tc.malleate()

			store := suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(suite.chainA.GetContext(), endpoint.ClientID)

			var metadata []exported.GenesisMetadata
			exportMetadata := func() {
				metadata = clientState.ExportMetadata(store)
			}

			if tc.expPanic == nil {
				exportMetadata()

				suite.Require().Equal(tc.expMetadata, metadata)
			} else {
				suite.Require().PanicsWithError(tc.expPanic.Error(), exportMetadata)
			}
		})
	}
}
