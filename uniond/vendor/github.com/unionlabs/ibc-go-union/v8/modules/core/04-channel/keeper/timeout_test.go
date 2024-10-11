package keeper_test

import (
	"errors"
	"fmt"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"

	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	"github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
	"github.com/cosmos/ibc-go/v8/testing/mock"
)

// TestTimeoutPacket test the TimeoutPacket call on chainA by ensuring the timeout has passed
// on chainB, but that no ack has been written yet. Test cases expected to reach proof
// verification must specify which proof to use using the ordered bool.
func (suite *KeeperTestSuite) TestTimeoutPacket() {
	var (
		path        *ibctesting.Path
		packet      types.Packet
		nextSeqRecv uint64
		ordered     bool
		expError    *errorsmod.Error
	)

	testCases := []testCase{
		{"success: ORDERED", func() {
			ordered = true
			path.SetChannelOrdered()
			suite.coordinator.Setup(path)

			timeoutHeight := clienttypes.GetSelfHeight(suite.chainB.GetContext())
			timeoutTimestamp := uint64(suite.chainB.GetContext().BlockTime().UnixNano())

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, timeoutTimestamp, ibctesting.MockPacketData)
			suite.Require().NoError(err)
			packet = types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, timeoutTimestamp)
			// need to update chainA's client representing chainB to prove missing ack
			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)
		}, true},
		{"success: UNORDERED", func() {
			ordered = false
			suite.coordinator.Setup(path)

			timeoutHeight := clienttypes.GetSelfHeight(suite.chainB.GetContext())

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, disabledTimeoutTimestamp, ibctesting.MockPacketData)
			suite.Require().NoError(err)
			packet = types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, disabledTimeoutTimestamp)
			// need to update chainA's client representing chainB to prove missing ack
			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)
		}, true},
		{"packet already timed out: ORDERED", func() {
			expError = types.ErrNoOpMsg
			ordered = true
			path.SetChannelOrdered()
			suite.coordinator.Setup(path)

			timeoutHeight := clienttypes.GetSelfHeight(suite.chainB.GetContext())
			timeoutTimestamp := uint64(suite.chainB.GetContext().BlockTime().UnixNano())

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, timeoutTimestamp, ibctesting.MockPacketData)
			suite.Require().NoError(err)
			// need to update chainA's client representing chainB to prove missing ack
			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			packet = types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, timeoutTimestamp)
			err = path.EndpointA.TimeoutPacket(packet)
			suite.Require().NoError(err)
		}, false},
		{"packet already timed out: UNORDERED", func() {
			expError = types.ErrNoOpMsg
			ordered = false
			suite.coordinator.Setup(path)

			timeoutHeight := clienttypes.GetSelfHeight(suite.chainB.GetContext())

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, disabledTimeoutTimestamp, ibctesting.MockPacketData)
			suite.Require().NoError(err)
			// need to update chainA's client representing chainB to prove missing ack
			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			packet = types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, disabledTimeoutTimestamp)
			err = path.EndpointA.TimeoutPacket(packet)
			suite.Require().NoError(err)
		}, false},
		{"channel not found", func() {
			expError = types.ErrChannelNotFound
			// use wrong channel naming
			suite.coordinator.Setup(path)
			packet = types.NewPacket(ibctesting.MockPacketData, 1, ibctesting.InvalidID, ibctesting.InvalidID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, defaultTimeoutHeight, disabledTimeoutTimestamp)
		}, false},
		{"packet destination port ≠ channel counterparty port", func() {
			expError = types.ErrInvalidPacket
			suite.coordinator.Setup(path)
			// use wrong port for dest
			packet = types.NewPacket(ibctesting.MockPacketData, 1, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, ibctesting.InvalidID, path.EndpointB.ChannelID, defaultTimeoutHeight, disabledTimeoutTimestamp)
		}, false},
		{"packet destination channel ID ≠ channel counterparty channel ID", func() {
			expError = types.ErrInvalidPacket
			suite.coordinator.Setup(path)
			// use wrong channel for dest
			packet = types.NewPacket(ibctesting.MockPacketData, 1, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, ibctesting.InvalidID, defaultTimeoutHeight, disabledTimeoutTimestamp)
		}, false},
		{"connection not found", func() {
			expError = connectiontypes.ErrConnectionNotFound
			// pass channel check
			suite.chainA.App.GetIBCKeeper().ChannelKeeper.SetChannel(
				suite.chainA.GetContext(),
				path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID,
				types.NewChannel(types.OPEN, types.ORDERED, types.NewCounterparty(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID), []string{connIDA}, path.EndpointA.ChannelConfig.Version),
			)
			packet = types.NewPacket(ibctesting.MockPacketData, 1, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, defaultTimeoutHeight, disabledTimeoutTimestamp)
		}, false},
		{"timeout", func() {
			expError = types.ErrTimeoutNotReached
			suite.coordinator.Setup(path)
			sequence, err := path.EndpointA.SendPacket(defaultTimeoutHeight, disabledTimeoutTimestamp, ibctesting.MockPacketData)
			suite.Require().NoError(err)
			packet = types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, defaultTimeoutHeight, disabledTimeoutTimestamp)
			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)
		}, false},
		{"packet already received ", func() {
			expError = types.ErrPacketReceived
			ordered = true
			path.SetChannelOrdered()
			suite.coordinator.Setup(path)

			nextSeqRecv = 2
			timeoutTimestamp := uint64(suite.chainB.GetContext().BlockTime().UnixNano())

			sequence, err := path.EndpointA.SendPacket(defaultTimeoutHeight, timeoutTimestamp, ibctesting.MockPacketData)
			suite.Require().NoError(err)
			packet = types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, defaultTimeoutHeight, timeoutTimestamp)
			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)
		}, false},
		{"packet hasn't been sent", func() {
			expError = types.ErrNoOpMsg
			ordered = true
			path.SetChannelOrdered()

			suite.coordinator.Setup(path)
			packet = types.NewPacket(ibctesting.MockPacketData, 1, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, defaultTimeoutHeight, uint64(suite.chainB.GetContext().BlockTime().UnixNano()))
			err := path.EndpointA.UpdateClient()
			suite.Require().NoError(err)
		}, false},
		{"next seq receive verification failed", func() {
			// skip error check, error occurs in light-clients

			// set ordered to false resulting in wrong proof provided
			ordered = false

			path.SetChannelOrdered()
			suite.coordinator.Setup(path)

			timeoutHeight := clienttypes.GetSelfHeight(suite.chainB.GetContext())

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, disabledTimeoutTimestamp, ibctesting.MockPacketData)
			suite.Require().NoError(err)
			packet = types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, disabledTimeoutTimestamp)
			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)
		}, false},
		{"packet ack verification failed", func() {
			// skip error check, error occurs in light-clients

			// set ordered to true resulting in wrong proof provided
			ordered = true

			suite.coordinator.Setup(path)

			timeoutHeight := clienttypes.GetSelfHeight(suite.chainB.GetContext())

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, disabledTimeoutTimestamp, ibctesting.MockPacketData)
			suite.Require().NoError(err)
			packet = types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, disabledTimeoutTimestamp)
			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)
		}, false},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.msg, func() {
			var (
				proof       []byte
				proofHeight exported.Height
			)

			suite.SetupTest() // reset
			expError = nil    // must be expliticly changed by failed cases
			nextSeqRecv = 1   // must be explicitly changed
			path = ibctesting.NewPath(suite.chainA, suite.chainB)

			tc.malleate()

			orderedPacketKey := host.NextSequenceRecvKey(packet.GetDestPort(), packet.GetDestChannel())
			unorderedPacketKey := host.PacketReceiptKey(packet.GetDestPort(), packet.GetDestChannel(), packet.GetSequence())

			if path.EndpointB.ConnectionID != "" {
				if ordered {
					proof, proofHeight = path.EndpointB.QueryProof(orderedPacketKey)
				} else {
					proof, proofHeight = path.EndpointB.QueryProof(unorderedPacketKey)
				}
			}

			err := suite.chainA.App.GetIBCKeeper().ChannelKeeper.TimeoutPacket(suite.chainA.GetContext(), packet, proof, proofHeight, nextSeqRecv)

			if tc.expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
				// only check if expError is set, since not all error codes can be known
				if expError != nil {
					suite.Require().True(errors.Is(err, expError))
				}
			}
		})
	}
}

// TestTimeoutExecuted verifies that packet commitments are deleted on chainA after the
// channel capabilities are verified. In addition, the test verifies that the channel state
// after a timeout is updated accordingly.
func (suite *KeeperTestSuite) TestTimeoutExecuted() {
	var (
		path    *ibctesting.Path
		packet  types.Packet
		chanCap *capabilitytypes.Capability
	)

	testCases := []struct {
		msg       string
		malleate  func()
		expResult func(packetCommitment []byte, err error)
		expEvents func(path *ibctesting.Path) map[string]map[string]string
	}{
		{
			"success ORDERED",
			func() {
				path.SetChannelOrdered()
				suite.coordinator.Setup(path)

				timeoutHeight := clienttypes.GetSelfHeight(suite.chainB.GetContext())
				timeoutTimestamp := uint64(suite.chainB.GetContext().BlockTime().UnixNano())

				sequence, err := path.EndpointA.SendPacket(timeoutHeight, timeoutTimestamp, ibctesting.MockPacketData)
				suite.Require().NoError(err)

				packet = types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, timeoutTimestamp)
				chanCap = suite.chainA.GetChannelCapability(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
			},
			func(packetCommitment []byte, err error) {
				suite.Require().NoError(err)
				suite.Require().Nil(packetCommitment)

				// Check channel has been closed
				channel := path.EndpointA.GetChannel()
				suite.Require().Equal(channel.State, types.CLOSED)
			},
			nil,
		},
		{
			"channel not found",
			func() {
				// use wrong channel naming
				suite.coordinator.Setup(path)
				packet = types.NewPacket(ibctesting.MockPacketData, 1, ibctesting.InvalidID, ibctesting.InvalidID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, defaultTimeoutHeight, disabledTimeoutTimestamp)
			},
			func(packetCommitment []byte, err error) {
				suite.Require().Error(err)
				suite.Require().ErrorIs(err, types.ErrChannelNotFound)

				// packet never sent.
				suite.Require().Nil(packetCommitment)
			},
			nil,
		},
		{
			"incorrect capability ORDERED",
			func() {
				path.SetChannelOrdered()
				suite.coordinator.Setup(path)

				timeoutHeight := clienttypes.GetSelfHeight(suite.chainB.GetContext())
				timeoutTimestamp := uint64(suite.chainB.GetContext().BlockTime().UnixNano())

				sequence, err := path.EndpointA.SendPacket(timeoutHeight, timeoutTimestamp, ibctesting.MockPacketData)
				suite.Require().NoError(err)

				packet = types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, timeoutTimestamp)
				chanCap = capabilitytypes.NewCapability(100)
			},
			func(packetCommitment []byte, err error) {
				suite.Require().Error(err)
				suite.Require().ErrorIs(err, types.ErrChannelCapabilityNotFound)

				// packet sent, never deleted.
				suite.Require().NotNil(packetCommitment)
			},
			nil,
		},
		{
			"set to flush complete with no inflight packets",
			func() {
				suite.coordinator.Setup(path)

				chanCap = suite.chainA.GetChannelCapability(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)

				channel := path.EndpointA.GetChannel()
				channel.State = types.FLUSHING
				path.EndpointA.SetChannel(channel)
				path.EndpointA.SetChannelCounterpartyUpgrade(types.Upgrade{
					Timeout: types.NewTimeout(clienttypes.ZeroHeight(), uint64(suite.chainA.GetContext().BlockTime().UnixNano())+types.DefaultTimeout.Timestamp),
				})
			},
			func(packetCommitment []byte, err error) {
				suite.Require().NoError(err)

				channel := path.EndpointA.GetChannel()
				suite.Require().Equal(types.FLUSHCOMPLETE, channel.State, "channel state should still be set to FLUSHCOMPLETE")
			},
			func(path *ibctesting.Path) map[string]map[string]string {
				return ibctesting.EventsMap{
					types.EventTypeChannelFlushComplete: {
						types.AttributeKeyPortID:             path.EndpointA.ChannelConfig.PortID,
						types.AttributeKeyChannelID:          path.EndpointA.ChannelID,
						types.AttributeCounterpartyPortID:    path.EndpointB.ChannelConfig.PortID,
						types.AttributeCounterpartyChannelID: path.EndpointB.ChannelID,
						types.AttributeKeyChannelState:       path.EndpointA.GetChannel().State.String(),
					},
					sdk.EventTypeMessage: {
						sdk.AttributeKeyModule: types.AttributeValueCategory,
					},
				}
			},
		},
		{
			"conterparty upgrade timeout is invalid",
			func() {
				suite.coordinator.Setup(path)

				timeoutHeight := clienttypes.GetSelfHeight(suite.chainB.GetContext())
				timeoutTimestamp := uint64(suite.chainB.GetContext().BlockTime().UnixNano())

				sequence, err := path.EndpointA.SendPacket(timeoutHeight, timeoutTimestamp, ibctesting.MockPacketData)
				suite.Require().NoError(err)

				packet = types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, timeoutTimestamp)
				chanCap = suite.chainA.GetChannelCapability(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)

				channel := path.EndpointA.GetChannel()
				channel.State = types.FLUSHING
				path.EndpointA.SetChannel(channel)
			},
			func(packetCommitment []byte, err error) {
				suite.Require().NoError(err)

				channel := path.EndpointA.GetChannel()
				suite.Require().Equal(types.FLUSHING, channel.State, "channel state should still be FLUSHING")
			},
			nil,
		},
		{
			"conterparty upgrade timed out (abort)",
			func() {
				suite.coordinator.Setup(path)

				timeoutHeight := clienttypes.GetSelfHeight(suite.chainB.GetContext())
				timeoutTimestamp := uint64(suite.chainB.GetContext().BlockTime().UnixNano())

				sequence, err := path.EndpointA.SendPacket(timeoutHeight, timeoutTimestamp, ibctesting.MockPacketData)
				suite.Require().NoError(err)

				packet = types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, timeoutTimestamp)
				chanCap = suite.chainA.GetChannelCapability(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)

				channel := path.EndpointA.GetChannel()
				channel.State = types.FLUSHING
				path.EndpointA.SetChannel(channel)
				path.EndpointA.SetChannelUpgrade(types.Upgrade{
					Fields:  path.EndpointA.GetProposedUpgrade().Fields,
					Timeout: types.NewTimeout(clienttypes.ZeroHeight(), 1),
				})
				path.EndpointA.SetChannelCounterpartyUpgrade(types.Upgrade{
					Timeout: types.NewTimeout(clienttypes.ZeroHeight(), 1),
				})
			},
			func(packetCommitment []byte, err error) {
				suite.Require().NoError(err)

				channel := path.EndpointA.GetChannel()
				suite.Require().Equal(types.OPEN, channel.State, "channel state should still be OPEN")

				upgrade, found := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetUpgrade(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().False(found, "upgrade should not be present")
				suite.Require().Equal(types.Upgrade{}, upgrade, "upgrade should be zero value")

				upgrade, found = suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetCounterpartyUpgrade(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().False(found, "counterparty upgrade should not be present")
				suite.Require().Equal(types.Upgrade{}, upgrade, "counterparty upgrade should be zero value")

				errorReceipt, found := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetUpgradeErrorReceipt(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found, "error receipt should be present")
				suite.Require().Equal(channel.UpgradeSequence, errorReceipt.Sequence, "error receipt sequence should be equal to channel upgrade sequence")
			},
			nil,
		},
		{
			"conterparty upgrade has not timed out with in-flight packets",
			func() {
				suite.coordinator.Setup(path)

				timeoutHeight := clienttypes.GetSelfHeight(suite.chainB.GetContext())
				timeoutTimestamp := uint64(suite.chainB.GetContext().BlockTime().UnixNano())

				// we are sending two packets here as one will be removed in TimeoutExecuted. This is to ensure that
				// there is still an in-flight packet so that the channel remains in the flushing state.
				_, err := path.EndpointA.SendPacket(timeoutHeight, timeoutTimestamp, ibctesting.MockPacketData)
				suite.Require().NoError(err)

				sequence, err := path.EndpointA.SendPacket(timeoutHeight, timeoutTimestamp, ibctesting.MockPacketData)
				suite.Require().NoError(err)

				packet = types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, timeoutTimestamp)
				chanCap = suite.chainA.GetChannelCapability(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)

				channel := path.EndpointA.GetChannel()
				channel.State = types.FLUSHING
				path.EndpointA.SetChannel(channel)
				path.EndpointA.SetChannelUpgrade(types.Upgrade{
					Fields:  path.EndpointA.GetProposedUpgrade().Fields,
					Timeout: types.NewTimeout(clienttypes.ZeroHeight(), uint64(suite.chainA.GetContext().BlockTime().UnixNano())+types.DefaultTimeout.Timestamp),
				})
				path.EndpointA.SetChannelCounterpartyUpgrade(types.Upgrade{
					Timeout: types.NewTimeout(clienttypes.ZeroHeight(), uint64(suite.chainB.GetContext().BlockTime().UnixNano())+types.DefaultTimeout.Timestamp),
				})
			},
			func(packetCommitment []byte, err error) {
				suite.Require().NoError(err)

				channel := path.EndpointA.GetChannel()
				suite.Require().Equal(types.FLUSHING, channel.State, "channel state should still be FLUSHING")

				_, found := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetUpgrade(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found, "upgrade should not be deleted")

				_, found = suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetCounterpartyUpgrade(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found, "counterparty upgrade should not be deleted")

				_, found = suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetUpgradeErrorReceipt(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().False(found, "error receipt should not be written")
			},
			nil,
		},
		{
			"ordered channel is closed and upgrade is aborted when timeout is executed",
			func() {
				path.SetChannelOrdered()
				suite.coordinator.Setup(path)

				timeoutHeight := clienttypes.GetSelfHeight(suite.chainB.GetContext())
				timeoutTimestamp := uint64(suite.chainB.GetContext().BlockTime().UnixNano())

				sequence, err := path.EndpointA.SendPacket(timeoutHeight, timeoutTimestamp, ibctesting.MockPacketData)
				suite.Require().NoError(err)

				packet = types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, timeoutTimestamp)
				chanCap = suite.chainA.GetChannelCapability(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)

				channel := path.EndpointA.GetChannel()
				channel.State = types.FLUSHING
				path.EndpointA.SetChannel(channel)
				path.EndpointA.SetChannelUpgrade(types.Upgrade{
					Fields:  path.EndpointA.GetProposedUpgrade().Fields,
					Timeout: types.NewTimeout(clienttypes.ZeroHeight(), 1),
				})
				path.EndpointA.SetChannelCounterpartyUpgrade(types.Upgrade{
					Timeout: types.NewTimeout(clienttypes.ZeroHeight(), 1),
				})
			},
			func(packetCommitment []byte, err error) {
				suite.Require().NoError(err)

				channel := path.EndpointA.GetChannel()
				suite.Require().Equal(types.CLOSED, channel.State, "channel state should be CLOSED")

				upgrade, found := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetUpgrade(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().False(found, "upgrade should not be present")
				suite.Require().Equal(types.Upgrade{}, upgrade, "upgrade should be zero value")

				upgrade, found = suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetCounterpartyUpgrade(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().False(found, "counterparty upgrade should not be present")
				suite.Require().Equal(types.Upgrade{}, upgrade, "counterparty upgrade should be zero value")

				errorReceipt, found := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetUpgradeErrorReceipt(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found, "error receipt should be present")
				suite.Require().Equal(channel.UpgradeSequence, errorReceipt.Sequence, "error receipt sequence should be equal to channel upgrade sequence")
			},
			nil,
		},
	}

	for i, tc := range testCases {
		tc := tc
		suite.Run(fmt.Sprintf("Case %s, %d/%d tests", tc.msg, i, len(testCases)), func() {
			suite.SetupTest() // reset
			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			ctx := suite.chainA.GetContext()

			tc.malleate()

			err := suite.chainA.App.GetIBCKeeper().ChannelKeeper.TimeoutExecuted(ctx, chanCap, packet)
			pc := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetPacketCommitment(ctx, packet.GetSourcePort(), packet.GetSourceChannel(), packet.GetSequence())

			tc.expResult(pc, err)
			if tc.expEvents != nil {
				events := ctx.EventManager().ABCIEvents()

				expEvents := tc.expEvents(path)

				ibctesting.AssertEventsLegacy(&suite.Suite, expEvents, events)
			}
		})
	}
}

// TestTimeoutOnClose tests the call TimeoutOnClose on chainA by closing the corresponding
// channel on chainB after the packet commitment has been created.
func (suite *KeeperTestSuite) TestTimeoutOnClose() {
	var (
		path                        *ibctesting.Path
		packet                      types.Packet
		chanCap                     *capabilitytypes.Capability
		nextSeqRecv                 uint64
		counterpartyUpgradeSequence uint64
		ordered                     bool
	)

	testCases := []testCase{
		{"success: ORDERED", func() {
			ordered = true
			path.SetChannelOrdered()
			suite.coordinator.Setup(path)

			timeoutHeight := clienttypes.GetSelfHeight(suite.chainB.GetContext())
			timeoutTimestamp := uint64(suite.chainB.GetContext().BlockTime().UnixNano())

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, timeoutTimestamp, ibctesting.MockPacketData)
			suite.Require().NoError(err)
			err = path.EndpointB.SetChannelState(types.CLOSED)
			suite.Require().NoError(err)
			// need to update chainA's client representing chainB to prove missing ack
			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			packet = types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, timeoutTimestamp)
			chanCap = suite.chainA.GetChannelCapability(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
		}, true},
		{"success: UNORDERED", func() {
			ordered = false
			suite.coordinator.Setup(path)

			timeoutHeight := clienttypes.GetSelfHeight(suite.chainB.GetContext())

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, disabledTimeoutTimestamp, ibctesting.MockPacketData)
			suite.Require().NoError(err)
			err = path.EndpointB.SetChannelState(types.CLOSED)
			suite.Require().NoError(err)
			// need to update chainA's client representing chainB to prove missing ack
			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			packet = types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, disabledTimeoutTimestamp)
			chanCap = suite.chainA.GetChannelCapability(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
		}, true},
		{"channel not found", func() {
			// use wrong channel naming
			suite.coordinator.Setup(path)
			packet = types.NewPacket(ibctesting.MockPacketData, 1, ibctesting.InvalidID, ibctesting.InvalidID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, defaultTimeoutHeight, disabledTimeoutTimestamp)
		}, false},
		{"packet dest port ≠ channel counterparty port", func() {
			suite.coordinator.Setup(path)
			// use wrong port for dest
			packet = types.NewPacket(ibctesting.MockPacketData, 1, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, ibctesting.InvalidID, path.EndpointB.ChannelID, defaultTimeoutHeight, disabledTimeoutTimestamp)
			chanCap = suite.chainA.GetChannelCapability(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
		}, false},
		{"packet dest channel ID ≠ channel counterparty channel ID", func() {
			suite.coordinator.Setup(path)
			// use wrong channel for dest
			packet = types.NewPacket(ibctesting.MockPacketData, 1, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, ibctesting.InvalidID, defaultTimeoutHeight, disabledTimeoutTimestamp)
			chanCap = suite.chainA.GetChannelCapability(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
		}, false},
		{"connection not found", func() {
			// pass channel check
			suite.chainA.App.GetIBCKeeper().ChannelKeeper.SetChannel(
				suite.chainA.GetContext(),
				path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID,
				types.NewChannel(types.OPEN, types.ORDERED, types.NewCounterparty(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID), []string{connIDA}, path.EndpointA.ChannelConfig.Version),
			)
			packet = types.NewPacket(ibctesting.MockPacketData, 1, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, defaultTimeoutHeight, disabledTimeoutTimestamp)

			// create chancap
			suite.chainA.CreateChannelCapability(suite.chainA.GetSimApp().ScopedIBCMockKeeper, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
			chanCap = suite.chainA.GetChannelCapability(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
		}, false},
		{"packet hasn't been sent ORDERED", func() {
			path.SetChannelOrdered()
			suite.coordinator.Setup(path)

			packet = types.NewPacket(ibctesting.MockPacketData, 1, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, clienttypes.GetSelfHeight(suite.chainB.GetContext()), uint64(suite.chainB.GetContext().BlockTime().UnixNano()))
			chanCap = suite.chainA.GetChannelCapability(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
		}, false},
		{"packet already received ORDERED", func() {
			path.SetChannelOrdered()
			nextSeqRecv = 2
			ordered = true
			suite.coordinator.Setup(path)

			timeoutHeight := clienttypes.GetSelfHeight(suite.chainB.GetContext())
			timeoutTimestamp := uint64(suite.chainB.GetContext().BlockTime().UnixNano())

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, timeoutTimestamp, ibctesting.MockPacketData)
			suite.Require().NoError(err)
			err = path.EndpointB.SetChannelState(types.CLOSED)
			suite.Require().NoError(err)
			// need to update chainA's client representing chainB to prove missing ack
			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			packet = types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, timeoutTimestamp)
			chanCap = suite.chainA.GetChannelCapability(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
		}, false},
		{"channel verification failed ORDERED", func() {
			ordered = true
			path.SetChannelOrdered()
			suite.coordinator.Setup(path)

			timeoutHeight := clienttypes.GetSelfHeight(suite.chainB.GetContext())
			timeoutTimestamp := uint64(suite.chainB.GetContext().BlockTime().UnixNano())

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, timeoutTimestamp, ibctesting.MockPacketData)
			suite.Require().NoError(err)
			packet = types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, timeoutTimestamp)
			chanCap = suite.chainA.GetChannelCapability(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
		}, false},
		{"next seq receive verification failed ORDERED", func() {
			// set ordered to false providing the wrong proof for ORDERED case
			ordered = false
			path.SetChannelOrdered()
			suite.coordinator.Setup(path)

			timeoutHeight := clienttypes.GetSelfHeight(suite.chainB.GetContext())
			timeoutTimestamp := uint64(suite.chainB.GetContext().BlockTime().UnixNano())

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, timeoutTimestamp, ibctesting.MockPacketData)
			suite.Require().NoError(err)
			err = path.EndpointB.SetChannelState(types.CLOSED)
			suite.Require().NoError(err)
			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			packet = types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, clienttypes.GetSelfHeight(suite.chainB.GetContext()), uint64(suite.chainB.GetContext().BlockTime().UnixNano()))
			chanCap = suite.chainA.GetChannelCapability(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
		}, false},
		{"packet ack verification failed", func() {
			// set ordered to true providing the wrong proof for UNORDERED case
			ordered = true
			suite.coordinator.Setup(path)

			timeoutHeight := clienttypes.GetSelfHeight(suite.chainB.GetContext())

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, disabledTimeoutTimestamp, ibctesting.MockPacketData)
			suite.Require().NoError(err)
			err = path.EndpointB.SetChannelState(types.CLOSED)
			suite.Require().NoError(err)
			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			packet = types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, disabledTimeoutTimestamp)
			chanCap = suite.chainA.GetChannelCapability(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
		}, false},
		{"channel capability not found ORDERED", func() {
			ordered = true
			path.SetChannelOrdered()
			suite.coordinator.Setup(path)

			timeoutHeight := clienttypes.GetSelfHeight(suite.chainB.GetContext())
			timeoutTimestamp := uint64(suite.chainB.GetContext().BlockTime().UnixNano())

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, timeoutTimestamp, ibctesting.MockPacketData)
			suite.Require().NoError(err)
			err = path.EndpointB.SetChannelState(types.CLOSED)
			suite.Require().NoError(err)
			// need to update chainA's client representing chainB to prove missing ack
			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			packet = types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, clienttypes.GetSelfHeight(suite.chainB.GetContext()), uint64(suite.chainB.GetContext().BlockTime().UnixNano()))
			chanCap = capabilitytypes.NewCapability(100)
		}, false},
		{
			"failure: invalid counterparty upgrade sequence",
			func() {
				ordered = false
				suite.coordinator.Setup(path)

				timeoutHeight := clienttypes.GetSelfHeight(suite.chainB.GetContext())

				sequence, err := path.EndpointA.SendPacket(timeoutHeight, disabledTimeoutTimestamp, ibctesting.MockPacketData)
				suite.Require().NoError(err)

				// trigger upgradeInit on B which will bump the counterparty upgrade sequence.
				path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = mock.UpgradeVersion
				err = path.EndpointB.ChanUpgradeInit()
				suite.Require().NoError(err)

				err = path.EndpointB.SetChannelState(types.CLOSED)
				suite.Require().NoError(err)

				// need to update chainA's client representing chainB to prove missing ack
				err = path.EndpointA.UpdateClient()
				suite.Require().NoError(err)

				packet = types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, disabledTimeoutTimestamp)
				chanCap = suite.chainA.GetChannelCapability(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
			},
			false,
		},
	}

	for i, tc := range testCases {
		tc := tc
		suite.Run(fmt.Sprintf("Case %s, %d/%d tests", tc.msg, i, len(testCases)), func() {
			var proof []byte

			suite.SetupTest()               // reset
			nextSeqRecv = 1                 // must be explicitly changed
			counterpartyUpgradeSequence = 0 // must be explicitly changed
			path = ibctesting.NewPath(suite.chainA, suite.chainB)

			tc.malleate()

			channelKey := host.ChannelKey(packet.GetDestPort(), packet.GetDestChannel())
			unorderedPacketKey := host.PacketReceiptKey(packet.GetDestPort(), packet.GetDestChannel(), packet.GetSequence())
			orderedPacketKey := host.NextSequenceRecvKey(packet.GetDestPort(), packet.GetDestChannel())

			closedProof, proofHeight := suite.chainB.QueryProof(channelKey)

			if ordered {
				proof, _ = suite.chainB.QueryProof(orderedPacketKey)
			} else {
				proof, _ = suite.chainB.QueryProof(unorderedPacketKey)
			}

			err := suite.chainA.App.GetIBCKeeper().ChannelKeeper.TimeoutOnCloseWithCounterpartyUpgradeSequence(
				suite.chainA.GetContext(),
				chanCap,
				packet,
				proof,
				closedProof,
				proofHeight,
				nextSeqRecv,
				counterpartyUpgradeSequence,
			)

			if tc.expPass {
				suite.Require().NoError(err)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}
