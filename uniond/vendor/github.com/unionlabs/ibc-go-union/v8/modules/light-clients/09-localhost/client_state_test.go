package localhost_test

import (
	sdk "github.com/cosmos/cosmos-sdk/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	ibctm "github.com/cosmos/ibc-go/v8/modules/light-clients/07-tendermint"
	localhost "github.com/cosmos/ibc-go/v8/modules/light-clients/09-localhost"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
	"github.com/cosmos/ibc-go/v8/testing/mock"
)

func (suite *LocalhostTestSuite) TestStatus() {
	clientState := localhost.NewClientState(clienttypes.NewHeight(3, 10))
	suite.Require().Equal(exported.Active, clientState.Status(suite.chain.GetContext(), nil, nil))
}

func (suite *LocalhostTestSuite) TestClientType() {
	clientState := localhost.NewClientState(clienttypes.NewHeight(3, 10))
	suite.Require().Equal(exported.Localhost, clientState.ClientType())
}

func (suite *LocalhostTestSuite) TestGetLatestHeight() {
	expectedHeight := clienttypes.NewHeight(3, 10)
	clientState := localhost.NewClientState(expectedHeight)
	suite.Require().Equal(expectedHeight, clientState.GetLatestHeight())
}

func (suite *LocalhostTestSuite) TestZeroCustomFields() {
	clientState := localhost.NewClientState(clienttypes.NewHeight(1, 10))
	suite.Require().Equal(clientState, clientState.ZeroCustomFields())
}

func (suite *LocalhostTestSuite) TestGetTimestampAtHeight() {
	ctx := suite.chain.GetContext()
	clientState := localhost.NewClientState(clienttypes.NewHeight(1, 10))

	timestamp, err := clientState.GetTimestampAtHeight(ctx, nil, nil, nil)
	suite.Require().NoError(err)
	suite.Require().Equal(uint64(ctx.BlockTime().UnixNano()), timestamp)
}

func (suite *LocalhostTestSuite) TestValidate() {
	testCases := []struct {
		name        string
		clientState exported.ClientState
		expPass     bool
	}{
		{
			name:        "valid client",
			clientState: localhost.NewClientState(clienttypes.NewHeight(3, 10)),
			expPass:     true,
		},
		{
			name:        "invalid height",
			clientState: localhost.NewClientState(clienttypes.ZeroHeight()),
			expPass:     false,
		},
	}

	for _, tc := range testCases {
		tc := tc
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

func (suite *LocalhostTestSuite) TestInitialize() {
	testCases := []struct {
		name      string
		consState exported.ConsensusState
		expPass   bool
	}{
		{
			"valid initialization",
			nil,
			true,
		},
		{
			"invalid consenus state",
			&ibctm.ConsensusState{},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			clientState := localhost.NewClientState(clienttypes.NewHeight(3, 10))
			clientStore := suite.chain.GetSimApp().GetIBCKeeper().ClientKeeper.ClientStore(suite.chain.GetContext(), exported.LocalhostClientID)

			err := clientState.Initialize(suite.chain.GetContext(), suite.chain.Codec, clientStore, tc.consState)

			if tc.expPass {
				suite.Require().NoError(err, "valid testcase: %s failed", tc.name)
			} else {
				suite.Require().Error(err, "invalid testcase: %s passed", tc.name)
			}
		})
		suite.SetupTest()

	}
}

func (suite *LocalhostTestSuite) TestVerifyMembership() {
	var (
		path  exported.Path
		value []byte
	)

	testCases := []struct {
		name     string
		malleate func()
		expPass  bool
	}{
		{
			"success: client state verification",
			func() {
				clientState := suite.chain.GetClientState(exported.LocalhostClientID)

				merklePath := commitmenttypes.NewMerklePath(host.FullClientStatePath(exported.LocalhostClientID))
				merklePath, err := commitmenttypes.ApplyPrefix(suite.chain.GetPrefix(), merklePath)
				suite.Require().NoError(err)

				path = merklePath
				value = clienttypes.MustMarshalClientState(suite.chain.Codec, clientState)
			},
			true,
		},
		{
			"success: connection state verification",
			func() {
				connectionEnd := connectiontypes.NewConnectionEnd(
					connectiontypes.OPEN,
					exported.LocalhostClientID,
					connectiontypes.NewCounterparty(exported.LocalhostClientID, exported.LocalhostConnectionID, suite.chain.GetPrefix()),
					connectiontypes.GetCompatibleVersions(), 0,
				)

				suite.chain.GetSimApp().GetIBCKeeper().ConnectionKeeper.SetConnection(suite.chain.GetContext(), exported.LocalhostConnectionID, connectionEnd)

				merklePath := commitmenttypes.NewMerklePath(host.ConnectionPath(exported.LocalhostConnectionID))
				merklePath, err := commitmenttypes.ApplyPrefix(suite.chain.GetPrefix(), merklePath)
				suite.Require().NoError(err)

				path = merklePath
				value = suite.chain.Codec.MustMarshal(&connectionEnd)
			},
			true,
		},
		{
			"success: channel state verification",
			func() {
				channel := channeltypes.NewChannel(
					channeltypes.OPEN,
					channeltypes.UNORDERED,
					channeltypes.NewCounterparty(mock.PortID, ibctesting.FirstChannelID),
					[]string{exported.LocalhostConnectionID},
					mock.Version,
				)

				suite.chain.GetSimApp().GetIBCKeeper().ChannelKeeper.SetChannel(suite.chain.GetContext(), mock.PortID, ibctesting.FirstChannelID, channel)

				merklePath := commitmenttypes.NewMerklePath(host.ChannelPath(mock.PortID, ibctesting.FirstChannelID))
				merklePath, err := commitmenttypes.ApplyPrefix(suite.chain.GetPrefix(), merklePath)
				suite.Require().NoError(err)

				path = merklePath
				value = suite.chain.Codec.MustMarshal(&channel)
			},
			true,
		},
		{
			"success: next sequence recv verification",
			func() {
				nextSeqRecv := uint64(100)
				suite.chain.GetSimApp().GetIBCKeeper().ChannelKeeper.SetNextSequenceRecv(suite.chain.GetContext(), mock.PortID, ibctesting.FirstChannelID, nextSeqRecv)

				merklePath := commitmenttypes.NewMerklePath(host.NextSequenceRecvPath(mock.PortID, ibctesting.FirstChannelID))
				merklePath, err := commitmenttypes.ApplyPrefix(suite.chain.GetPrefix(), merklePath)
				suite.Require().NoError(err)

				path = merklePath
				value = sdk.Uint64ToBigEndian(nextSeqRecv)
			},
			true,
		},
		{
			"success: packet commitment verification",
			func() {
				packet := channeltypes.NewPacket(
					ibctesting.MockPacketData,
					1,
					ibctesting.MockPort,
					ibctesting.FirstChannelID,
					ibctesting.MockPort,
					ibctesting.FirstChannelID,
					clienttypes.NewHeight(0, 10),
					0,
				)

				commitmentBz := channeltypes.CommitPacket(suite.chain.Codec, packet)
				suite.chain.GetSimApp().GetIBCKeeper().ChannelKeeper.SetPacketCommitment(suite.chain.GetContext(), mock.PortID, ibctesting.FirstChannelID, 1, commitmentBz)

				merklePath := commitmenttypes.NewMerklePath(host.PacketCommitmentPath(packet.GetSourcePort(), packet.GetSourceChannel(), packet.GetSequence()))
				merklePath, err := commitmenttypes.ApplyPrefix(suite.chain.GetPrefix(), merklePath)
				suite.Require().NoError(err)

				path = merklePath
				value = commitmentBz
			},
			true,
		},
		{
			"success: packet acknowledgement verification",
			func() {
				suite.chain.GetSimApp().GetIBCKeeper().ChannelKeeper.SetPacketAcknowledgement(suite.chain.GetContext(), mock.PortID, ibctesting.FirstChannelID, 1, ibctesting.MockAcknowledgement)

				merklePath := commitmenttypes.NewMerklePath(host.PacketAcknowledgementPath(mock.PortID, ibctesting.FirstChannelID, 1))
				merklePath, err := commitmenttypes.ApplyPrefix(suite.chain.GetPrefix(), merklePath)
				suite.Require().NoError(err)

				path = merklePath
				value = ibctesting.MockAcknowledgement
			},
			true,
		},
		{
			"invalid type for key path",
			func() {
				path = mock.KeyPath{}
			},
			false,
		},
		{
			"key path has too many elements",
			func() {
				path = commitmenttypes.NewMerklePath("ibc", "test", "key")
			},
			false,
		},
		{
			"no value found at provided key path",
			func() {
				merklePath := commitmenttypes.NewMerklePath(host.PacketAcknowledgementPath(mock.PortID, ibctesting.FirstChannelID, 100))
				merklePath, err := commitmenttypes.ApplyPrefix(suite.chain.GetPrefix(), merklePath)
				suite.Require().NoError(err)

				path = merklePath
				value = ibctesting.MockAcknowledgement
			},
			false,
		},
		{
			"invalid value, bytes are not equal",
			func() {
				channel := channeltypes.NewChannel(
					channeltypes.OPEN,
					channeltypes.UNORDERED,
					channeltypes.NewCounterparty(mock.PortID, ibctesting.FirstChannelID),
					[]string{exported.LocalhostConnectionID},
					mock.Version,
				)

				suite.chain.GetSimApp().GetIBCKeeper().ChannelKeeper.SetChannel(suite.chain.GetContext(), mock.PortID, ibctesting.FirstChannelID, channel)

				merklePath := commitmenttypes.NewMerklePath(host.ChannelPath(mock.PortID, ibctesting.FirstChannelID))
				merklePath, err := commitmenttypes.ApplyPrefix(suite.chain.GetPrefix(), merklePath)
				suite.Require().NoError(err)

				path = merklePath

				// modify the channel before marshalling to value bz
				channel.State = channeltypes.CLOSED
				value = suite.chain.Codec.MustMarshal(&channel)
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest()

			tc.malleate()

			clientState := suite.chain.GetClientState(exported.LocalhostClientID)
			store := suite.chain.GetContext().KVStore(suite.chain.GetSimApp().GetKey(exported.StoreKey))

			err := clientState.VerifyMembership(
				suite.chain.GetContext(),
				store,
				suite.chain.Codec,
				clienttypes.ZeroHeight(),
				0, 0, // use zero values for delay periods
				localhost.SentinelProof,
				path,
				value,
			)

			if tc.expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *LocalhostTestSuite) TestVerifyNonMembership() {
	var path exported.Path

	testCases := []struct {
		name     string
		malleate func()
		expPass  bool
	}{
		{
			"success: packet receipt absence verification",
			func() {
				merklePath := commitmenttypes.NewMerklePath(host.PacketReceiptPath(mock.PortID, ibctesting.FirstChannelID, 1))
				merklePath, err := commitmenttypes.ApplyPrefix(suite.chain.GetPrefix(), merklePath)
				suite.Require().NoError(err)

				path = merklePath
			},
			true,
		},
		{
			"packet receipt absence verification fails",
			func() {
				suite.chain.GetSimApp().GetIBCKeeper().ChannelKeeper.SetPacketReceipt(suite.chain.GetContext(), mock.PortID, ibctesting.FirstChannelID, 1)

				merklePath := commitmenttypes.NewMerklePath(host.PacketReceiptPath(mock.PortID, ibctesting.FirstChannelID, 1))
				merklePath, err := commitmenttypes.ApplyPrefix(suite.chain.GetPrefix(), merklePath)
				suite.Require().NoError(err)

				path = merklePath
			},
			false,
		},
		{
			"invalid type for key path",
			func() {
				path = mock.KeyPath{}
			},
			false,
		},
		{
			"key path has too many elements",
			func() {
				path = commitmenttypes.NewMerklePath("ibc", "test", "key")
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest()

			tc.malleate()

			clientState := suite.chain.GetClientState(exported.LocalhostClientID)
			store := suite.chain.GetContext().KVStore(suite.chain.GetSimApp().GetKey(exported.StoreKey))

			err := clientState.VerifyNonMembership(
				suite.chain.GetContext(),
				store,
				suite.chain.Codec,
				clienttypes.ZeroHeight(),
				0, 0, // use zero values for delay periods
				localhost.SentinelProof,
				path,
			)

			if tc.expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *LocalhostTestSuite) TestVerifyClientMessage() {
	clientState := localhost.NewClientState(clienttypes.NewHeight(1, 10))
	suite.Require().Error(clientState.VerifyClientMessage(suite.chain.GetContext(), nil, nil, nil))
}

func (suite *LocalhostTestSuite) TestVerifyCheckForMisbehaviour() {
	clientState := localhost.NewClientState(clienttypes.NewHeight(1, 10))
	suite.Require().False(clientState.CheckForMisbehaviour(suite.chain.GetContext(), nil, nil, nil))
}

func (suite *LocalhostTestSuite) TestUpdateState() {
	clientState := localhost.NewClientState(clienttypes.NewHeight(1, uint64(suite.chain.GetContext().BlockHeight())))
	store := suite.chain.GetSimApp().GetIBCKeeper().ClientKeeper.ClientStore(suite.chain.GetContext(), exported.LocalhostClientID)

	suite.coordinator.CommitBlock(suite.chain)

	heights := clientState.UpdateState(suite.chain.GetContext(), suite.chain.Codec, store, nil)

	expHeight := clienttypes.NewHeight(1, uint64(suite.chain.GetContext().BlockHeight()))
	suite.Require().True(heights[0].EQ(expHeight))

	clientState = suite.chain.GetClientState(exported.LocalhostClientID)
	suite.Require().True(heights[0].EQ(clientState.GetLatestHeight()))
}

func (suite *LocalhostTestSuite) TestExportMetadata() {
	clientState := localhost.NewClientState(clienttypes.NewHeight(1, 10))
	suite.Require().Nil(clientState.ExportMetadata(nil))
}

func (suite *LocalhostTestSuite) TestCheckSubstituteAndUpdateState() {
	clientState := localhost.NewClientState(clienttypes.NewHeight(1, 10))
	err := clientState.CheckSubstituteAndUpdateState(suite.chain.GetContext(), suite.chain.Codec, nil, nil, nil)
	suite.Require().Error(err)
}

func (suite *LocalhostTestSuite) TestVerifyUpgradeAndUpdateState() {
	clientState := localhost.NewClientState(clienttypes.NewHeight(1, 10))
	err := clientState.VerifyUpgradeAndUpdateState(suite.chain.GetContext(), suite.chain.Codec, nil, nil, nil, nil, nil)
	suite.Require().Error(err)
}
