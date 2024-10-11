package keeper_test

import (
	"encoding/json"
	"errors"
	"testing"

	wasmvm "github.com/CosmWasm/wasmvm/v2"
	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"
	dbm "github.com/cosmos/cosmos-db"
	testifysuite "github.com/stretchr/testify/suite"

	"cosmossdk.io/log"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/runtime"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	govtypes "cosmossdk.io/x/gov/types"

	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/internal/ibcwasm"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/keeper"
	wasmtesting "github.com/cosmos/ibc-go/modules/light-clients/08-wasm/testing"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/testing/simapp"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

const (
	defaultWasmClientID = "08-wasm-0"
)

type KeeperTestSuite struct {
	testifysuite.Suite

	coordinator *ibctesting.Coordinator

	// mockVM is a mock wasm VM that implements the WasmEngine interface
	mockVM *wasmtesting.MockWasmEngine
	chainA *ibctesting.TestChain
}

func init() {
	ibctesting.DefaultTestingAppInit = setupTestingApp
}

// setupTestingApp provides the duplicated simapp which is specific to the 08-wasm module on chain creation.
func setupTestingApp() (ibctesting.TestingApp, map[string]json.RawMessage) {
	db := dbm.NewMemDB()
	app := simapp.NewSimApp(log.NewNopLogger(), db, nil, true, simtestutil.EmptyAppOptions{}, nil)
	return app, app.DefaultGenesis()
}

// GetSimApp returns the duplicated SimApp from within the 08-wasm directory.
// This must be used instead of chain.GetSimApp() for tests within this directory.
func GetSimApp(chain *ibctesting.TestChain) *simapp.SimApp {
	app, ok := chain.App.(*simapp.SimApp)
	if !ok {
		panic(errors.New("chain is not a simapp.SimApp"))
	}
	return app
}

func (suite *KeeperTestSuite) SetupTest() {
	suite.coordinator = ibctesting.NewCoordinator(suite.T(), 1)
	suite.chainA = suite.coordinator.GetChain(ibctesting.GetChainID(1))

	queryHelper := baseapp.NewQueryServerTestHelper(suite.chainA.GetContext(), GetSimApp(suite.chainA).InterfaceRegistry())
	types.RegisterQueryServer(queryHelper, GetSimApp(suite.chainA).WasmClientKeeper)
}

// SetupWasmWithMockVM sets up mock cometbft chain with a mock vm.
func (suite *KeeperTestSuite) SetupWasmWithMockVM() {
	ibctesting.DefaultTestingAppInit = suite.setupWasmWithMockVM

	suite.coordinator = ibctesting.NewCoordinator(suite.T(), 1)
	suite.chainA = suite.coordinator.GetChain(ibctesting.GetChainID(1))

	wasmtesting.AllowWasmClients(suite.chainA)
}

func (suite *KeeperTestSuite) setupWasmWithMockVM() (ibctesting.TestingApp, map[string]json.RawMessage) {
	suite.mockVM = wasmtesting.NewMockWasmEngine()

	suite.mockVM.InstantiateFn = func(checksum wasmvm.Checksum, env wasmvmtypes.Env, info wasmvmtypes.MessageInfo, initMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
		var payload types.InstantiateMessage
		err := json.Unmarshal(initMsg, &payload)
		suite.Require().NoError(err)

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

	suite.mockVM.RegisterQueryCallback(types.StatusMsg{}, func(checksum wasmvm.Checksum, env wasmvmtypes.Env, queryMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.QueryResult, uint64, error) {
		resp, err := json.Marshal(types.StatusResult{Status: exported.Active.String()})
		suite.Require().NoError(err)
		return &wasmvmtypes.QueryResult{Ok: resp}, wasmtesting.DefaultGasUsed, nil
	})

	db := dbm.NewMemDB()
	app := simapp.NewSimApp(log.NewNopLogger(), db, nil, true, simtestutil.EmptyAppOptions{}, suite.mockVM)

	// reset DefaultTestingAppInit to its original value
	ibctesting.DefaultTestingAppInit = setupTestingApp
	return app, app.DefaultGenesis()
}

// storeWasmCode stores the wasm code on chain and returns the checksum.
func storeWasmCode(suite *KeeperTestSuite, wasmCode []byte) []byte {
	ctx := suite.chainA.GetContext().WithBlockGasMeter(storetypes.NewInfiniteGasMeter())

	msg := types.NewMsgStoreCode(authtypes.NewModuleAddress(govtypes.ModuleName).String(), wasmCode)
	response, err := GetSimApp(suite.chainA).WasmClientKeeper.StoreCode(ctx, msg)
	suite.Require().NoError(err)
	suite.Require().NotNil(response.Checksum)
	return response.Checksum
}

func (suite *KeeperTestSuite) SetupSnapshotterWithMockVM() *simapp.SimApp {
	suite.mockVM = wasmtesting.NewMockWasmEngine()

	return simapp.SetupWithSnapshotter(suite.T(), suite.mockVM)
}

func TestKeeperTestSuite(t *testing.T) {
	testifysuite.Run(t, new(KeeperTestSuite))
}

func (suite *KeeperTestSuite) TestNewKeeper() {
	testCases := []struct {
		name          string
		instantiateFn func()
		expPass       bool
		expError      error
	}{
		{
			"success",
			func() {
				keeper.NewKeeperWithVM(
					GetSimApp(suite.chainA).AppCodec(),
					runtime.NewKVStoreService(GetSimApp(suite.chainA).GetKey(types.StoreKey)),
					GetSimApp(suite.chainA).IBCKeeper.ClientKeeper,
					GetSimApp(suite.chainA).WasmClientKeeper.GetAuthority(),
					ibcwasm.GetVM(),
					GetSimApp(suite.chainA).GRPCQueryRouter(),
				)
			},
			true,
			nil,
		},
		{
			"failure: empty authority",
			func() {
				keeper.NewKeeperWithVM(
					GetSimApp(suite.chainA).AppCodec(),
					runtime.NewKVStoreService(GetSimApp(suite.chainA).GetKey(types.StoreKey)),
					GetSimApp(suite.chainA).IBCKeeper.ClientKeeper,
					"", // authority
					ibcwasm.GetVM(),
					GetSimApp(suite.chainA).GRPCQueryRouter(),
				)
			},
			false,
			errors.New("authority must be non-empty"),
		},
		{
			"failure: nil client keeper",
			func() {
				keeper.NewKeeperWithVM(
					GetSimApp(suite.chainA).AppCodec(),
					runtime.NewKVStoreService(GetSimApp(suite.chainA).GetKey(types.StoreKey)),
					nil, // client keeper,
					GetSimApp(suite.chainA).WasmClientKeeper.GetAuthority(),
					ibcwasm.GetVM(),
					GetSimApp(suite.chainA).GRPCQueryRouter(),
				)
			},
			false,
			errors.New("client keeper must not be nil"),
		},
		{
			"failure: nil wasm VM",
			func() {
				keeper.NewKeeperWithVM(
					GetSimApp(suite.chainA).AppCodec(),
					runtime.NewKVStoreService(GetSimApp(suite.chainA).GetKey(types.StoreKey)),
					GetSimApp(suite.chainA).IBCKeeper.ClientKeeper,
					GetSimApp(suite.chainA).WasmClientKeeper.GetAuthority(),
					nil,
					GetSimApp(suite.chainA).GRPCQueryRouter(),
				)
			},
			false,
			errors.New("wasm VM must not be nil"),
		},
		{
			"failure: nil store service",
			func() {
				keeper.NewKeeperWithVM(
					GetSimApp(suite.chainA).AppCodec(),
					nil,
					GetSimApp(suite.chainA).IBCKeeper.ClientKeeper,
					GetSimApp(suite.chainA).WasmClientKeeper.GetAuthority(),
					ibcwasm.GetVM(),
					GetSimApp(suite.chainA).GRPCQueryRouter(),
				)
			},
			false,
			errors.New("store service must not be nil"),
		},
		{
			"failure: nil query router",
			func() {
				keeper.NewKeeperWithVM(
					GetSimApp(suite.chainA).AppCodec(),
					runtime.NewKVStoreService(GetSimApp(suite.chainA).GetKey(types.StoreKey)),
					GetSimApp(suite.chainA).IBCKeeper.ClientKeeper,
					GetSimApp(suite.chainA).WasmClientKeeper.GetAuthority(),
					ibcwasm.GetVM(),
					nil,
				)
			},
			false,
			errors.New("query router must not be nil"),
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.SetupTest()

		suite.Run(tc.name, func() {
			if tc.expPass {
				suite.Require().NotPanics(
					tc.instantiateFn,
				)
			} else {
				suite.Require().PanicsWithError(tc.expError.Error(), func() {
					tc.instantiateFn()
				})
			}
		})
	}
}

func (suite *KeeperTestSuite) TestInitializedPinnedCodes() {
	var capturedChecksums []wasmvm.Checksum

	testCases := []struct {
		name     string
		malleate func()
		expError error
	}{
		{
			"success",
			func() {
				suite.mockVM.PinFn = func(checksum wasmvm.Checksum) error {
					capturedChecksums = append(capturedChecksums, checksum)
					return nil
				}
			},
			nil,
		},
		{
			"failure: pin error",
			func() {
				suite.mockVM.PinFn = func(checksum wasmvm.Checksum) error {
					return wasmtesting.ErrMockVM
				}
			},
			wasmtesting.ErrMockVM,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupWasmWithMockVM()

			ctx := suite.chainA.GetContext()
			wasmClientKeeper := GetSimApp(suite.chainA).WasmClientKeeper

			contracts := [][]byte{wasmtesting.Code, wasmtesting.CreateMockContract([]byte("gzipped-contract"))}
			checksumIDs := make([]types.Checksum, len(contracts))
			signer := authtypes.NewModuleAddress(govtypes.ModuleName).String()

			// store contract on chain
			for i, contract := range contracts {
				msg := types.NewMsgStoreCode(signer, contract)

				res, err := wasmClientKeeper.StoreCode(ctx, msg)
				suite.Require().NoError(err)

				checksumIDs[i] = res.Checksum
			}

			// malleate after storing contracts
			tc.malleate()

			err := keeper.InitializePinnedCodes(ctx)

			expPass := tc.expError == nil
			if expPass {
				suite.Require().NoError(err)
				suite.ElementsMatch(checksumIDs, capturedChecksums)
			} else {
				suite.Require().ErrorIs(err, tc.expError)
			}
		})
	}
}
