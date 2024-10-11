package v7_test

import (
	"strconv"
	"testing"

	testifysuite "github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/runtime"

	v7 "github.com/cosmos/ibc-go/v8/modules/core/02-client/migrations/v7"
	"github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibcexported "github.com/cosmos/ibc-go/v8/modules/core/exported"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

// numCreations is the number of clients/consensus states created for
// solo machine and localhost clients
const numCreations = 10

type MigrationsV7TestSuite struct {
	testifysuite.Suite

	coordinator *ibctesting.Coordinator

	chainA *ibctesting.TestChain
	chainB *ibctesting.TestChain
}

func (suite *MigrationsV7TestSuite) SetupTest() {
	suite.coordinator = ibctesting.NewCoordinator(suite.T(), 2)

	suite.chainA = suite.coordinator.GetChain(ibctesting.GetChainID(1))
	suite.chainB = suite.coordinator.GetChain(ibctesting.GetChainID(2))
}

func TestIBCTestSuite(t *testing.T) {
	testifysuite.Run(t, new(MigrationsV7TestSuite))
}

// create multiple solo machine clients, tendermint and localhost clients
// ensure that solo machine clients are migrated and their consensus states are removed
// ensure the localhost is deleted entirely.
func (suite *MigrationsV7TestSuite) TestMigrateStore() {
	paths := []*ibctesting.Path{
		ibctesting.NewPath(suite.chainA, suite.chainB),
		ibctesting.NewPath(suite.chainA, suite.chainB),
	}

	// create tendermint clients
	for _, path := range paths {
		suite.coordinator.SetupClients(path)
	}

	solomachines := []*ibctesting.Solomachine{
		ibctesting.NewSolomachine(suite.T(), suite.chainA.Codec, ibctesting.DefaultSolomachineClientID, "testing", 1),
		ibctesting.NewSolomachine(suite.T(), suite.chainA.Codec, "06-solomachine-1", "testing", 4),
	}

	suite.createSolomachineClients(solomachines)
	suite.createLocalhostClients()

	err := v7.MigrateStore(suite.chainA.GetContext(), runtime.NewKVStoreService(suite.chainA.GetSimApp().GetKey(ibcexported.StoreKey)), suite.chainA.App.AppCodec(), suite.chainA.GetSimApp().IBCKeeper.ClientKeeper)
	suite.Require().NoError(err)

	suite.assertSolomachineClients(solomachines)
	suite.assertNoLocalhostClients()
}

func (suite *MigrationsV7TestSuite) TestMigrateStoreNoTendermintClients() {
	solomachines := []*ibctesting.Solomachine{
		ibctesting.NewSolomachine(suite.T(), suite.chainA.Codec, ibctesting.DefaultSolomachineClientID, "testing", 1),
		ibctesting.NewSolomachine(suite.T(), suite.chainA.Codec, "06-solomachine-1", "testing", 4),
	}

	suite.createSolomachineClients(solomachines)
	suite.createLocalhostClients()

	err := v7.MigrateStore(suite.chainA.GetContext(), runtime.NewKVStoreService(suite.chainA.GetSimApp().GetKey(ibcexported.StoreKey)), suite.chainA.App.AppCodec(), suite.chainA.GetSimApp().IBCKeeper.ClientKeeper)
	suite.Require().NoError(err)

	suite.assertSolomachineClients(solomachines)
	suite.assertNoLocalhostClients()
}

func (suite *MigrationsV7TestSuite) createSolomachineClients(solomachines []*ibctesting.Solomachine) {
	// manually generate old protobuf definitions and set in store
	// NOTE: we cannot use 'CreateClient' and 'UpdateClient' functions since we are
	// using client states and consensus states which do not implement the exported.ClientState
	// and exported.ConsensusState interface
	for _, sm := range solomachines {
		clientStore := suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(suite.chainA.GetContext(), sm.ClientID)
		clientState := sm.ClientState()

		// generate old client state proto definition
		legacyClientState := &v7.ClientState{
			Sequence: clientState.Sequence,
			ConsensusState: &v7.ConsensusState{
				PublicKey:   clientState.ConsensusState.PublicKey,
				Diversifier: clientState.ConsensusState.Diversifier,
				Timestamp:   clientState.ConsensusState.Timestamp,
			},
			AllowUpdateAfterProposal: true,
		}

		cdc := suite.chainA.App.AppCodec().(*codec.ProtoCodec)
		v7.RegisterInterfaces(cdc.InterfaceRegistry())

		bz, err := cdc.MarshalInterface(legacyClientState)
		suite.Require().NoError(err)
		clientStore.Set(host.ClientStateKey(), bz)

		bz, err = cdc.MarshalInterface(legacyClientState.ConsensusState)
		suite.Require().NoError(err)

		// set some consensus states
		for i := uint64(0); i < numCreations; i++ {
			height := types.NewHeight(1, i)
			clientStore.Set(host.ConsensusStateKey(height), bz)
		}

	}
}

func (suite *MigrationsV7TestSuite) assertSolomachineClients(solomachines []*ibctesting.Solomachine) {
	// verify client state has been migrated
	for _, sm := range solomachines {
		clientState, ok := suite.chainA.App.GetIBCKeeper().ClientKeeper.GetClientState(suite.chainA.GetContext(), sm.ClientID)
		suite.Require().True(ok)
		suite.Require().Equal(sm.ClientState(), clientState)

		for i := uint64(0); i < numCreations; i++ {
			height := types.NewHeight(1, i)

			consState, ok := suite.chainA.App.GetIBCKeeper().ClientKeeper.GetClientConsensusState(suite.chainA.GetContext(), sm.ClientID, height)
			suite.Require().False(ok)
			suite.Require().Empty(consState)
		}
	}
}

// createLocalhostClients clients creates multiple localhost clients and multiple consensus states for each
func (suite *MigrationsV7TestSuite) createLocalhostClients() {
	for numClients := uint64(0); numClients < numCreations; numClients++ {
		clientID := v7.Localhost + "-" + strconv.FormatUint(numClients, 10)
		clientStore := suite.chainA.GetSimApp().IBCKeeper.ClientKeeper.ClientStore(suite.chainA.GetContext(), clientID)

		clientStore.Set(host.ClientStateKey(), []byte("clientState"))

		for i := 0; i < numCreations; i++ {
			clientStore.Set(host.ConsensusStateKey(types.NewHeight(1, uint64(i))), []byte("consensusState"))
		}
	}
}

// assertLocalhostClients asserts that all localhost information has been deleted
func (suite *MigrationsV7TestSuite) assertNoLocalhostClients() {
	for numClients := uint64(0); numClients < numCreations; numClients++ {
		clientID := v7.Localhost + "-" + strconv.FormatUint(numClients, 10)
		clientStore := suite.chainA.GetSimApp().IBCKeeper.ClientKeeper.ClientStore(suite.chainA.GetContext(), clientID)

		suite.Require().False(clientStore.Has(host.ClientStateKey()))

		for i := uint64(0); i < numCreations; i++ {
			suite.Require().False(clientStore.Has(host.ConsensusStateKey(types.NewHeight(1, i))))
		}
	}
}
