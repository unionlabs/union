package types_test

import (
	"encoding/json"
	"time"

	wasmvm "github.com/CosmWasm/wasmvm/v2"
	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"

	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/internal/ibcwasm"
	internaltypes "github.com/cosmos/ibc-go/modules/light-clients/08-wasm/internal/types"
	wasmtesting "github.com/cosmos/ibc-go/modules/light-clients/08-wasm/testing"
	mock "github.com/cosmos/ibc-go/modules/light-clients/08-wasm/testing/mock"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	solomachine "github.com/cosmos/ibc-go/v8/modules/light-clients/06-solomachine"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

func (suite *TypesTestSuite) TestStatus() {
	testCases := []struct {
		name      string
		malleate  func()
		expStatus exported.Status
	}{
		{
			"client is active",
			func() {},
			exported.Active,
		},
		{
			"client is frozen",
			func() {
				suite.mockVM.RegisterQueryCallback(types.StatusMsg{}, func(checksum wasmvm.Checksum, env wasmvmtypes.Env, queryMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.QueryResult, uint64, error) {
					resp, err := json.Marshal(types.StatusResult{Status: exported.Frozen.String()})
					suite.Require().NoError(err)
					return &wasmvmtypes.QueryResult{Ok: resp}, wasmtesting.DefaultGasUsed, nil
				})
			},
			exported.Frozen,
		},
		{
			"client status is expired",
			func() {
				suite.mockVM.RegisterQueryCallback(types.StatusMsg{}, func(checksum wasmvm.Checksum, env wasmvmtypes.Env, queryMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.QueryResult, uint64, error) {
					resp, err := json.Marshal(types.StatusResult{Status: exported.Expired.String()})
					suite.Require().NoError(err)
					return &wasmvmtypes.QueryResult{Ok: resp}, wasmtesting.DefaultGasUsed, nil
				})
			},
			exported.Expired,
		},
		{
			"client status is unknown: vm returns an error",
			func() {
				suite.mockVM.RegisterQueryCallback(types.StatusMsg{}, func(checksum wasmvm.Checksum, env wasmvmtypes.Env, queryMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.QueryResult, uint64, error) {
					return nil, 0, wasmtesting.ErrMockContract
				})
			},
			exported.Unknown,
		},
		{
			"client status is unauthorized: checksum is not stored",
			func() {
				err := ibcwasm.Checksums.Remove(suite.chainA.GetContext(), suite.checksum)
				suite.Require().NoError(err)
			},
			exported.Unauthorized,
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

			status := clientState.Status(suite.chainA.GetContext(), clientStore, suite.chainA.App.AppCodec())
			suite.Require().Equal(tc.expStatus, status)
		})
	}
}

func (suite *TypesTestSuite) TestGetTimestampAtHeight() {
	var height exported.Height

	expectedTimestamp := uint64(time.Now().UnixNano())

	testCases := []struct {
		name     string
		malleate func()
		expErr   error
	}{
		{
			"success",
			func() {
				suite.mockVM.RegisterQueryCallback(types.TimestampAtHeightMsg{}, func(_ wasmvm.Checksum, _ wasmvmtypes.Env, queryMsg []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.QueryResult, uint64, error) {
					var payload types.QueryMsg
					err := json.Unmarshal(queryMsg, &payload)
					suite.Require().NoError(err)

					suite.Require().NotNil(payload.TimestampAtHeight)
					suite.Require().Nil(payload.CheckForMisbehaviour)
					suite.Require().Nil(payload.Status)
					suite.Require().Nil(payload.ExportMetadata)
					suite.Require().Nil(payload.VerifyClientMessage)

					resp, err := json.Marshal(types.TimestampAtHeightResult{Timestamp: expectedTimestamp})
					suite.Require().NoError(err)

					return &wasmvmtypes.QueryResult{Ok: resp}, wasmtesting.DefaultGasUsed, nil
				})
			},
			nil,
		},
		{
			"failure: vm returns error",
			func() {
				suite.mockVM.RegisterQueryCallback(types.TimestampAtHeightMsg{}, func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.QueryResult, uint64, error) {
					return nil, 0, wasmtesting.ErrMockVM
				})
			},
			types.ErrVMError,
		},
		{
			"failure: contract returns error",
			func() {
				suite.mockVM.RegisterQueryCallback(types.TimestampAtHeightMsg{}, func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.QueryResult, uint64, error) {
					return &wasmvmtypes.QueryResult{Err: wasmtesting.ErrMockContract.Error()}, 0, nil
				})
			},
			types.ErrWasmContractCallFailed,
		},
		{
			"error: invalid height",
			func() {
				height = mock.Height{}
			},
			ibcerrors.ErrInvalidType,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupWasmWithMockVM()

			endpoint := wasmtesting.NewWasmEndpoint(suite.chainA)
			err := endpoint.CreateClient()
			suite.Require().NoError(err)

			clientStore := suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(suite.chainA.GetContext(), endpoint.ClientID)
			clientState := endpoint.GetClientState().(*types.ClientState)
			height = clientState.GetLatestHeight()

			tc.malleate()

			timestamp, err := clientState.GetTimestampAtHeight(suite.chainA.GetContext(), clientStore, suite.chainA.App.AppCodec(), height)

			expPass := tc.expErr == nil
			if expPass {
				suite.Require().NoError(err)
				suite.Require().Equal(expectedTimestamp, timestamp)
			} else {
				suite.Require().ErrorIs(err, tc.expErr)
			}
		})
	}
}

func (suite *TypesTestSuite) TestValidate() {
	testCases := []struct {
		name        string
		clientState *types.ClientState
		expPass     bool
	}{
		{
			name:        "valid client",
			clientState: types.NewClientState([]byte{0}, wasmtesting.Code, clienttypes.ZeroHeight()),
			expPass:     true,
		},
		{
			name:        "nil data",
			clientState: types.NewClientState(nil, wasmtesting.Code, clienttypes.ZeroHeight()),
			expPass:     false,
		},
		{
			name:        "empty data",
			clientState: types.NewClientState([]byte{}, wasmtesting.Code, clienttypes.ZeroHeight()),
			expPass:     false,
		},
		{
			name:        "nil checksum",
			clientState: types.NewClientState([]byte{0}, nil, clienttypes.ZeroHeight()),
			expPass:     false,
		},
		{
			name:        "empty checksum",
			clientState: types.NewClientState([]byte{0}, []byte{}, clienttypes.ZeroHeight()),
			expPass:     false,
		},
		{
			name: "longer than 32 bytes checksum",
			clientState: types.NewClientState(
				[]byte{0},
				[]byte{
					0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
					10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
					20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
					30, 31, 32, 33,
				},
				clienttypes.ZeroHeight(),
			),
			expPass: false,
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			err := tc.clientState.Validate()
			if tc.expPass {
				suite.Require().NoError(err, tc.name)
			} else {
				suite.Require().Error(err, tc.name)
			}
		})
	}
}

func (suite *TypesTestSuite) TestInitialize() {
	var (
		consensusState exported.ConsensusState
		clientState    exported.ClientState
		clientStore    storetypes.KVStore
	)

	testCases := []struct {
		name     string
		malleate func()
		expError error
	}{
		{
			"success: new mock client",
			func() {},
			nil,
		},
		{
			"success: validate contract address",
			func() {
				suite.mockVM.InstantiateFn = func(_ wasmvm.Checksum, env wasmvmtypes.Env, _ wasmvmtypes.MessageInfo, initMsg []byte, store wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
					var payload types.InstantiateMessage
					err := json.Unmarshal(initMsg, &payload)
					suite.Require().NoError(err)

					suite.Require().Equal(env.Contract.Address, defaultWasmClientID)

					wrappedClientState := clienttypes.MustUnmarshalClientState(suite.chainA.App.AppCodec(), payload.ClientState)

					clientState := types.NewClientState(payload.ClientState, payload.Checksum, wrappedClientState.GetLatestHeight().(clienttypes.Height))
					clientStateBz := clienttypes.MustMarshalClientState(suite.chainA.App.AppCodec(), clientState)
					store.Set(host.ClientStateKey(), clientStateBz)

					consensusState := types.NewConsensusState(payload.ConsensusState)
					consensusStateBz := clienttypes.MustMarshalConsensusState(suite.chainA.App.AppCodec(), consensusState)
					store.Set(host.ConsensusStateKey(clientState.GetLatestHeight()), consensusStateBz)

					resp, err := json.Marshal(types.EmptyResult{})
					suite.Require().NoError(err)

					return &wasmvmtypes.ContractResult{Ok: &wasmvmtypes.Response{Data: resp}}, 0, nil
				}
			},
			nil,
		},
		{
			"failure: clientStore prefix does not include clientID",
			func() {
				clientStore = suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(suite.chainA.GetContext(), ibctesting.InvalidID)
			},
			types.ErrVMError,
		},
		{
			"failure: invalid consensus state",
			func() {
				// set upgraded consensus state to solomachine consensus state
				consensusState = &solomachine.ConsensusState{}
			},
			clienttypes.ErrInvalidConsensus,
		},
		{
			"failure: checksum has not been stored.",
			func() {
				clientState = types.NewClientState([]byte{1}, []byte("unknown"), clienttypes.NewHeight(0, 1))
			},
			types.ErrInvalidChecksum,
		},
		{
			"failure: vm returns error",
			func() {
				suite.mockVM.InstantiateFn = func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ wasmvmtypes.MessageInfo, _ []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
					return nil, 0, wasmtesting.ErrMockVM
				}
			},
			types.ErrVMError,
		},
		{
			"failure: contract returns error",
			func() {
				suite.mockVM.InstantiateFn = func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ wasmvmtypes.MessageInfo, _ []byte, _ wasmvm.KVStore, _ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
					return &wasmvmtypes.ContractResult{Err: wasmtesting.ErrMockContract.Error()}, 0, nil
				}
			},
			types.ErrWasmContractCallFailed,
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			suite.SetupWasmWithMockVM()

			wrappedClientStateBz := clienttypes.MustMarshalClientState(suite.chainA.App.AppCodec(), wasmtesting.MockTendermitClientState)
			wrappedClientConsensusStateBz := clienttypes.MustMarshalConsensusState(suite.chainA.App.AppCodec(), wasmtesting.MockTendermintClientConsensusState)
			clientState = types.NewClientState(wrappedClientStateBz, suite.checksum, wasmtesting.MockTendermitClientState.GetLatestHeight().(clienttypes.Height))
			consensusState = types.NewConsensusState(wrappedClientConsensusStateBz)

			clientID := suite.chainA.App.GetIBCKeeper().ClientKeeper.GenerateClientIdentifier(suite.chainA.GetContext(), clientState.ClientType())
			clientStore = suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(suite.chainA.GetContext(), clientID)

			tc.malleate()

			err := clientState.Initialize(suite.chainA.GetContext(), suite.chainA.Codec, clientStore, consensusState)

			expPass := tc.expError == nil
			if expPass {
				suite.Require().NoError(err)

				expClientState := clienttypes.MustMarshalClientState(suite.chainA.Codec, clientState)
				suite.Require().Equal(expClientState, clientStore.Get(host.ClientStateKey()))

				expConsensusState := clienttypes.MustMarshalConsensusState(suite.chainA.Codec, consensusState)
				suite.Require().Equal(expConsensusState, clientStore.Get(host.ConsensusStateKey(clientState.GetLatestHeight())))
			} else {
				suite.Require().ErrorIs(err, tc.expError)
			}
		})
	}
}

func (suite *TypesTestSuite) TestVerifyMembership() {
	var (
		clientState      exported.ClientState
		expClientStateBz []byte
		path             exported.Path
		proof            []byte
		proofHeight      exported.Height
		value            []byte
	)

	testCases := []struct {
		name     string
		malleate func()
		expError error
	}{
		{
			"success",
			func() {
				expClientStateBz = GetSimApp(suite.chainA).GetIBCKeeper().ClientKeeper.MustMarshalClientState(clientState)
				suite.mockVM.RegisterSudoCallback(types.VerifyMembershipMsg{}, func(_ wasmvm.Checksum, _ wasmvmtypes.Env, sudoMsg []byte, _ wasmvm.KVStore,
					_ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction,
				) (*wasmvmtypes.ContractResult, uint64, error) {
					var payload types.SudoMsg
					err := json.Unmarshal(sudoMsg, &payload)
					suite.Require().NoError(err)

					suite.Require().NotNil(payload.VerifyMembership)
					suite.Require().Nil(payload.UpdateState)
					suite.Require().Nil(payload.UpdateStateOnMisbehaviour)
					suite.Require().Nil(payload.VerifyNonMembership)
					suite.Require().Nil(payload.VerifyUpgradeAndUpdateState)
					suite.Require().Equal(proofHeight, payload.VerifyMembership.Height)
					suite.Require().Equal(proof, payload.VerifyMembership.Proof)
					suite.Require().Equal(value, payload.VerifyMembership.Value)

					mpath, ok := path.(commitmenttypes.MerklePath)
					suite.Require().True(ok)
					// assert that merkle path key path is unmarshalled as bytes rather than strings
					suite.Require().Equal(internaltypes.ToMerklePathV2(mpath), payload.VerifyMembership.Path)

					bz, err := json.Marshal(types.EmptyResult{})
					suite.Require().NoError(err)

					return &wasmvmtypes.ContractResult{Ok: &wasmvmtypes.Response{Data: bz}}, wasmtesting.DefaultGasUsed, nil
				})
			},
			nil,
		},
		{
			"success: with update client state",
			func() {
				suite.mockVM.RegisterSudoCallback(types.VerifyMembershipMsg{}, func(_ wasmvm.Checksum, _ wasmvmtypes.Env, sudoMsg []byte, store wasmvm.KVStore,
					_ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction,
				) (*wasmvmtypes.ContractResult, uint64, error) {
					var payload types.SudoMsg
					err := json.Unmarshal(sudoMsg, &payload)
					suite.Require().NoError(err)

					suite.Require().NotNil(payload.VerifyMembership)
					suite.Require().Nil(payload.UpdateState)
					suite.Require().Nil(payload.UpdateStateOnMisbehaviour)
					suite.Require().Nil(payload.VerifyNonMembership)
					suite.Require().Nil(payload.VerifyUpgradeAndUpdateState)
					suite.Require().Equal(proofHeight, payload.VerifyMembership.Height)
					suite.Require().Equal(proof, payload.VerifyMembership.Proof)
					suite.Require().Equal(value, payload.VerifyMembership.Value)

					mpath, ok := path.(commitmenttypes.MerklePath)
					suite.Require().True(ok)
					// assert that merkle path key path is unmarshalled as bytes rather than strings
					suite.Require().Equal(internaltypes.ToMerklePathV2(mpath), payload.VerifyMembership.Path)

					bz, err := json.Marshal(types.EmptyResult{})
					suite.Require().NoError(err)

					expClientStateBz = wasmtesting.CreateMockClientStateBz(suite.chainA.Codec, suite.checksum)
					store.Set(host.ClientStateKey(), expClientStateBz)

					return &wasmvmtypes.ContractResult{Ok: &wasmvmtypes.Response{Data: bz}}, wasmtesting.DefaultGasUsed, nil
				})
			},
			nil,
		},
		{
			"contract returns invalid proof error",
			func() {
				proof = wasmtesting.MockInvalidProofBz

				suite.mockVM.RegisterSudoCallback(types.VerifyMembershipMsg{}, func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, _ wasmvm.KVStore,
					_ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction,
				) (*wasmvmtypes.ContractResult, uint64, error) {
					return &wasmvmtypes.ContractResult{Err: commitmenttypes.ErrInvalidProof.Error()}, wasmtesting.DefaultGasUsed, nil
				})
			},
			types.ErrWasmContractCallFailed,
		},
		{
			"proof height greater than client state latest height",
			func() {
				proofHeight = clienttypes.NewHeight(1, 100)
			},
			ibcerrors.ErrInvalidHeight,
		},
		{
			"invalid path argument",
			func() {
				path = mock.KeyPath{}
			},
			ibcerrors.ErrInvalidType,
		},
		{
			"proof height is invalid type",
			func() {
				proofHeight = mock.Height{}
			},
			ibcerrors.ErrInvalidType,
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			suite.SetupWasmWithMockVM()

			endpoint := wasmtesting.NewWasmEndpoint(suite.chainA)
			err := endpoint.CreateClient()
			suite.Require().NoError(err)

			path = commitmenttypes.NewMerklePath("/ibc/key/path")
			proof = wasmtesting.MockValidProofBz
			proofHeight = clienttypes.NewHeight(0, 1)
			value = []byte("value")

			clientStore := suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(suite.chainA.GetContext(), endpoint.ClientID)
			clientState = endpoint.GetClientState()

			tc.malleate()

			err = clientState.VerifyMembership(suite.chainA.GetContext(), clientStore, suite.chainA.Codec, proofHeight, 0, 0, proof, path, value)

			expPass := tc.expError == nil
			if expPass {
				suite.Require().NoError(err)

				clientStateBz := clientStore.Get(host.ClientStateKey())
				suite.Require().Equal(expClientStateBz, clientStateBz)
			} else {
				suite.Require().ErrorIs(err, tc.expError, "unexpected error in VerifyMembership")
			}
		})
	}
}

func (suite *TypesTestSuite) TestVerifyNonMembership() {
	var (
		clientState      exported.ClientState
		expClientStateBz []byte
		path             exported.Path
		proof            []byte
		proofHeight      exported.Height
	)

	testCases := []struct {
		name     string
		malleate func()
		expError error
	}{
		{
			"success",
			func() {
				expClientStateBz = GetSimApp(suite.chainA).GetIBCKeeper().ClientKeeper.MustMarshalClientState(clientState)
				suite.mockVM.RegisterSudoCallback(types.VerifyNonMembershipMsg{}, func(_ wasmvm.Checksum, _ wasmvmtypes.Env, sudoMsg []byte, _ wasmvm.KVStore,
					_ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction,
				) (*wasmvmtypes.ContractResult, uint64, error) {
					var payload types.SudoMsg
					err := json.Unmarshal(sudoMsg, &payload)
					suite.Require().NoError(err)

					suite.Require().NotNil(payload.VerifyNonMembership)
					suite.Require().Nil(payload.UpdateState)
					suite.Require().Nil(payload.UpdateStateOnMisbehaviour)
					suite.Require().Nil(payload.VerifyMembership)
					suite.Require().Nil(payload.VerifyUpgradeAndUpdateState)
					suite.Require().Equal(proofHeight, payload.VerifyNonMembership.Height)
					suite.Require().Equal(proof, payload.VerifyNonMembership.Proof)

					mpath, ok := path.(commitmenttypes.MerklePath)
					suite.Require().True(ok)
					// assert that merkle path key path is unmarshalled as bytes rather than strings
					suite.Require().Equal(internaltypes.ToMerklePathV2(mpath), payload.VerifyNonMembership.Path)

					bz, err := json.Marshal(types.EmptyResult{})
					suite.Require().NoError(err)

					return &wasmvmtypes.ContractResult{Ok: &wasmvmtypes.Response{Data: bz}}, wasmtesting.DefaultGasUsed, nil
				})
			},
			nil,
		},
		{
			"success: with update client state",
			func() {
				suite.mockVM.RegisterSudoCallback(types.VerifyNonMembershipMsg{}, func(_ wasmvm.Checksum, _ wasmvmtypes.Env, sudoMsg []byte, store wasmvm.KVStore,
					_ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction,
				) (*wasmvmtypes.ContractResult, uint64, error) {
					var payload types.SudoMsg
					err := json.Unmarshal(sudoMsg, &payload)
					suite.Require().NoError(err)

					suite.Require().NotNil(payload.VerifyNonMembership)
					suite.Require().Nil(payload.UpdateState)
					suite.Require().Nil(payload.UpdateStateOnMisbehaviour)
					suite.Require().Nil(payload.VerifyMembership)
					suite.Require().Nil(payload.VerifyUpgradeAndUpdateState)
					suite.Require().Equal(proofHeight, payload.VerifyNonMembership.Height)
					suite.Require().Equal(proof, payload.VerifyNonMembership.Proof)

					mpath, ok := path.(commitmenttypes.MerklePath)
					suite.Require().True(ok)
					// assert that merkle path key path is unmarshalled as bytes rather than strings
					suite.Require().Equal(internaltypes.ToMerklePathV2(mpath), payload.VerifyNonMembership.Path)

					bz, err := json.Marshal(types.EmptyResult{})
					suite.Require().NoError(err)

					expClientStateBz = wasmtesting.CreateMockClientStateBz(suite.chainA.Codec, suite.checksum)
					store.Set(host.ClientStateKey(), expClientStateBz)

					return &wasmvmtypes.ContractResult{Ok: &wasmvmtypes.Response{Data: bz}}, wasmtesting.DefaultGasUsed, nil
				})
			},
			nil,
		},
		{
			"wasm vm returns error",
			func() {
				proof = wasmtesting.MockInvalidProofBz

				suite.mockVM.RegisterSudoCallback(types.VerifyNonMembershipMsg{}, func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, _ wasmvm.KVStore,
					_ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction,
				) (*wasmvmtypes.ContractResult, uint64, error) {
					return nil, wasmtesting.DefaultGasUsed, wasmtesting.ErrMockVM
				})
			},
			types.ErrVMError,
		},
		{
			"contract returns invalid proof error",
			func() {
				proof = wasmtesting.MockInvalidProofBz

				suite.mockVM.RegisterSudoCallback(types.VerifyNonMembershipMsg{}, func(_ wasmvm.Checksum, _ wasmvmtypes.Env, _ []byte, _ wasmvm.KVStore,
					_ wasmvm.GoAPI, _ wasmvm.Querier, _ wasmvm.GasMeter, _ uint64, _ wasmvmtypes.UFraction,
				) (*wasmvmtypes.ContractResult, uint64, error) {
					return &wasmvmtypes.ContractResult{Err: commitmenttypes.ErrInvalidProof.Error()}, wasmtesting.DefaultGasUsed, nil
				})
			},
			types.ErrWasmContractCallFailed,
		},
		{
			"proof height greater than client state latest height",
			func() {
				proofHeight = clienttypes.NewHeight(1, 100)
			},
			ibcerrors.ErrInvalidHeight,
		},
		{
			"invalid path argument",
			func() {
				path = mock.KeyPath{}
			},
			ibcerrors.ErrInvalidType,
		},
		{
			"proof height is invalid type",
			func() {
				proofHeight = mock.Height{}
			},
			ibcerrors.ErrInvalidType,
		},
	}

	for _, tc := range testCases {
		suite.Run(tc.name, func() {
			suite.SetupWasmWithMockVM()

			endpoint := wasmtesting.NewWasmEndpoint(suite.chainA)
			err := endpoint.CreateClient()
			suite.Require().NoError(err)

			path = commitmenttypes.NewMerklePath("/ibc/key/path")
			proof = wasmtesting.MockInvalidProofBz
			proofHeight = clienttypes.NewHeight(0, 1)

			clientStore := suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(suite.chainA.GetContext(), endpoint.ClientID)
			clientState = endpoint.GetClientState()

			tc.malleate()

			err = clientState.VerifyNonMembership(suite.chainA.GetContext(), clientStore, suite.chainA.Codec, proofHeight, 0, 0, proof, path)

			expPass := tc.expError == nil
			if expPass {
				suite.Require().NoError(err)

				clientStateBz := clientStore.Get(host.ClientStateKey())
				suite.Require().Equal(expClientStateBz, clientStateBz)
			} else {
				suite.Require().ErrorIs(err, tc.expError, "unexpected error in VerifyNonMembership")
			}
		})
	}
}
