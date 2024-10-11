package v7_test

import (
	"encoding/json"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/runtime"

	ibcclient "github.com/cosmos/ibc-go/v8/modules/core/02-client"
	v7 "github.com/cosmos/ibc-go/v8/modules/core/02-client/migrations/v7"
	"github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibcexported "github.com/cosmos/ibc-go/v8/modules/core/exported"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

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
	var clients []types.IdentifiedClientState
	for _, sm := range []*ibctesting.Solomachine{solomachine, solomachineMulti} {
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

		// set client state
		protoAny, err := codectypes.NewAnyWithValue(legacyClientState)
		suite.Require().NoError(err)
		suite.Require().NotNil(protoAny)

		clients = append(clients, types.IdentifiedClientState{
			ClientId:    sm.ClientID,
			ClientState: protoAny,
		})

		// set in store for ease of determining expected genesis
		clientStore := suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(suite.chainA.GetContext(), sm.ClientID)

		cdc := suite.chainA.App.AppCodec().(*codec.ProtoCodec)
		v7.RegisterInterfaces(cdc.InterfaceRegistry())

		bz, err := cdc.MarshalInterface(legacyClientState)
		suite.Require().NoError(err)
		clientStore.Set(host.ClientStateKey(), bz)

		protoAny, err = codectypes.NewAnyWithValue(legacyClientState.ConsensusState)
		suite.Require().NoError(err)
		suite.Require().NotNil(protoAny)

		// obtain marshalled bytes to set in client store
		bz, err = cdc.MarshalInterface(legacyClientState.ConsensusState)
		suite.Require().NoError(err)

		var consensusStates []types.ConsensusStateWithHeight

		// set consensus states in store and genesis
		for i := uint64(0); i < numCreations; i++ {
			height := types.NewHeight(1, i)
			clientStore.Set(host.ConsensusStateKey(height), bz)
			consensusStates = append(consensusStates, types.ConsensusStateWithHeight{
				Height:         height,
				ConsensusState: protoAny,
			})
		}

		clientGenState.ClientsConsensus = append(clientGenState.ClientsConsensus, types.ClientConsensusStates{
			ClientId:        sm.ClientID,
			ConsensusStates: consensusStates,
		})
	}

	// solo machine clients must come before tendermint in expected
	clientGenState.Clients = append(clients, clientGenState.Clients...)

	// migrate store get expected genesis
	// store migration and genesis migration should produce identical results
	// NOTE: tendermint clients are not pruned in genesis so the test should not have expired tendermint clients
	err = v7.MigrateStore(suite.chainA.GetContext(), runtime.NewKVStoreService(suite.chainA.GetSimApp().GetKey(ibcexported.StoreKey)), suite.chainA.App.AppCodec(), suite.chainA.GetSimApp().IBCKeeper.ClientKeeper)
	suite.Require().NoError(err)
	expectedClientGenState, err := ibcclient.ExportGenesis(suite.chainA.GetContext(), suite.chainA.App.GetIBCKeeper().ClientKeeper)
	suite.Require().NoError(err)

	cdc, ok := suite.chainA.App.AppCodec().(codec.ProtoCodecMarshaler)
	suite.Require().True(ok)

	migrated, err := v7.MigrateGenesis(&clientGenState, cdc)
	suite.Require().NoError(err)

	bz, err := cdc.MarshalJSON(&expectedClientGenState)
	suite.Require().NoError(err)

	// Indent the JSON bz correctly.
	var jsonObj map[string]interface{}
	err = json.Unmarshal(bz, &jsonObj)
	suite.Require().NoError(err)
	expectedIndentedBz, err := json.MarshalIndent(jsonObj, "", "\t")
	suite.Require().NoError(err)

	bz, err = cdc.MarshalJSON(migrated)
	suite.Require().NoError(err)

	// Indent the JSON bz correctly.
	err = json.Unmarshal(bz, &jsonObj)
	suite.Require().NoError(err)
	indentedBz, err := json.MarshalIndent(jsonObj, "", "\t")
	suite.Require().NoError(err)

	suite.Require().Equal(string(expectedIndentedBz), string(indentedBz))
}
