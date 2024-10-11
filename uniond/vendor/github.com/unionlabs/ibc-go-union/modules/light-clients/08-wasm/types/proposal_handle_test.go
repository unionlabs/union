package types_test

import (
	"encoding/json"

	cosmwasm "github.com/CosmWasm/wasmvm/v2"
	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"

	wasmtesting "github.com/cosmos/ibc-go/modules/light-clients/08-wasm/testing"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	ibctm "github.com/cosmos/ibc-go/v8/modules/light-clients/07-tendermint"
)

func (suite *TypesTestSuite) TestCheckSubstituteAndUpdateState() {
	var substituteClientState exported.ClientState
	var expectedClientStateBz []byte

	testCases := []struct {
		name     string
		malleate func()
		expErr   error
	}{
		{
			"success",
			func() {
				suite.mockVM.RegisterSudoCallback(
					types.MigrateClientStoreMsg{},
					func(_ cosmwasm.Checksum, _ wasmvmtypes.Env, sudoMsg []byte, store cosmwasm.KVStore, _ cosmwasm.GoAPI, _ cosmwasm.Querier, _ cosmwasm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
						var payload types.SudoMsg
						err := json.Unmarshal(sudoMsg, &payload)
						suite.Require().NoError(err)

						suite.Require().NotNil(payload.MigrateClientStore)
						suite.Require().Nil(payload.UpdateState)
						suite.Require().Nil(payload.UpdateStateOnMisbehaviour)
						suite.Require().Nil(payload.VerifyMembership)
						suite.Require().Nil(payload.VerifyNonMembership)
						suite.Require().Nil(payload.VerifyUpgradeAndUpdateState)

						bz, err := json.Marshal(types.EmptyResult{})
						suite.Require().NoError(err)

						prefixedKey := types.SubjectPrefix
						prefixedKey = append(prefixedKey, host.ClientStateKey()...)
						expectedClientStateBz = wasmtesting.CreateMockClientStateBz(suite.chainA.Codec, suite.checksum)
						store.Set(prefixedKey, expectedClientStateBz)

						return &wasmvmtypes.ContractResult{Ok: &wasmvmtypes.Response{Data: bz}}, wasmtesting.DefaultGasUsed, nil
					},
				)
			},
			nil,
		},
		{
			"failure: invalid substitute client state",
			func() {
				substituteClientState = &ibctm.ClientState{}
			},
			clienttypes.ErrInvalidClient,
		},
		{
			"failure: checksums do not match",
			func() {
				substituteClientState = &types.ClientState{
					Checksum: []byte("invalid"),
				}
			},
			clienttypes.ErrInvalidClient,
		},
		{
			"failure: vm returns error",
			func() {
				suite.mockVM.RegisterSudoCallback(
					types.MigrateClientStoreMsg{},
					func(_ cosmwasm.Checksum, _ wasmvmtypes.Env, _ []byte, _ cosmwasm.KVStore, _ cosmwasm.GoAPI, _ cosmwasm.Querier, _ cosmwasm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
						return nil, wasmtesting.DefaultGasUsed, wasmtesting.ErrMockVM
					},
				)
			},
			types.ErrVMError,
		},
		{
			"failure: contract returns error",
			func() {
				suite.mockVM.RegisterSudoCallback(
					types.MigrateClientStoreMsg{},
					func(_ cosmwasm.Checksum, _ wasmvmtypes.Env, _ []byte, _ cosmwasm.KVStore, _ cosmwasm.GoAPI, _ cosmwasm.Querier, _ cosmwasm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
						return &wasmvmtypes.ContractResult{Err: wasmtesting.ErrMockContract.Error()}, wasmtesting.DefaultGasUsed, nil
					},
				)
			},
			types.ErrWasmContractCallFailed,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupWasmWithMockVM()
			expectedClientStateBz = nil

			endpointA := wasmtesting.NewWasmEndpoint(suite.chainA)
			err := endpointA.CreateClient()
			suite.Require().NoError(err)

			subjectClientStore := suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(suite.chainA.GetContext(), endpointA.ClientID)
			subjectClientState := endpointA.GetClientState()

			substituteEndpoint := wasmtesting.NewWasmEndpoint(suite.chainA)
			err = substituteEndpoint.CreateClient()
			suite.Require().NoError(err)

			substituteClientState = substituteEndpoint.GetClientState()
			substituteClientStore := suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(suite.chainA.GetContext(), substituteEndpoint.ClientID)

			tc.malleate()

			err = subjectClientState.CheckSubstituteAndUpdateState(
				suite.chainA.GetContext(),
				suite.chainA.Codec,
				subjectClientStore,
				substituteClientStore,
				substituteClientState,
			)

			expPass := tc.expErr == nil
			if expPass {
				suite.Require().NoError(err)

				clientStateBz := subjectClientStore.Get(host.ClientStateKey())
				suite.Require().Equal(expectedClientStateBz, clientStateBz)
			} else {
				suite.Require().ErrorIs(err, tc.expErr)
			}
		})
	}
}
