package v7_test

import (
	"testing"

	testifysuite "github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/runtime"
	genutiltypes "github.com/cosmos/cosmos-sdk/x/genutil/types"

	ibcclient "github.com/cosmos/ibc-go/v8/modules/core/02-client"
	clientv7 "github.com/cosmos/ibc-go/v8/modules/core/02-client/migrations/v7"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibcexported "github.com/cosmos/ibc-go/v8/modules/core/exported"
	v7 "github.com/cosmos/ibc-go/v8/modules/core/migrations/v7"
	"github.com/cosmos/ibc-go/v8/modules/core/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

type MigrationsV7TestSuite struct {
	testifysuite.Suite

	coordinator *ibctesting.Coordinator

	// testing chains used for convenience and readability
	chainA *ibctesting.TestChain
	chainB *ibctesting.TestChain
}

// TestMigrationsV7TestSuite runs all the tests within this package.
func TestMigrationsV7TestSuite(t *testing.T) {
	testifysuite.Run(t, new(MigrationsV7TestSuite))
}

// SetupTest creates a coordinator with 2 test chains.
func (suite *MigrationsV7TestSuite) SetupTest() {
	suite.coordinator = ibctesting.NewCoordinator(suite.T(), 2)
	suite.chainA = suite.coordinator.GetChain(ibctesting.GetChainID(1))
	suite.chainB = suite.coordinator.GetChain(ibctesting.GetChainID(2))
}

// NOTE: this test is mainly copied from 02-client/migrations/v7/genesis_test.go
func (suite *MigrationsV7TestSuite) TestMigrateGenesisSolomachine() {
	// create tendermint clients
	for i := 0; i < 3; i++ {
		path := ibctesting.NewPath(suite.chainA, suite.chainB)

		suite.coordinator.SetupClients(path)

		err := path.EndpointA.UpdateClient()
		suite.Require().NoError(err)

		// update a second time to add more state
		err = path.EndpointA.UpdateClient()
		suite.Require().NoError(err)
	}

	// create multiple legacy solo machine clients
	solomachine := ibctesting.NewSolomachine(suite.T(), suite.chainA.Codec, ibctesting.DefaultSolomachineClientID, "testing", 1)
	solomachineMulti := ibctesting.NewSolomachine(suite.T(), suite.chainA.Codec, "06-solomachine-1", "testing", 4)

	clientGenState, err := ibcclient.ExportGenesis(suite.chainA.GetContext(), suite.chainA.App.GetIBCKeeper().ClientKeeper)
	suite.Require().NoError(err)

	// manually generate old proto buf definitions and set in genesis
	// NOTE: we cannot use 'ExportGenesis' for the solo machines since we are
	// using client states and consensus states which do not implement the exported.ClientState
	// and exported.ConsensusState interface
	var clients []clienttypes.IdentifiedClientState
	for _, sm := range []*ibctesting.Solomachine{solomachine, solomachineMulti} {
		clientState := sm.ClientState()

		// generate old client state proto definition
		legacyClientState := &clientv7.ClientState{
			Sequence: clientState.Sequence,
			ConsensusState: &clientv7.ConsensusState{
				PublicKey:   clientState.ConsensusState.PublicKey,
				Diversifier: clientState.ConsensusState.Diversifier,
				Timestamp:   clientState.ConsensusState.Timestamp,
			},
			AllowUpdateAfterProposal: true,
		}

		// set client state
		protoAny, err := codectypes.NewAnyWithValue(legacyClientState)
		suite.Require().NoError(err)
		suite.Require().NotNil(protoAny)

		clients = append(clients, clienttypes.IdentifiedClientState{
			ClientId:    sm.ClientID,
			ClientState: protoAny,
		})

		// set in store for ease of determining expected genesis
		clientStore := suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(suite.chainA.GetContext(), sm.ClientID)
		cdc := suite.chainA.App.AppCodec().(*codec.ProtoCodec)
		clientv7.RegisterInterfaces(cdc.InterfaceRegistry())

		bz, err := cdc.MarshalInterface(legacyClientState)
		suite.Require().NoError(err)
		clientStore.Set(host.ClientStateKey(), bz)

		protoAny, err = codectypes.NewAnyWithValue(legacyClientState.ConsensusState)
		suite.Require().NoError(err)
		suite.Require().NotNil(protoAny)

		// obtain marshalled bytes to set in client store
		bz, err = cdc.MarshalInterface(legacyClientState.ConsensusState)
		suite.Require().NoError(err)

		var consensusStates []clienttypes.ConsensusStateWithHeight

		// set consensus states in store and genesis
		for i := uint64(0); i < 10; i++ {
			height := clienttypes.NewHeight(1, i)
			clientStore.Set(host.ConsensusStateKey(height), bz)
			consensusStates = append(consensusStates, clienttypes.ConsensusStateWithHeight{
				Height:         height,
				ConsensusState: protoAny,
			})
		}

		clientGenState.ClientsConsensus = append(clientGenState.ClientsConsensus, clienttypes.ClientConsensusStates{
			ClientId:        sm.ClientID,
			ConsensusStates: consensusStates,
		})
	}

	// solo machine clients must come before tendermint in expected
	clientGenState.Clients = append(clients, clientGenState.Clients...)

	// migrate store get expected genesis
	// store migration and genesis migration should produce identical results
	// NOTE: tendermint clients are not pruned in genesis so the test should not have expired tendermint clients
	err = clientv7.MigrateStore(suite.chainA.GetContext(), runtime.NewKVStoreService(suite.chainA.GetSimApp().GetKey(ibcexported.StoreKey)), suite.chainA.App.AppCodec(), suite.chainA.GetSimApp().IBCKeeper.ClientKeeper)
	suite.Require().NoError(err)
	expectedClientGenState, err := ibcclient.ExportGenesis(suite.chainA.GetContext(), suite.chainA.App.GetIBCKeeper().ClientKeeper)
	suite.Require().NoError(err)

	cdc := suite.chainA.App.AppCodec().(*codec.ProtoCodec)

	// NOTE: these lines are added in comparison to 02-client/migrations/v7/genesis_test.go
	// generate appState with old ibc genesis state
	appState := genutiltypes.AppMap{}
	ibcGenState := types.DefaultGenesisState()
	ibcGenState.ClientGenesis = clientGenState

	// ensure tests pass even if the legacy solo machine is already registered
	clientv7.RegisterInterfaces(cdc.InterfaceRegistry())
	appState[ibcexported.ModuleName] = cdc.MustMarshalJSON(ibcGenState)

	// NOTE: genesis time isn't updated since we aren't testing for tendermint consensus state pruning
	migrated, err := v7.MigrateGenesis(appState, cdc)
	suite.Require().NoError(err)

	expectedAppState := genutiltypes.AppMap{}
	expectedIBCGenState := types.DefaultGenesisState()
	expectedIBCGenState.ClientGenesis = expectedClientGenState

	bz, err := cdc.MarshalJSON(expectedIBCGenState)
	suite.Require().NoError(err)
	expectedAppState[ibcexported.ModuleName] = bz

	suite.Require().Equal(expectedAppState, migrated)
}
