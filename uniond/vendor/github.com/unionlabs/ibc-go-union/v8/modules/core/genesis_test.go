package ibc_test

import (
	"fmt"
	"testing"

	testifysuite "github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/codec"

	ibc "github.com/cosmos/ibc-go/v8/modules/core"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	"github.com/cosmos/ibc-go/v8/modules/core/types"
	ibctm "github.com/cosmos/ibc-go/v8/modules/light-clients/07-tendermint"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
	"github.com/cosmos/ibc-go/v8/testing/simapp"
)

const (
	connectionID  = "connection-0"
	clientID      = "07-tendermint-0"
	connectionID2 = "connection-1"
	clientID2     = "07-tendermint-1"

	port1 = "firstport"
	port2 = "secondport"

	channel1 = "channel-0"
	channel2 = "channel-1"
)

var clientHeight = clienttypes.NewHeight(1, 10)

type IBCTestSuite struct {
	testifysuite.Suite

	coordinator *ibctesting.Coordinator

	chainA *ibctesting.TestChain
	chainB *ibctesting.TestChain
}

// SetupTest creates a coordinator with 2 test chains.
func (suite *IBCTestSuite) SetupTest() {
	suite.coordinator = ibctesting.NewCoordinator(suite.T(), 2)

	suite.chainA = suite.coordinator.GetChain(ibctesting.GetChainID(1))
	suite.chainB = suite.coordinator.GetChain(ibctesting.GetChainID(2))
}

func TestIBCTestSuite(t *testing.T) {
	testifysuite.Run(t, new(IBCTestSuite))
}

func (suite *IBCTestSuite) TestValidateGenesis() {
	header := suite.chainA.CreateTMClientHeader(suite.chainA.ChainID, suite.chainA.CurrentHeader.Height, clienttypes.NewHeight(0, uint64(suite.chainA.CurrentHeader.Height-1)), suite.chainA.CurrentHeader.Time, suite.chainA.Vals, suite.chainA.Vals, suite.chainA.Vals, suite.chainA.Signers)

	testCases := []struct {
		name     string
		genState *types.GenesisState
		expPass  bool
	}{
		{
			name:     "default",
			genState: types.DefaultGenesisState(),
			expPass:  true,
		},
		{
			name: "valid genesis",
			genState: &types.GenesisState{
				ClientGenesis: clienttypes.NewGenesisState(
					[]clienttypes.IdentifiedClientState{
						clienttypes.NewIdentifiedClientState(
							clientID, ibctm.NewClientState(suite.chainA.ChainID, ibctm.DefaultTrustLevel, ibctesting.TrustingPeriod, ibctesting.UnbondingPeriod, ibctesting.MaxClockDrift, clientHeight, commitmenttypes.GetSDKSpecs(), ibctesting.UpgradePath),
						),
					},
					[]clienttypes.ClientConsensusStates{
						clienttypes.NewClientConsensusStates(
							clientID,
							[]clienttypes.ConsensusStateWithHeight{
								clienttypes.NewConsensusStateWithHeight(
									header.GetHeight().(clienttypes.Height),
									ibctm.NewConsensusState(
										header.GetTime(), commitmenttypes.NewMerkleRoot(header.Header.AppHash), header.Header.NextValidatorsHash,
									),
								),
							},
						),
					},
					[]clienttypes.IdentifiedGenesisMetadata{
						clienttypes.NewIdentifiedGenesisMetadata(
							clientID,
							[]clienttypes.GenesisMetadata{
								clienttypes.NewGenesisMetadata([]byte("key1"), []byte("val1")),
								clienttypes.NewGenesisMetadata([]byte("key2"), []byte("val2")),
							},
						),
					},
					clienttypes.NewParams(exported.Tendermint),
					false,
					2,
				),
				ConnectionGenesis: connectiontypes.NewGenesisState(
					[]connectiontypes.IdentifiedConnection{
						connectiontypes.NewIdentifiedConnection(connectionID, connectiontypes.NewConnectionEnd(connectiontypes.INIT, clientID, connectiontypes.NewCounterparty(clientID2, connectionID2, commitmenttypes.NewMerklePrefix([]byte("prefix"))), []*connectiontypes.Version{ibctesting.ConnectionVersion}, 0)),
					},
					[]connectiontypes.ConnectionPaths{
						connectiontypes.NewConnectionPaths(clientID, []string{connectionID}),
					},
					0,
					connectiontypes.NewParams(10),
				),
				ChannelGenesis: channeltypes.NewGenesisState(
					[]channeltypes.IdentifiedChannel{
						channeltypes.NewIdentifiedChannel(
							port1, channel1, channeltypes.NewChannel(
								channeltypes.INIT, channeltypes.ORDERED,
								channeltypes.NewCounterparty(port2, channel2), []string{connectionID}, ibctesting.DefaultChannelVersion,
							),
						),
					},
					[]channeltypes.PacketState{
						channeltypes.NewPacketState(port2, channel2, 1, []byte("ack")),
					},
					[]channeltypes.PacketState{
						channeltypes.NewPacketState(port2, channel2, 1, []byte("")),
					},
					[]channeltypes.PacketState{
						channeltypes.NewPacketState(port1, channel1, 1, []byte("commit_hash")),
					},
					[]channeltypes.PacketSequence{
						channeltypes.NewPacketSequence(port1, channel1, 1),
					},
					[]channeltypes.PacketSequence{
						channeltypes.NewPacketSequence(port2, channel2, 1),
					},
					[]channeltypes.PacketSequence{
						channeltypes.NewPacketSequence(port2, channel2, 1),
					},
					0,
				),
			},
			expPass: true,
		},
		{
			name: "invalid client genesis",
			genState: &types.GenesisState{
				ClientGenesis: clienttypes.NewGenesisState(
					[]clienttypes.IdentifiedClientState{
						clienttypes.NewIdentifiedClientState(
							clientID, ibctm.NewClientState(suite.chainA.ChainID, ibctm.DefaultTrustLevel, ibctesting.TrustingPeriod, ibctesting.UnbondingPeriod, ibctesting.MaxClockDrift, clientHeight, commitmenttypes.GetSDKSpecs(), ibctesting.UpgradePath),
						),
					},
					nil,
					[]clienttypes.IdentifiedGenesisMetadata{
						clienttypes.NewIdentifiedGenesisMetadata(
							clientID,
							[]clienttypes.GenesisMetadata{
								clienttypes.NewGenesisMetadata([]byte(""), []byte("val1")),
								clienttypes.NewGenesisMetadata([]byte("key2"), []byte("")),
							},
						),
					},
					clienttypes.NewParams(exported.Tendermint),
					false,
					2,
				),
				ConnectionGenesis: connectiontypes.DefaultGenesisState(),
			},
			expPass: false,
		},
		{
			name: "invalid connection genesis",
			genState: &types.GenesisState{
				ClientGenesis: clienttypes.DefaultGenesisState(),
				ConnectionGenesis: connectiontypes.NewGenesisState(
					[]connectiontypes.IdentifiedConnection{
						connectiontypes.NewIdentifiedConnection(connectionID, connectiontypes.NewConnectionEnd(connectiontypes.INIT, "(CLIENTIDONE)", connectiontypes.NewCounterparty(clientID, connectionID2, commitmenttypes.NewMerklePrefix([]byte("prefix"))), []*connectiontypes.Version{connectiontypes.NewVersion("1.1", nil)}, 0)),
					},
					[]connectiontypes.ConnectionPaths{
						connectiontypes.NewConnectionPaths(clientID, []string{connectionID}),
					},
					0,
					connectiontypes.Params{},
				),
			},
			expPass: false,
		},
		{
			name: "invalid channel genesis",
			genState: &types.GenesisState{
				ClientGenesis:     clienttypes.DefaultGenesisState(),
				ConnectionGenesis: connectiontypes.DefaultGenesisState(),
				ChannelGenesis: channeltypes.GenesisState{
					Acknowledgements: []channeltypes.PacketState{
						channeltypes.NewPacketState("(portID)", channel1, 1, []byte("ack")),
					},
				},
			},
			expPass: false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		err := tc.genState.Validate()
		if tc.expPass {
			suite.Require().NoError(err, tc.name)
		} else {
			suite.Require().Error(err, tc.name)
		}
	}
}

func (suite *IBCTestSuite) TestInitGenesis() {
	header := suite.chainA.CreateTMClientHeader(suite.chainA.ChainID, suite.chainA.CurrentHeader.Height, clienttypes.NewHeight(0, uint64(suite.chainA.CurrentHeader.Height-1)), suite.chainA.CurrentHeader.Time, suite.chainA.Vals, suite.chainA.Vals, suite.chainA.Vals, suite.chainA.Signers)

	testCases := []struct {
		name     string
		genState *types.GenesisState
	}{
		{
			name:     "default",
			genState: types.DefaultGenesisState(),
		},
		{
			name: "valid genesis",
			genState: &types.GenesisState{
				ClientGenesis: clienttypes.NewGenesisState(
					[]clienttypes.IdentifiedClientState{
						clienttypes.NewIdentifiedClientState(
							clientID, ibctm.NewClientState(suite.chainA.ChainID, ibctm.DefaultTrustLevel, ibctesting.TrustingPeriod, ibctesting.UnbondingPeriod, ibctesting.MaxClockDrift, clientHeight, commitmenttypes.GetSDKSpecs(), ibctesting.UpgradePath),
						),
					},
					[]clienttypes.ClientConsensusStates{
						clienttypes.NewClientConsensusStates(
							clientID,
							[]clienttypes.ConsensusStateWithHeight{
								clienttypes.NewConsensusStateWithHeight(
									header.GetHeight().(clienttypes.Height),
									ibctm.NewConsensusState(
										header.GetTime(), commitmenttypes.NewMerkleRoot(header.Header.AppHash), header.Header.NextValidatorsHash,
									),
								),
							},
						),
					},
					[]clienttypes.IdentifiedGenesisMetadata{
						clienttypes.NewIdentifiedGenesisMetadata(
							clientID,
							[]clienttypes.GenesisMetadata{
								clienttypes.NewGenesisMetadata([]byte("key1"), []byte("val1")),
								clienttypes.NewGenesisMetadata([]byte("key2"), []byte("val2")),
							},
						),
					},
					clienttypes.NewParams(exported.Tendermint),
					false,
					0,
				),
				ConnectionGenesis: connectiontypes.NewGenesisState(
					[]connectiontypes.IdentifiedConnection{
						connectiontypes.NewIdentifiedConnection(connectionID, connectiontypes.NewConnectionEnd(connectiontypes.INIT, clientID, connectiontypes.NewCounterparty(clientID2, connectionID2, commitmenttypes.NewMerklePrefix([]byte("prefix"))), []*connectiontypes.Version{ibctesting.ConnectionVersion}, 0)),
					},
					[]connectiontypes.ConnectionPaths{
						connectiontypes.NewConnectionPaths(clientID, []string{connectionID}),
					},
					0,
					connectiontypes.NewParams(10),
				),
				ChannelGenesis: channeltypes.NewGenesisState(
					[]channeltypes.IdentifiedChannel{
						channeltypes.NewIdentifiedChannel(
							port1, channel1, channeltypes.NewChannel(
								channeltypes.INIT, channeltypes.ORDERED,
								channeltypes.NewCounterparty(port2, channel2), []string{connectionID}, ibctesting.DefaultChannelVersion,
							),
						),
					},
					[]channeltypes.PacketState{
						channeltypes.NewPacketState(port2, channel2, 1, []byte("ack")),
					},
					[]channeltypes.PacketState{
						channeltypes.NewPacketState(port2, channel2, 1, []byte("")),
					},
					[]channeltypes.PacketState{
						channeltypes.NewPacketState(port1, channel1, 1, []byte("commit_hash")),
					},
					[]channeltypes.PacketSequence{
						channeltypes.NewPacketSequence(port1, channel1, 1),
					},
					[]channeltypes.PacketSequence{
						channeltypes.NewPacketSequence(port2, channel2, 1),
					},
					[]channeltypes.PacketSequence{
						channeltypes.NewPacketSequence(port2, channel2, 1),
					},
					0,
				),
			},
		},
	}

	for _, tc := range testCases {
		tc := tc

		app := simapp.Setup(suite.T(), false)

		suite.NotPanics(func() {
			ibc.InitGenesis(app.BaseApp.NewContext(false), *app.IBCKeeper, tc.genState)
		})
	}
}

func (suite *IBCTestSuite) TestExportGenesis() {
	testCases := []struct {
		msg      string
		malleate func()
	}{
		{
			"success",
			func() {
				// creates clients
				suite.coordinator.Setup(ibctesting.NewPath(suite.chainA, suite.chainB))
				// create extra clients
				suite.coordinator.SetupClients(ibctesting.NewPath(suite.chainA, suite.chainB))
				suite.coordinator.SetupClients(ibctesting.NewPath(suite.chainA, suite.chainB))
			},
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(fmt.Sprintf("Case %s", tc.msg), func() {
			suite.SetupTest()

			tc.malleate()

			var gs *types.GenesisState
			var err error
			suite.NotPanics(func() {
				gs, err = ibc.ExportGenesis(suite.chainA.GetContext(), *suite.chainA.App.GetIBCKeeper())
				suite.Require().NoError(err)
			})

			// init genesis based on export
			suite.NotPanics(func() {
				ibc.InitGenesis(suite.chainA.GetContext(), *suite.chainA.App.GetIBCKeeper(), gs)
			})

			suite.NotPanics(func() {
				cdc := codec.NewProtoCodec(suite.chainA.GetSimApp().InterfaceRegistry())
				genState := cdc.MustMarshalJSON(gs)
				cdc.MustUnmarshalJSON(genState, gs)
			})

			// init genesis based on marshal and unmarshal
			suite.NotPanics(func() {
				ibc.InitGenesis(suite.chainA.GetContext(), *suite.chainA.App.GetIBCKeeper(), gs)
			})
		})
	}
}
