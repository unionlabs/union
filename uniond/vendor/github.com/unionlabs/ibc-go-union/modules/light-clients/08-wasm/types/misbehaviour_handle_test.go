package types_test

import (
	"encoding/json"
	"errors"

	wasmvm "github.com/CosmWasm/wasmvm/v2"
	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"

	wasmtesting "github.com/cosmos/ibc-go/modules/light-clients/08-wasm/testing"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	ibctm "github.com/cosmos/ibc-go/v8/modules/light-clients/07-tendermint"
)

func (suite *TypesTestSuite) TestCheckForMisbehaviour() {
	var clientMessage exported.ClientMessage

	testCases := []struct {
		name                 string
		malleate             func()
		expFoundMisbehaviour bool
	}{
		{
			"success: no misbehaviour",
			func() {
				suite.mockVM.RegisterQueryCallback(types.CheckForMisbehaviourMsg{}, func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.QueryResult, uint64, error) {
					resp, err := json.Marshal(types.CheckForMisbehaviourResult{FoundMisbehaviour: false})
					suite.Require().NoError(err)
					return &wasmvmtypes.QueryResult{Ok: resp}, wasmtesting.DefaultGasUsed, nil
				})
			},
			false,
		},
		{
			"success: misbehaviour found", func() {
				suite.mockVM.RegisterQueryCallback(types.CheckForMisbehaviourMsg{}, func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.QueryResult, uint64, error) {
					resp, err := json.Marshal(types.CheckForMisbehaviourResult{FoundMisbehaviour: true})
					suite.Require().NoError(err)
					return &wasmvmtypes.QueryResult{Ok: resp}, wasmtesting.DefaultGasUsed, nil
				})
			},
			true,
		},
		{
			"success: contract error, resp cannot be marshalled", func() {
				suite.mockVM.RegisterQueryCallback(types.CheckForMisbehaviourMsg{}, func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.QueryResult, uint64, error) {
					resp := "cannot be unmarshalled"
					return &wasmvmtypes.QueryResult{Ok: []byte(resp)}, wasmtesting.DefaultGasUsed, nil
				})
			},
			false,
		},
		{
			"success: contract returns error", func() {
				suite.mockVM.RegisterQueryCallback(types.CheckForMisbehaviourMsg{}, func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.QueryResult, uint64, error) {
					return &wasmvmtypes.QueryResult{Err: wasmtesting.ErrMockContract.Error()}, wasmtesting.DefaultGasUsed, nil
				})
			},
			false,
		},
		{
			"success: vm returns error, ", func() {
				suite.mockVM.RegisterQueryCallback(types.CheckForMisbehaviourMsg{}, func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.QueryResult, uint64, error) {
					return nil, 0, errors.New("invalid block ID")
				})
			},
			false,
		},
		{
			"success: invalid client message", func() {
				clientMessage = &ibctm.Header{}
				// we will not register the callback here because this test case does not reach the VM
			},
			false,
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			// reset suite to create fresh application state
			suite.SetupWasmWithMockVM()
			endpoint := wasmtesting.NewWasmEndpoint(suite.chainA)
			err := endpoint.CreateClient()
			suite.Require().NoError(err)

			clientState := endpoint.GetClientState()
			clientMessage = &types.ClientMessage{
				Data: clienttypes.MustMarshalClientMessage(suite.chainA.App.AppCodec(), wasmtesting.MockTendermintClientMisbehaviour),
			}

			clientStore := suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(suite.chainA.GetContext(), endpoint.ClientID)

			tc.malleate()

			foundMisbehaviour := clientState.CheckForMisbehaviour(suite.chainA.GetContext(), suite.chainA.App.AppCodec(), clientStore, clientMessage)
			suite.Require().Equal(tc.expFoundMisbehaviour, foundMisbehaviour)
		})
	}
}
