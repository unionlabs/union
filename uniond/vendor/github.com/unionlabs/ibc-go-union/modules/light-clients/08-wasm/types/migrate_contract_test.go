package types_test

import (
	"encoding/hex"
	"encoding/json"

	wasmvm "github.com/CosmWasm/wasmvm/v2"
	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"

	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/internal/ibcwasm"
	wasmtesting "github.com/cosmos/ibc-go/modules/light-clients/08-wasm/testing"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
)

func (suite *TypesTestSuite) TestMigrateContract() {
	var (
		oldHash        []byte
		newHash        []byte
		payload        []byte
		expClientState *types.ClientState
	)

	testCases := []struct {
		name     string
		malleate func()
		expErr   error
	}{
		{
			"success: no update to client state",
			func() {
				err := ibcwasm.Checksums.Set(suite.chainA.GetContext(), newHash)
				suite.Require().NoError(err)

				payload = []byte{1}
				expChecksum := wasmvmtypes.ForceNewChecksum(hex.EncodeToString(newHash))

				suite.mockVM.MigrateFn = func(checksum wasmvm.Checksum, env wasmvmtypes.Env, msg []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
					suite.Require().Equal(expChecksum, checksum)
					suite.Require().Equal(defaultWasmClientID, env.Contract.Address)
					suite.Require().Equal(payload, msg)

					data, err := json.Marshal(types.EmptyResult{})
					suite.Require().NoError(err)

					return &wasmvmtypes.ContractResult{Ok: &wasmvmtypes.Response{Data: data}}, wasmtesting.DefaultGasUsed, nil
				}
			},
			nil,
		},
		{
			"success: update client state",
			func() {
				suite.mockVM.MigrateFn = func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, store wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
					expClientState = types.NewClientState([]byte{1}, newHash, clienttypes.NewHeight(2000, 2))
					store.Set(host.ClientStateKey(), clienttypes.MustMarshalClientState(suite.chainA.App.AppCodec(), expClientState))

					data, err := json.Marshal(types.EmptyResult{})
					suite.Require().NoError(err)

					return &wasmvmtypes.ContractResult{Ok: &wasmvmtypes.Response{Data: data}}, wasmtesting.DefaultGasUsed, nil
				}
			},
			nil,
		},
		{
			"failure: new and old checksum are the same",
			func() {
				newHash = oldHash
				// this should not be called
				suite.mockVM.MigrateFn = func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
					panic("unreachable")
				}
			},
			types.ErrWasmCodeExists,
		},
		{
			"failure: checksum not found",
			func() {
				err := ibcwasm.Checksums.Remove(suite.chainA.GetContext(), newHash)
				suite.Require().NoError(err)
			},
			types.ErrWasmChecksumNotFound,
		},
		{
			"failure: vm returns error",
			func() {
				err := ibcwasm.Checksums.Set(suite.chainA.GetContext(), newHash)
				suite.Require().NoError(err)

				suite.mockVM.MigrateFn = func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
					return nil, wasmtesting.DefaultGasUsed, wasmtesting.ErrMockVM
				}
			},
			types.ErrVMError,
		},
		{
			"failure: contract returns error",
			func() {
				err := ibcwasm.Checksums.Set(suite.chainA.GetContext(), newHash)
				suite.Require().NoError(err)

				suite.mockVM.MigrateFn = func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
					return &wasmvmtypes.ContractResult{Err: wasmtesting.ErrMockContract.Error()}, wasmtesting.DefaultGasUsed, nil
				}
			},
			types.ErrWasmContractCallFailed,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupWasmWithMockVM()

			var err error
			oldHash, err = types.CreateChecksum(wasmtesting.Code)
			suite.Require().NoError(err)
			newHash, err = types.CreateChecksum(wasmtesting.CreateMockContract([]byte{1, 2, 3}))
			suite.Require().NoError(err)

			err = ibcwasm.Checksums.Set(suite.chainA.GetContext(), newHash)
			suite.Require().NoError(err)

			endpointA := wasmtesting.NewWasmEndpoint(suite.chainA)
			err = endpointA.CreateClient()
			suite.Require().NoError(err)

			clientStore := suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(suite.chainA.GetContext(), endpointA.ClientID)
			clientState, ok := endpointA.GetClientState().(*types.ClientState)
			suite.Require().True(ok)

			expClientState = clientState

			tc.malleate()

			err = clientState.MigrateContract(suite.chainA.GetContext(), suite.chainA.App.AppCodec(), clientStore, endpointA.ClientID, newHash, payload)

			// updated client state
			clientState, ok = endpointA.GetClientState().(*types.ClientState)
			suite.Require().True(ok)

			expPass := tc.expErr == nil
			if expPass {
				suite.Require().NoError(err)
				suite.Require().Equal(expClientState, clientState)
			} else {
				suite.Require().ErrorIs(err, tc.expErr)
			}
		})
	}
}
