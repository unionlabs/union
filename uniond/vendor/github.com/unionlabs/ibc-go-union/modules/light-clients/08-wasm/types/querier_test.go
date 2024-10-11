package types_test

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"math"

	wasmvm "github.com/CosmWasm/wasmvm/v2"
	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/internal/ibcwasm"
	wasmtesting "github.com/cosmos/ibc-go/modules/light-clients/08-wasm/testing"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

type CustomQuery struct {
	Echo *QueryEcho `json:"echo,omitempty"`
}

type QueryEcho struct {
	Data string `json:"data"`
}

func MockCustomQuerier() func(sdk.Context, json.RawMessage) ([]byte, error) {
	return func(ctx sdk.Context, request json.RawMessage) ([]byte, error) {
		var customQuery CustomQuery
		err := json.Unmarshal([]byte(request), &customQuery)
		if err != nil {
			return nil, wasmtesting.ErrMockContract
		}

		if customQuery.Echo != nil {
			data, err := json.Marshal(customQuery.Echo.Data)
			return data, err
		}

		return nil, wasmtesting.ErrMockContract
	}
}

func (suite *TypesTestSuite) TestCustomQuery() {
	testCases := []struct {
		name     string
		malleate func()
	}{
		{
			"success: custom query",
			func() {
				querierPlugin := types.QueryPlugins{
					Custom: MockCustomQuerier(),
				}
				ibcwasm.SetQueryPlugins(&querierPlugin)

				suite.mockVM.RegisterQueryCallback(types.StatusMsg{}, func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, querier wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.QueryResult, uint64, error) {
					echo := CustomQuery{
						Echo: &QueryEcho{
							Data: "hello world",
						},
					}
					echoJSON, err := json.Marshal(echo)
					suite.Require().NoError(err)

					resp, err := querier.Query(wasmvmtypes.QueryRequest{
						Custom: json.RawMessage(echoJSON),
					}, math.MaxUint64)
					suite.Require().NoError(err)

					var respData string
					err = json.Unmarshal(resp, &respData)
					suite.Require().NoError(err)
					suite.Require().Equal("hello world", respData)

					resp, err = json.Marshal(types.StatusResult{Status: exported.Active.String()})
					suite.Require().NoError(err)

					return &wasmvmtypes.QueryResult{Ok: resp}, wasmtesting.DefaultGasUsed, nil
				})
			},
		},
		{
			"failure: default query",
			func() {
				suite.mockVM.RegisterQueryCallback(types.StatusMsg{}, func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, querier wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.QueryResult, uint64, error) {
					resp, err := querier.Query(wasmvmtypes.QueryRequest{Custom: json.RawMessage("{}")}, math.MaxUint64)
					suite.Require().ErrorIs(err, wasmvmtypes.UnsupportedRequest{Kind: "Custom queries are not allowed"})
					suite.Require().Nil(resp)

					return nil, wasmtesting.DefaultGasUsed, err
				})
			},
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			suite.SetupWasmWithMockVM()

			endpoint := wasmtesting.NewWasmEndpoint(suite.chainA)
			err := endpoint.CreateClient()
			suite.Require().NoError(err)

			tc.malleate()

			clientStore := suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(suite.chainA.GetContext(), endpoint.ClientID)
			clientState := endpoint.GetClientState()
			clientState.Status(suite.chainA.GetContext(), clientStore, suite.chainA.App.AppCodec())

			// reset query plugins after each test
			ibcwasm.SetQueryPlugins(types.NewDefaultQueryPlugins())
		})
	}
}

func (suite *TypesTestSuite) TestStargateQuery() {
	typeURL := "/ibc.lightclients.wasm.v1.Query/Checksums"

	var (
		endpoint          *wasmtesting.WasmEndpoint
		expDiscardedState = false
		proofKey          = []byte("mock-key")
		testKey           = []byte("test-key")
		value             = []byte("mock-value")
	)

	testCases := []struct {
		name     string
		malleate func()
		expError error
	}{
		{
			"success: custom query",
			func() {
				querierPlugin := types.QueryPlugins{
					Stargate: types.AcceptListStargateQuerier([]string{typeURL}),
				}

				ibcwasm.SetQueryPlugins(&querierPlugin)

				suite.mockVM.RegisterQueryCallback(types.TimestampAtHeightMsg{}, func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, store wasmvm.KVStore, _ wasmvm.GoAPI, querier wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.QueryResult, uint64, error) {
					queryRequest := types.QueryChecksumsRequest{}
					bz, err := queryRequest.Marshal()
					suite.Require().NoError(err)

					resp, err := querier.Query(wasmvmtypes.QueryRequest{
						Stargate: &wasmvmtypes.StargateQuery{
							Path: typeURL,
							Data: bz,
						},
					}, math.MaxUint64)
					suite.Require().NoError(err)

					var respData types.QueryChecksumsResponse
					err = respData.Unmarshal(resp)
					suite.Require().NoError(err)

					expChecksum := hex.EncodeToString(suite.checksum)

					suite.Require().Len(respData.Checksums, 1)
					suite.Require().Equal(expChecksum, respData.Checksums[0])

					store.Set(testKey, value)

					result, err := json.Marshal(types.TimestampAtHeightResult{})
					suite.Require().NoError(err)

					return &wasmvmtypes.QueryResult{Ok: result}, wasmtesting.DefaultGasUsed, nil
				})
			},
			nil,
		},
		{
			// The following test sets a mock proof key and value in the ibc store and registers a query callback on the Status msg.
			// The Status handler will then perform the QueryVerifyMembershipRequest using the wasmvm.Querier.
			// As the VerifyMembership query rpc will call directly back into the same client, we also register a callback for VerifyMembership.
			// Here we decode the proof and verify the mock proof key and value are set in the ibc store.
			// This exercises the full flow through the grpc handler and into the light client for verification, handling encoding and routing.
			// Furthermore we write a test key and assert that the state changes made by this handler were discarded by the cachedCtx at the grpc handler.
			"success: verify membership query",
			func() {
				querierPlugin := types.QueryPlugins{
					Stargate: types.AcceptListStargateQuerier([]string{""}),
				}

				ibcwasm.SetQueryPlugins(&querierPlugin)

				store := suite.chainA.GetContext().KVStore(GetSimApp(suite.chainA).GetKey(exported.StoreKey))
				store.Set(proofKey, value)

				suite.coordinator.CommitBlock(suite.chainA)
				proof, proofHeight := endpoint.QueryProofAtHeight(proofKey, uint64(suite.chainA.GetContext().BlockHeight()))

				merklePath := commitmenttypes.NewMerklePath(string(proofKey))
				merklePath, err := commitmenttypes.ApplyPrefix(suite.chainA.GetPrefix(), merklePath)
				suite.Require().NoError(err)

				suite.mockVM.RegisterQueryCallback(types.TimestampAtHeightMsg{}, func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, querier wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.QueryResult, uint64, error) {
					queryRequest := clienttypes.QueryVerifyMembershipRequest{
						ClientId:    endpoint.ClientID,
						Proof:       proof,
						ProofHeight: proofHeight,
						MerklePath:  merklePath,
						Value:       value,
					}

					bz, err := queryRequest.Marshal()
					suite.Require().NoError(err)

					resp, err := querier.Query(wasmvmtypes.QueryRequest{
						Stargate: &wasmvmtypes.StargateQuery{
							Path: "/ibc.core.client.v1.Query/VerifyMembership",
							Data: bz,
						},
					}, math.MaxUint64)
					suite.Require().NoError(err)

					var respData clienttypes.QueryVerifyMembershipResponse
					err = respData.Unmarshal(resp)
					suite.Require().NoError(err)

					suite.Require().True(respData.Success)

					result, err := json.Marshal(types.TimestampAtHeightResult{})
					suite.Require().NoError(err)

					return &wasmvmtypes.QueryResult{Ok: result}, wasmtesting.DefaultGasUsed, nil
				})

				suite.mockVM.RegisterSudoCallback(types.VerifyMembershipMsg{}, func(_ wasmvm.Checksum, _ wasmvmtypes.Env, sudoMsg []byte, store wasmvm.KVStore,
					_ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction,
				) (*wasmvmtypes.ContractResult, uint64, error) {
					var payload types.SudoMsg
					err := json.Unmarshal(sudoMsg, &payload)
					suite.Require().NoError(err)

					var merkleProof commitmenttypes.MerkleProof
					err = suite.chainA.Codec.Unmarshal(payload.VerifyMembership.Proof, &merkleProof)
					suite.Require().NoError(err)

					root := commitmenttypes.NewMerkleRoot(suite.chainA.App.LastCommitID().Hash)
					err = merkleProof.VerifyMembership(commitmenttypes.GetSDKSpecs(), root, merklePath, payload.VerifyMembership.Value)
					suite.Require().NoError(err)

					bz, err := json.Marshal(types.EmptyResult{})
					suite.Require().NoError(err)

					expDiscardedState = true
					store.Set(testKey, value)

					return &wasmvmtypes.ContractResult{Ok: &wasmvmtypes.Response{Data: bz}}, wasmtesting.DefaultGasUsed, nil
				})
			},
			nil,
		},
		{
			"failure: default querier",
			func() {
				suite.mockVM.RegisterQueryCallback(types.TimestampAtHeightMsg{}, func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, store wasmvm.KVStore, _ wasmvm.GoAPI, querier wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.QueryResult, uint64, error) {
					queryRequest := types.QueryChecksumsRequest{}
					bz, err := queryRequest.Marshal()
					suite.Require().NoError(err)

					resp, err := querier.Query(wasmvmtypes.QueryRequest{
						Stargate: &wasmvmtypes.StargateQuery{
							Path: typeURL,
							Data: bz,
						},
					}, math.MaxUint64)
					suite.Require().ErrorIs(err, wasmvmtypes.UnsupportedRequest{Kind: fmt.Sprintf("'%s' path is not allowed from the contract", typeURL)})
					suite.Require().Nil(resp)

					store.Set(testKey, value)

					return nil, wasmtesting.DefaultGasUsed, err
				})
			},
			wasmvmtypes.UnsupportedRequest{Kind: fmt.Sprintf("'%s' path is not allowed from the contract", typeURL)},
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			expDiscardedState = false
			suite.SetupWasmWithMockVM()

			endpoint = wasmtesting.NewWasmEndpoint(suite.chainA)
			err := endpoint.CreateClient()
			suite.Require().NoError(err)

			tc.malleate()

			clientStore := suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(suite.chainA.GetContext(), endpoint.ClientID)
			clientState := endpoint.GetClientState()

			// NOTE: we register query callbacks against: types.TimestampAtHeightMsg{}
			// in practise, this can against any client state msg, however registering against types.StatusMsg{} introduces recursive loops
			// due to test case: "success: verify membership query"
			_, err = clientState.GetTimestampAtHeight(suite.chainA.GetContext(), clientStore, suite.chainA.App.AppCodec(), clienttypes.NewHeight(1, 100))

			expPass := tc.expError == nil
			if expPass {
				suite.Require().NoError(err)
			} else {
				// use error contains as wasmvm errors do not implement errors.Is method
				suite.Require().ErrorContains(err, tc.expError.Error())
			}

			if expDiscardedState {
				suite.Require().False(clientStore.Has(testKey))
			} else {
				suite.Require().True(clientStore.Has(testKey))
			}

			// reset query plugins after each test
			ibcwasm.SetQueryPlugins(types.NewDefaultQueryPlugins())
		})
	}
}
