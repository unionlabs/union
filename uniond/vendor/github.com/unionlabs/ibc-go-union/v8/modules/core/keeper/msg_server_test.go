package keeper_test

import (
	"context"
	"errors"
	"fmt"

	upgradetypes "cosmossdk.io/x/upgrade/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"

	abci "github.com/cometbft/cometbft/abci/types"

	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	porttypes "github.com/cosmos/ibc-go/v8/modules/core/05-port/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	"github.com/cosmos/ibc-go/v8/modules/core/keeper"
	ibctm "github.com/cosmos/ibc-go/v8/modules/light-clients/07-tendermint"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
	ibcmock "github.com/cosmos/ibc-go/v8/testing/mock"
)

var (
	timeoutHeight = clienttypes.NewHeight(1, 10000)
	maxSequence   = uint64(10)
)

// tests the IBC handler receiving a packet on ordered and unordered channels.
// It verifies that the storing of an acknowledgement on success occurs. It
// tests high level properties like ordering and basic sanity checks. More
// rigorous testing of 'RecvPacket' can be found in the
// 04-channel/keeper/packet_test.go.
func (suite *KeeperTestSuite) TestHandleRecvPacket() {
	var (
		packet channeltypes.Packet
		path   *ibctesting.Path
	)

	testCases := []struct {
		name      string
		malleate  func()
		expPass   bool
		expRevert bool
		async     bool // indicate no ack written
		replay    bool // indicate replay (no-op)
	}{
		{"success: ORDERED", func() {
			path.SetChannelOrdered()
			suite.coordinator.Setup(path)

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, 0, ibctesting.MockPacketData)
			suite.Require().NoError(err)

			packet = channeltypes.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, 0)
		}, true, false, false, false},
		{"success: UNORDERED", func() {
			suite.coordinator.Setup(path)

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, 0, ibctesting.MockPacketData)
			suite.Require().NoError(err)

			packet = channeltypes.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, 0)
		}, true, false, false, false},
		{"success: UNORDERED out of order packet", func() {
			// setup uses an UNORDERED channel
			suite.coordinator.Setup(path)

			// attempts to receive packet with sequence 10 without receiving packet with sequence 1
			for i := uint64(1); i < 10; i++ {
				sequence, err := path.EndpointA.SendPacket(timeoutHeight, 0, ibctesting.MockPacketData)
				suite.Require().NoError(err)

				packet = channeltypes.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, 0)
			}
		}, true, false, false, false},
		{"success: OnRecvPacket callback returns revert=true", func() {
			suite.coordinator.Setup(path)

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, 0, ibctesting.MockFailPacketData)
			suite.Require().NoError(err)

			packet = channeltypes.NewPacket(ibctesting.MockFailPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, 0)
		}, true, true, false, false},
		{"success: ORDERED - async acknowledgement", func() {
			path.SetChannelOrdered()
			suite.coordinator.Setup(path)

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, 0, ibcmock.MockAsyncPacketData)
			suite.Require().NoError(err)

			packet = channeltypes.NewPacket(ibcmock.MockAsyncPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, 0)
		}, true, false, true, false},
		{"success: UNORDERED - async acknowledgement", func() {
			suite.coordinator.Setup(path)

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, 0, ibcmock.MockAsyncPacketData)
			suite.Require().NoError(err)

			packet = channeltypes.NewPacket(ibcmock.MockAsyncPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, 0)
		}, true, false, true, false},
		{"failure: ORDERED out of order packet", func() {
			path.SetChannelOrdered()
			suite.coordinator.Setup(path)

			// attempts to receive packet with sequence 10 without receiving packet with sequence 1
			for i := uint64(1); i < 10; i++ {
				sequence, err := path.EndpointA.SendPacket(timeoutHeight, 0, ibctesting.MockPacketData)
				suite.Require().NoError(err)

				packet = channeltypes.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, 0)
			}
		}, false, false, false, false},
		{"channel does not exist", func() {
			// any non-nil value of packet is valid
			suite.Require().NotNil(packet)
		}, false, false, false, false},
		{"packet not sent", func() {
			suite.coordinator.Setup(path)
			packet = channeltypes.NewPacket(ibctesting.MockPacketData, 1, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, 0)
		}, false, false, false, false},
		{"successful no-op: ORDERED - packet already received (replay)", func() {
			// mock will panic if application callback is called twice on the same packet
			path.SetChannelOrdered()
			suite.coordinator.Setup(path)

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, 0, ibctesting.MockPacketData)
			suite.Require().NoError(err)

			packet = channeltypes.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, 0)
			err = path.EndpointB.RecvPacket(packet)
			suite.Require().NoError(err)
		}, true, false, false, true},
		{"successful no-op: UNORDERED - packet already received (replay)", func() {
			// mock will panic if application callback is called twice on the same packet
			suite.coordinator.Setup(path)

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, 0, ibctesting.MockPacketData)
			suite.Require().NoError(err)

			packet = channeltypes.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, 0)
			err = path.EndpointB.RecvPacket(packet)
			suite.Require().NoError(err)
		}, true, false, false, true},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest() // reset
			path = ibctesting.NewPath(suite.chainA, suite.chainB)

			tc.malleate()

			var (
				proof       []byte
				proofHeight clienttypes.Height
			)

			// get proof of packet commitment from chainA
			packetKey := host.PacketCommitmentKey(packet.GetSourcePort(), packet.GetSourceChannel(), packet.GetSequence())
			if path.EndpointA.ChannelID != "" {
				proof, proofHeight = path.EndpointA.QueryProof(packetKey)
			}

			msg := channeltypes.NewMsgRecvPacket(packet, proof, proofHeight, suite.chainB.SenderAccount.GetAddress().String())

			ctx := suite.chainB.GetContext()
			_, err := keeper.Keeper.RecvPacket(*suite.chainB.App.GetIBCKeeper(), ctx, msg)

			events := ctx.EventManager().Events()

			if tc.expPass {
				suite.Require().NoError(err)

				// replay should not fail since it will be treated as a no-op
				_, err := keeper.Keeper.RecvPacket(*suite.chainB.App.GetIBCKeeper(), suite.chainB.GetContext(), msg)
				suite.Require().NoError(err)

				// check that callback state was handled correctly
				_, exists := suite.chainB.GetSimApp().ScopedIBCMockKeeper.GetCapability(suite.chainB.GetContext(), ibcmock.GetMockRecvCanaryCapabilityName(packet))
				if tc.expRevert {
					suite.Require().False(exists, "capability exists in store even after callback reverted")

					// context events should contain error events
					suite.Require().Contains(events, keeper.ConvertToErrorEvents(sdk.Events{ibcmock.NewMockRecvPacketEvent()})[0])
					suite.Require().NotContains(events, ibcmock.NewMockRecvPacketEvent())
				} else {
					suite.Require().True(exists, "callback state not persisted when revert is false")

					if tc.replay {
						// context should not contain application events
						suite.Require().NotContains(events, ibcmock.NewMockRecvPacketEvent())
						suite.Require().NotContains(events, keeper.ConvertToErrorEvents(sdk.Events{ibcmock.NewMockRecvPacketEvent()})[0])
					} else {
						// context events should contain application events
						suite.Require().Contains(events, ibcmock.NewMockRecvPacketEvent())
					}
				}

				// verify if ack was written
				ack, found := suite.chainB.App.GetIBCKeeper().ChannelKeeper.GetPacketAcknowledgement(suite.chainB.GetContext(), packet.GetDestPort(), packet.GetDestChannel(), packet.GetSequence())

				if tc.async {
					suite.Require().Nil(ack)
					suite.Require().False(found)

				} else {
					suite.Require().NotNil(ack)
					suite.Require().True(found)
				}
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestRecoverClient() {
	var msg *clienttypes.MsgRecoverClient

	testCases := []struct {
		name     string
		malleate func()
		expErr   error
	}{
		{
			"success: recover client",
			func() {},
			nil,
		},
		{
			"signer doesn't match authority",
			func() {
				msg.Signer = ibctesting.InvalidID
			},
			ibcerrors.ErrUnauthorized,
		},
		{
			"invalid subject client",
			func() {
				msg.SubjectClientId = ibctesting.InvalidID
			},
			clienttypes.ErrClientNotFound,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			subjectPath := ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupClients(subjectPath)
			subject := subjectPath.EndpointA.ClientID
			subjectClientState := suite.chainA.GetClientState(subject)

			substitutePath := ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupClients(substitutePath)
			substitute := substitutePath.EndpointA.ClientID

			// update substitute twice
			err := substitutePath.EndpointA.UpdateClient()
			suite.Require().NoError(err)
			err = substitutePath.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			tmClientState, ok := subjectClientState.(*ibctm.ClientState)
			suite.Require().True(ok)
			tmClientState.FrozenHeight = tmClientState.LatestHeight
			suite.chainA.App.GetIBCKeeper().ClientKeeper.SetClientState(suite.chainA.GetContext(), subject, tmClientState)

			msg = clienttypes.NewMsgRecoverClient(suite.chainA.App.GetIBCKeeper().GetAuthority(), subject, substitute)

			tc.malleate()

			_, err = keeper.Keeper.RecoverClient(*suite.chainA.App.GetIBCKeeper(), suite.chainA.GetContext(), msg)

			expPass := tc.expErr == nil
			if expPass {
				suite.Require().NoError(err)

				// Assert that client status is now Active
				clientStore := suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(suite.chainA.GetContext(), subjectPath.EndpointA.ClientID)
				tmClientState := subjectPath.EndpointA.GetClientState().(*ibctm.ClientState)
				suite.Require().Equal(tmClientState.Status(suite.chainA.GetContext(), clientStore, suite.chainA.App.AppCodec()), exported.Active)
			} else {
				suite.Require().Error(err)
				suite.Require().ErrorIs(err, tc.expErr)
			}
		})
	}
}

// tests the IBC handler acknowledgement of a packet on ordered and unordered
// channels. It verifies that the deletion of packet commitments from state
// occurs. It test high level properties like ordering and basic sanity
// checks. More rigorous testing of 'AcknowledgePacket'
// can be found in the 04-channel/keeper/packet_test.go.
func (suite *KeeperTestSuite) TestHandleAcknowledgePacket() {
	var (
		packet channeltypes.Packet
		path   *ibctesting.Path
	)

	testCases := []struct {
		name     string
		malleate func()
		expPass  bool
		replay   bool // indicate replay (no-op)
	}{
		{"success: ORDERED", func() {
			path.SetChannelOrdered()
			suite.coordinator.Setup(path)

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, 0, ibctesting.MockPacketData)
			suite.Require().NoError(err)

			packet = channeltypes.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, 0)
			err = path.EndpointB.RecvPacket(packet)
			suite.Require().NoError(err)
		}, true, false},
		{"success: UNORDERED", func() {
			suite.coordinator.Setup(path)

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, 0, ibctesting.MockPacketData)
			suite.Require().NoError(err)

			packet = channeltypes.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, 0)
			err = path.EndpointB.RecvPacket(packet)
			suite.Require().NoError(err)
		}, true, false},
		{"success: UNORDERED acknowledge out of order packet", func() {
			// setup uses an UNORDERED channel
			suite.coordinator.Setup(path)

			// attempts to acknowledge ack with sequence 10 without acknowledging ack with sequence 1 (removing packet commitment)
			for i := uint64(1); i < 10; i++ {
				sequence, err := path.EndpointA.SendPacket(timeoutHeight, 0, ibctesting.MockPacketData)
				suite.Require().NoError(err)

				packet = channeltypes.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, 0)
				err = path.EndpointB.RecvPacket(packet)
				suite.Require().NoError(err)
			}
		}, true, false},
		{"failure: ORDERED acknowledge out of order packet", func() {
			path.SetChannelOrdered()
			suite.coordinator.Setup(path)

			// attempts to acknowledge ack with sequence 10 without acknowledging ack with sequence 1 (removing packet commitment
			for i := uint64(1); i < 10; i++ {
				sequence, err := path.EndpointA.SendPacket(timeoutHeight, 0, ibctesting.MockPacketData)
				suite.Require().NoError(err)

				packet = channeltypes.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, 0)
				err = path.EndpointB.RecvPacket(packet)
				suite.Require().NoError(err)
			}
		}, false, false},
		{"channel does not exist", func() {
			// any non-nil value of packet is valid
			suite.Require().NotNil(packet)
		}, false, false},
		{"packet not received", func() {
			suite.coordinator.Setup(path)

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, 0, ibctesting.MockPacketData)
			suite.Require().NoError(err)

			packet = channeltypes.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, 0)
		}, false, false},
		{"successful no-op: ORDERED - packet already acknowledged (replay)", func() {
			suite.coordinator.Setup(path)

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, 0, ibctesting.MockPacketData)
			suite.Require().NoError(err)

			packet = channeltypes.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, 0)
			err = path.EndpointB.RecvPacket(packet)
			suite.Require().NoError(err)

			err = path.EndpointA.AcknowledgePacket(packet, ibctesting.MockAcknowledgement)
			suite.Require().NoError(err)
		}, true, true},
		{"successful no-op: UNORDERED - packet already acknowledged (replay)", func() {
			suite.coordinator.Setup(path)

			sequence, err := path.EndpointA.SendPacket(timeoutHeight, 0, ibctesting.MockPacketData)
			suite.Require().NoError(err)

			packet = channeltypes.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, 0)
			err = path.EndpointB.RecvPacket(packet)
			suite.Require().NoError(err)

			err = path.EndpointA.AcknowledgePacket(packet, ibctesting.MockAcknowledgement)
			suite.Require().NoError(err)
		}, true, true},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest() // reset
			path = ibctesting.NewPath(suite.chainA, suite.chainB)

			tc.malleate()

			var (
				proof       []byte
				proofHeight clienttypes.Height
			)
			packetKey := host.PacketAcknowledgementKey(packet.GetDestPort(), packet.GetDestChannel(), packet.GetSequence())
			if path.EndpointB.ChannelID != "" {
				proof, proofHeight = path.EndpointB.QueryProof(packetKey)
			}

			msg := channeltypes.NewMsgAcknowledgement(packet, ibcmock.MockAcknowledgement.Acknowledgement(), proof, proofHeight, suite.chainA.SenderAccount.GetAddress().String())

			ctx := suite.chainA.GetContext()
			_, err := keeper.Keeper.Acknowledgement(*suite.chainA.App.GetIBCKeeper(), ctx, msg)

			events := ctx.EventManager().Events()

			if tc.expPass {
				suite.Require().NoError(err)

				// verify packet commitment was deleted on source chain
				has := suite.chainA.App.GetIBCKeeper().ChannelKeeper.HasPacketCommitment(suite.chainA.GetContext(), packet.GetSourcePort(), packet.GetSourceChannel(), packet.GetSequence())
				suite.Require().False(has)

				// replay should not error as it is treated as a no-op
				_, err := keeper.Keeper.Acknowledgement(*suite.chainA.App.GetIBCKeeper(), suite.chainA.GetContext(), msg)
				suite.Require().NoError(err)

				if tc.replay {
					// context should not contain application events
					suite.Require().NotContains(events, ibcmock.NewMockAckPacketEvent())
				} else {
					// context events should contain application events
					suite.Require().Contains(events, ibcmock.NewMockAckPacketEvent())
				}
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

// tests the IBC handler timing out a packet on ordered and unordered channels.
// It verifies that the deletion of a packet commitment occurs. It tests
// high level properties like ordering and basic sanity checks. More
// rigorous testing of 'TimeoutPacket' and 'TimeoutExecuted' can be found in
// the 04-channel/keeper/timeout_test.go.
func (suite *KeeperTestSuite) TestHandleTimeoutPacket() {
	var (
		packet    channeltypes.Packet
		packetKey []byte
		path      *ibctesting.Path
	)

	testCases := []struct {
		name     string
		malleate func()
		expPass  bool
		noop     bool // indicate no-op
	}{
		{"success: ORDERED", func() {
			path.SetChannelOrdered()
			suite.coordinator.Setup(path)

			timeoutHeight := clienttypes.GetSelfHeight(suite.chainB.GetContext())
			timeoutTimestamp := uint64(suite.chainB.GetContext().BlockTime().UnixNano())

			// create packet commitment
			sequence, err := path.EndpointA.SendPacket(timeoutHeight, timeoutTimestamp, ibctesting.MockPacketData)
			suite.Require().NoError(err)

			// need to update chainA client to prove missing ack
			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			packet = channeltypes.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, timeoutTimestamp)
			packetKey = host.NextSequenceRecvKey(packet.GetDestPort(), packet.GetDestChannel())
		}, true, false},
		{"success: UNORDERED", func() {
			suite.coordinator.Setup(path)

			timeoutHeight := clienttypes.GetSelfHeight(suite.chainB.GetContext())
			timeoutTimestamp := uint64(suite.chainB.GetContext().BlockTime().UnixNano())

			// create packet commitment
			sequence, err := path.EndpointA.SendPacket(timeoutHeight, timeoutTimestamp, ibctesting.MockPacketData)
			suite.Require().NoError(err)

			// need to update chainA client to prove missing ack
			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			packet = channeltypes.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, timeoutTimestamp)
			packetKey = host.PacketReceiptKey(packet.GetDestPort(), packet.GetDestChannel(), packet.GetSequence())
		}, true, false},
		{"success: UNORDERED timeout out of order packet", func() {
			// setup uses an UNORDERED channel
			suite.coordinator.Setup(path)

			// attempts to timeout the last packet sent without timing out the first packet
			// packet sequences begin at 1
			for i := uint64(1); i < maxSequence; i++ {
				timeoutHeight := clienttypes.GetSelfHeight(suite.chainB.GetContext())

				// create packet commitment
				sequence, err := path.EndpointA.SendPacket(timeoutHeight, 0, ibctesting.MockPacketData)
				suite.Require().NoError(err)

				packet = channeltypes.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, 0)
			}

			err := path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			packetKey = host.PacketReceiptKey(packet.GetDestPort(), packet.GetDestChannel(), packet.GetSequence())
		}, true, false},
		{"success: ORDERED timeout out of order packet", func() {
			path.SetChannelOrdered()
			suite.coordinator.Setup(path)

			// attempts to timeout the last packet sent without timing out the first packet
			// packet sequences begin at 1
			for i := uint64(1); i < maxSequence; i++ {
				timeoutHeight := clienttypes.GetSelfHeight(suite.chainB.GetContext())

				// create packet commitment
				sequence, err := path.EndpointA.SendPacket(timeoutHeight, 0, ibctesting.MockPacketData)
				suite.Require().NoError(err)

				packet = channeltypes.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, 0)
			}

			err := path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			packetKey = host.NextSequenceRecvKey(packet.GetDestPort(), packet.GetDestChannel())
		}, true, false},
		{"channel does not exist", func() {
			// any non-nil value of packet is valid
			suite.Require().NotNil(packet)

			packetKey = host.NextSequenceRecvKey(packet.GetDestPort(), packet.GetDestChannel())
		}, false, false},
		{"successful no-op: UNORDERED - packet not sent", func() {
			suite.coordinator.Setup(path)

			packet = channeltypes.NewPacket(ibctesting.MockPacketData, 1, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, clienttypes.NewHeight(0, 1), 0)
			packetKey = host.PacketReceiptKey(packet.GetDestPort(), packet.GetDestChannel(), packet.GetSequence())
		}, true, true},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest() // reset
			path = ibctesting.NewPath(suite.chainA, suite.chainB)

			tc.malleate()

			var (
				proof       []byte
				proofHeight clienttypes.Height
			)
			if path.EndpointB.ChannelID != "" {
				proof, proofHeight = path.EndpointB.QueryProof(packetKey)
			}

			msg := channeltypes.NewMsgTimeout(packet, 1, proof, proofHeight, suite.chainA.SenderAccount.GetAddress().String())

			ctx := suite.chainA.GetContext()
			_, err := keeper.Keeper.Timeout(*suite.chainA.App.GetIBCKeeper(), ctx, msg)

			events := ctx.EventManager().Events()

			if tc.expPass {
				suite.Require().NoError(err)

				// replay should not return an error as it is treated as a no-op
				_, err := keeper.Keeper.Timeout(*suite.chainA.App.GetIBCKeeper(), suite.chainA.GetContext(), msg)
				suite.Require().NoError(err)

				// verify packet commitment was deleted on source chain
				has := suite.chainA.App.GetIBCKeeper().ChannelKeeper.HasPacketCommitment(suite.chainA.GetContext(), packet.GetSourcePort(), packet.GetSourceChannel(), packet.GetSequence())
				suite.Require().False(has)

				if tc.noop {
					// context should not contain application events
					suite.Require().NotContains(events, ibcmock.NewMockTimeoutPacketEvent())
				} else {
					// context should contain application events
					suite.Require().Contains(events, ibcmock.NewMockTimeoutPacketEvent())
				}

			} else {
				suite.Require().Error(err)
			}
		})
	}
}

// tests the IBC handler timing out a packet via channel closure on ordered
// and unordered channels. It verifies that the deletion of a packet
// commitment occurs. It tests high level properties like ordering and basic
// sanity checks. More rigorous testing of 'TimeoutOnClose' and
// 'TimeoutExecuted' can be found in the 04-channel/keeper/timeout_test.go.
func (suite *KeeperTestSuite) TestHandleTimeoutOnClosePacket() {
	var (
		packet                      channeltypes.Packet
		packetKey                   []byte
		path                        *ibctesting.Path
		counterpartyUpgradeSequence uint64
	)

	testCases := []struct {
		name     string
		malleate func()
		expPass  bool
	}{
		{"success: ORDERED", func() {
			path.SetChannelOrdered()
			suite.coordinator.Setup(path)

			// create packet commitment
			sequence, err := path.EndpointA.SendPacket(timeoutHeight, 0, ibctesting.MockPacketData)
			suite.Require().NoError(err)

			// need to update chainA client to prove missing ack
			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			packet = channeltypes.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, 0)
			packetKey = host.NextSequenceRecvKey(packet.GetDestPort(), packet.GetDestChannel())

			// close counterparty channel
			err = path.EndpointB.SetChannelState(channeltypes.CLOSED)
			suite.Require().NoError(err)
		}, true},
		{"success: UNORDERED", func() {
			suite.coordinator.Setup(path)

			// create packet commitment
			sequence, err := path.EndpointA.SendPacket(timeoutHeight, 0, ibctesting.MockPacketData)
			suite.Require().NoError(err)

			// need to update chainA client to prove missing ack
			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			packet = channeltypes.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, 0)
			packetKey = host.PacketReceiptKey(packet.GetDestPort(), packet.GetDestChannel(), packet.GetSequence())

			// close counterparty channel
			err = path.EndpointB.SetChannelState(channeltypes.CLOSED)
			suite.Require().NoError(err)
		}, true},
		{"success: UNORDERED timeout out of order packet", func() {
			// setup uses an UNORDERED channel
			suite.coordinator.Setup(path)

			// attempts to timeout the last packet sent without timing out the first packet
			// packet sequences begin at 1
			for i := uint64(1); i < maxSequence; i++ {
				// create packet commitment
				sequence, err := path.EndpointA.SendPacket(timeoutHeight, 0, ibctesting.MockPacketData)
				suite.Require().NoError(err)

				packet = channeltypes.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, 0)
			}

			err := path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			packetKey = host.PacketReceiptKey(packet.GetDestPort(), packet.GetDestChannel(), packet.GetSequence())

			// close counterparty channel
			err = path.EndpointB.SetChannelState(channeltypes.CLOSED)
			suite.Require().NoError(err)
		}, true},
		{"success: ORDERED timeout out of order packet", func() {
			path.SetChannelOrdered()
			suite.coordinator.Setup(path)

			// attempts to timeout the last packet sent without timing out the first packet
			// packet sequences begin at 1
			for i := uint64(1); i < maxSequence; i++ {
				// create packet commitment
				sequence, err := path.EndpointA.SendPacket(timeoutHeight, 0, ibctesting.MockPacketData)
				suite.Require().NoError(err)

				packet = channeltypes.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, 0)
			}

			err := path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			packetKey = host.NextSequenceRecvKey(packet.GetDestPort(), packet.GetDestChannel())

			// close counterparty channel
			err = path.EndpointB.SetChannelState(channeltypes.CLOSED)
			suite.Require().NoError(err)
		}, true},
		{"channel does not exist", func() {
			// any non-nil value of packet is valid
			suite.Require().NotNil(packet)

			packetKey = host.NextSequenceRecvKey(packet.GetDestPort(), packet.GetDestChannel())
		}, false},
		{"successful no-op: UNORDERED - packet not sent", func() {
			suite.coordinator.Setup(path)
			packet = channeltypes.NewPacket(ibctesting.MockPacketData, 1, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, clienttypes.NewHeight(0, 1), 0)
			packetKey = host.PacketAcknowledgementKey(packet.GetDestPort(), packet.GetDestChannel(), packet.GetSequence())

			// close counterparty channel
			err := path.EndpointB.SetChannelState(channeltypes.CLOSED)
			suite.Require().NoError(err)
		}, true},
		{"ORDERED: channel not closed", func() {
			path.SetChannelOrdered()
			suite.coordinator.Setup(path)

			// create packet commitment
			sequence, err := path.EndpointA.SendPacket(timeoutHeight, 0, ibctesting.MockPacketData)
			suite.Require().NoError(err)

			// need to update chainA client to prove missing ack
			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			packet = channeltypes.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, 0)
			packetKey = host.NextSequenceRecvKey(packet.GetDestPort(), packet.GetDestChannel())
		}, false},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest() // reset
			path = ibctesting.NewPath(suite.chainA, suite.chainB)

			tc.malleate()

			proof, proofHeight := suite.chainB.QueryProof(packetKey)

			channelKey := host.ChannelKey(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
			closedProof, _ := suite.chainB.QueryProof(channelKey)

			msg := channeltypes.NewMsgTimeoutOnCloseWithCounterpartyUpgradeSequence(packet, 1, proof, closedProof, proofHeight, suite.chainA.SenderAccount.GetAddress().String(), counterpartyUpgradeSequence)

			_, err := keeper.Keeper.TimeoutOnClose(*suite.chainA.App.GetIBCKeeper(), suite.chainA.GetContext(), msg)

			if tc.expPass {
				suite.Require().NoError(err)

				// replay should not return an error as it will be treated as a no-op
				_, err := keeper.Keeper.TimeoutOnClose(*suite.chainA.App.GetIBCKeeper(), suite.chainA.GetContext(), msg)
				suite.Require().NoError(err)

				// verify packet commitment was deleted on source chain
				has := suite.chainA.App.GetIBCKeeper().ChannelKeeper.HasPacketCommitment(suite.chainA.GetContext(), packet.GetSourcePort(), packet.GetSourceChannel(), packet.GetSequence())
				suite.Require().False(has)

			} else {
				suite.Require().Error(err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestUpgradeClient() {
	var (
		path              *ibctesting.Path
		newChainID        string
		newClientHeight   clienttypes.Height
		upgradedClient    exported.ClientState
		upgradedConsState exported.ConsensusState
		lastHeight        exported.Height
		msg               *clienttypes.MsgUpgradeClient
	)
	cases := []struct {
		name    string
		setup   func()
		expPass bool
	}{
		{
			name: "successful upgrade",
			setup: func() {
				upgradedClient = ibctm.NewClientState(newChainID, ibctm.DefaultTrustLevel, ibctesting.TrustingPeriod, ibctesting.UnbondingPeriod+ibctesting.TrustingPeriod, ibctesting.MaxClockDrift, newClientHeight, commitmenttypes.GetSDKSpecs(), ibctesting.UpgradePath)
				// Call ZeroCustomFields on upgraded clients to clear any client-chosen parameters in test-case upgradedClient
				upgradedClient = upgradedClient.ZeroCustomFields()

				upgradedConsState = &ibctm.ConsensusState{
					NextValidatorsHash: []byte("nextValsHash"),
				}

				// last Height is at next block
				lastHeight = clienttypes.NewHeight(0, uint64(suite.chainB.GetContext().BlockHeight()+1))

				upgradedClientBz, err := clienttypes.MarshalClientState(suite.chainA.App.AppCodec(), upgradedClient)
				suite.Require().NoError(err)
				upgradedConsStateBz, err := clienttypes.MarshalConsensusState(suite.chainA.App.AppCodec(), upgradedConsState)
				suite.Require().NoError(err)

				// zero custom fields and store in upgrade store
				suite.chainB.GetSimApp().UpgradeKeeper.SetUpgradedClient(suite.chainB.GetContext(), int64(lastHeight.GetRevisionHeight()), upgradedClientBz)            //nolint:errcheck // ignore error for testing
				suite.chainB.GetSimApp().UpgradeKeeper.SetUpgradedConsensusState(suite.chainB.GetContext(), int64(lastHeight.GetRevisionHeight()), upgradedConsStateBz) //nolint:errcheck // ignore error for testing

				// commit upgrade store changes and update clients
				suite.coordinator.CommitBlock(suite.chainB)
				err = path.EndpointA.UpdateClient()
				suite.Require().NoError(err)

				cs, found := suite.chainA.App.GetIBCKeeper().ClientKeeper.GetClientState(suite.chainA.GetContext(), path.EndpointA.ClientID)
				suite.Require().True(found)

				upgradeClientProof, _ := suite.chainB.QueryUpgradeProof(upgradetypes.UpgradedClientKey(int64(lastHeight.GetRevisionHeight())), cs.GetLatestHeight().GetRevisionHeight())
				upgradedConsensusStateProof, _ := suite.chainB.QueryUpgradeProof(upgradetypes.UpgradedConsStateKey(int64(lastHeight.GetRevisionHeight())), cs.GetLatestHeight().GetRevisionHeight())

				msg, err = clienttypes.NewMsgUpgradeClient(path.EndpointA.ClientID, upgradedClient, upgradedConsState,
					upgradeClientProof, upgradedConsensusStateProof, suite.chainA.SenderAccount.GetAddress().String())
				suite.Require().NoError(err)
			},
			expPass: true,
		},
		{
			name: "VerifyUpgrade fails",
			setup: func() {
				upgradedClient = ibctm.NewClientState(newChainID, ibctm.DefaultTrustLevel, ibctesting.TrustingPeriod, ibctesting.UnbondingPeriod+ibctesting.TrustingPeriod, ibctesting.MaxClockDrift, newClientHeight, commitmenttypes.GetSDKSpecs(), ibctesting.UpgradePath)
				// Call ZeroCustomFields on upgraded clients to clear any client-chosen parameters in test-case upgradedClient
				upgradedClient = upgradedClient.ZeroCustomFields()

				upgradedConsState = &ibctm.ConsensusState{
					NextValidatorsHash: []byte("nextValsHash"),
				}

				// last Height is at next block
				lastHeight = clienttypes.NewHeight(0, uint64(suite.chainB.GetContext().BlockHeight()+1))

				upgradedClientBz, err := clienttypes.MarshalClientState(suite.chainA.App.AppCodec(), upgradedClient)
				suite.Require().NoError(err)
				upgradedConsStateBz, err := clienttypes.MarshalConsensusState(suite.chainA.App.AppCodec(), upgradedConsState)
				suite.Require().NoError(err)

				// zero custom fields and store in upgrade store
				suite.chainB.GetSimApp().UpgradeKeeper.SetUpgradedClient(suite.chainB.GetContext(), int64(lastHeight.GetRevisionHeight()), upgradedClientBz)            //nolint:errcheck // ignore error for testing
				suite.chainB.GetSimApp().UpgradeKeeper.SetUpgradedConsensusState(suite.chainB.GetContext(), int64(lastHeight.GetRevisionHeight()), upgradedConsStateBz) //nolint:errcheck // ignore error for testing

				// commit upgrade store changes and update clients
				suite.coordinator.CommitBlock(suite.chainB)
				err = path.EndpointA.UpdateClient()
				suite.Require().NoError(err)

				msg, err = clienttypes.NewMsgUpgradeClient(path.EndpointA.ClientID, upgradedClient, upgradedConsState, nil, nil, suite.chainA.SenderAccount.GetAddress().String())
				suite.Require().NoError(err)
			},
			expPass: false,
		},
	}

	for _, tc := range cases {
		tc := tc
		path = ibctesting.NewPath(suite.chainA, suite.chainB)
		suite.coordinator.SetupClients(path)

		var err error
		clientState := path.EndpointA.GetClientState().(*ibctm.ClientState)
		revisionNumber := clienttypes.ParseChainID(clientState.ChainId)

		newChainID, err = clienttypes.SetRevisionNumber(clientState.ChainId, revisionNumber+1)
		suite.Require().NoError(err)

		newClientHeight = clienttypes.NewHeight(revisionNumber+1, clientState.GetLatestHeight().GetRevisionHeight()+1)

		tc.setup()

		_, err = keeper.Keeper.UpgradeClient(*suite.chainA.App.GetIBCKeeper(), suite.chainA.GetContext(), msg)

		if tc.expPass {
			suite.Require().NoError(err, "upgrade handler failed on valid case: %s", tc.name)
			newClient, ok := suite.chainA.App.GetIBCKeeper().ClientKeeper.GetClientState(suite.chainA.GetContext(), path.EndpointA.ClientID)
			suite.Require().True(ok)
			newChainSpecifiedClient := newClient.ZeroCustomFields()
			suite.Require().Equal(upgradedClient, newChainSpecifiedClient)
		} else {
			suite.Require().Error(err, "upgrade handler passed on invalid case: %s", tc.name)
		}
	}
}

func (suite *KeeperTestSuite) TestChannelUpgradeInit() {
	var (
		path *ibctesting.Path
		msg  *channeltypes.MsgChannelUpgradeInit
	)

	cases := []struct {
		name      string
		malleate  func()
		expResult func(res *channeltypes.MsgChannelUpgradeInitResponse, events []abci.Event, err error)
	}{
		{
			"success",
			func() {
				msg = channeltypes.NewMsgChannelUpgradeInit(
					path.EndpointA.ChannelConfig.PortID,
					path.EndpointA.ChannelID,
					path.EndpointA.GetProposedUpgrade().Fields,
					path.EndpointA.Chain.GetSimApp().IBCKeeper.GetAuthority(),
				)
			},
			func(res *channeltypes.MsgChannelUpgradeInitResponse, events []abci.Event, err error) {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)
				suite.Require().Equal(uint64(1), res.UpgradeSequence)

				upgrade := path.EndpointA.GetChannelUpgrade()
				channel := path.EndpointA.GetChannel()

				expEvents := ibctesting.EventsMap{
					channeltypes.EventTypeChannelUpgradeInit: {
						channeltypes.AttributeKeyPortID:             path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointA.ChannelID,
						channeltypes.AttributeCounterpartyPortID:    path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointB.ChannelID,
						channeltypes.AttributeKeyConnectionHops:     upgrade.Fields.ConnectionHops[0],
						channeltypes.AttributeVersion:               upgrade.Fields.Version,
						channeltypes.AttributeKeyOrdering:           upgrade.Fields.Ordering.String(),
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
					},
					sdk.EventTypeMessage: {
						sdk.AttributeKeyModule: channeltypes.AttributeValueCategory,
					},
				}
				ibctesting.AssertEventsLegacy(&suite.Suite, expEvents, events)
			},
		},
		{
			"authority is not signer of the upgrade init msg",
			func() {
				msg = channeltypes.NewMsgChannelUpgradeInit(
					path.EndpointA.ChannelConfig.PortID,
					path.EndpointA.ChannelID,
					path.EndpointA.GetProposedUpgrade().Fields,
					path.EndpointA.Chain.SenderAccount.String(),
				)
			},
			func(res *channeltypes.MsgChannelUpgradeInitResponse, events []abci.Event, err error) {
				suite.Require().Error(err)
				suite.Require().ErrorContains(err, ibcerrors.ErrUnauthorized.Error())
				suite.Require().Nil(res)

				suite.Require().Empty(events)
			},
		},
		{
			"ibc application does not implement the UpgradeableModule interface",
			func() {
				path = ibctesting.NewPath(suite.chainA, suite.chainB)
				path.EndpointA.ChannelConfig.PortID = ibcmock.MockBlockUpgrade
				path.EndpointB.ChannelConfig.PortID = ibcmock.MockBlockUpgrade

				suite.coordinator.Setup(path)

				msg = channeltypes.NewMsgChannelUpgradeInit(
					path.EndpointA.ChannelConfig.PortID,
					path.EndpointA.ChannelID,
					path.EndpointA.GetProposedUpgrade().Fields,
					path.EndpointA.Chain.GetSimApp().IBCKeeper.GetAuthority(),
				)
			},
			func(res *channeltypes.MsgChannelUpgradeInitResponse, events []abci.Event, err error) {
				suite.Require().ErrorIs(err, porttypes.ErrInvalidRoute)
				suite.Require().Nil(res)

				suite.Require().Empty(events)
			},
		},
		{
			"ibc application does not commit state changes in callback",
			func() {
				msg = channeltypes.NewMsgChannelUpgradeInit(
					path.EndpointA.ChannelConfig.PortID,
					path.EndpointA.ChannelID,
					path.EndpointA.GetProposedUpgrade().Fields,
					path.EndpointA.Chain.GetSimApp().IBCKeeper.GetAuthority(),
				)

				suite.chainA.GetSimApp().IBCMockModule.IBCApp.OnChanUpgradeInit = func(ctx context.Context, portID, channelID string, order channeltypes.Order, connectionHops []string, version string) (string, error) {
					storeKey := suite.chainA.GetSimApp().GetKey(exported.ModuleName)
					sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/5917
					store := sdkCtx.KVStore(storeKey)
					store.Set(ibcmock.TestKey, ibcmock.TestValue)

					sdkCtx.EventManager().EmitEvent(sdk.NewEvent(ibcmock.MockEventType))
					return ibcmock.UpgradeVersion, nil
				}
			},
			func(res *channeltypes.MsgChannelUpgradeInitResponse, events []abci.Event, err error) {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)
				suite.Require().Equal(uint64(1), res.UpgradeSequence)

				storeKey := suite.chainA.GetSimApp().GetKey(exported.ModuleName)
				store := suite.chainA.GetContext().KVStore(storeKey)
				suite.Require().Nil(store.Get(ibcmock.TestKey))

				for _, event := range events {
					if event.GetType() == ibcmock.MockEventType {
						suite.Fail("expected application callback events to be discarded")
					}
				}
			},
		},
	}

	for _, tc := range cases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			// configure the channel upgrade version on testing endpoints
			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = ibcmock.UpgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = ibcmock.UpgradeVersion

			tc.malleate()

			ctx := suite.chainA.GetContext()
			res, err := suite.chainA.GetSimApp().GetIBCKeeper().ChannelUpgradeInit(ctx, msg)
			events := ctx.EventManager().Events().ToABCIEvents()

			tc.expResult(res, events, err)
		})
	}
}

func (suite *KeeperTestSuite) TestChannelUpgradeTry() {
	var (
		path *ibctesting.Path
		msg  *channeltypes.MsgChannelUpgradeTry
	)

	cases := []struct {
		name      string
		malleate  func()
		expResult func(res *channeltypes.MsgChannelUpgradeTryResponse, events []abci.Event, err error)
	}{
		{
			"success",
			func() {},
			func(res *channeltypes.MsgChannelUpgradeTryResponse, events []abci.Event, err error) {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)
				suite.Require().Equal(channeltypes.SUCCESS, res.Result)

				channel := path.EndpointB.GetChannel()
				suite.Require().Equal(channeltypes.FLUSHING, channel.State)
				suite.Require().Equal(uint64(1), channel.UpgradeSequence)

				upgrade := path.EndpointB.GetChannelUpgrade()

				expEvents := ibctesting.EventsMap{
					channeltypes.EventTypeChannelUpgradeTry: {
						channeltypes.AttributeKeyPortID:             path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointB.ChannelID,
						channeltypes.AttributeCounterpartyPortID:    path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointA.ChannelID,
						channeltypes.AttributeKeyConnectionHops:     upgrade.Fields.ConnectionHops[0],
						channeltypes.AttributeVersion:               upgrade.Fields.Version,
						channeltypes.AttributeKeyOrdering:           upgrade.Fields.Ordering.String(),
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
					},
					sdk.EventTypeMessage: {
						sdk.AttributeKeyModule: channeltypes.AttributeValueCategory,
					},
				}
				ibctesting.AssertEventsLegacy(&suite.Suite, expEvents, events)
			},
		},
		{
			"module capability not found",
			func() {
				msg.PortId = "invalid-port"
				msg.ChannelId = "invalid-channel"
			},
			func(res *channeltypes.MsgChannelUpgradeTryResponse, events []abci.Event, err error) {
				suite.Require().Error(err)
				suite.Require().Nil(res)

				suite.Require().ErrorIs(err, capabilitytypes.ErrCapabilityNotFound)
				suite.Require().Empty(events)
			},
		},
		{
			"unsynchronized upgrade sequence writes upgrade error receipt",
			func() {
				channel := path.EndpointB.GetChannel()
				channel.UpgradeSequence = 99

				path.EndpointB.SetChannel(channel)
			},
			func(res *channeltypes.MsgChannelUpgradeTryResponse, events []abci.Event, err error) {
				suite.Require().NoError(err)

				suite.Require().NotNil(res)
				suite.Require().Equal(channeltypes.FAILURE, res.Result)

				errorReceipt, found := suite.chainB.GetSimApp().GetIBCKeeper().ChannelKeeper.GetUpgradeErrorReceipt(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
				suite.Require().True(found)
				suite.Require().Equal(uint64(99), errorReceipt.Sequence)

				channel := path.EndpointB.GetChannel()

				expEvents := ibctesting.EventsMap{
					channeltypes.EventTypeChannelUpgradeError: {
						channeltypes.AttributeKeyPortID:             path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointB.ChannelID,
						channeltypes.AttributeCounterpartyPortID:    path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointA.ChannelID,
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
						// need to manually insert this because the errorReceipt is a string constant as it is written into state
						channeltypes.AttributeKeyErrorReceipt: "counterparty upgrade sequence < current upgrade sequence (1 < 99): invalid upgrade sequence",
					},
					sdk.EventTypeMessage: {
						sdk.AttributeKeyModule: channeltypes.AttributeValueCategory,
					},
				}
				ibctesting.AssertEventsLegacy(&suite.Suite, expEvents, events)
			},
		},
		{
			"ibc application does not implement the UpgradeableModule interface",
			func() {
				path = ibctesting.NewPath(suite.chainA, suite.chainB)
				path.EndpointA.ChannelConfig.PortID = ibcmock.MockBlockUpgrade
				path.EndpointB.ChannelConfig.PortID = ibcmock.MockBlockUpgrade

				suite.coordinator.Setup(path)

				msg.PortId = path.EndpointB.ChannelConfig.PortID
				msg.ChannelId = path.EndpointB.ChannelID
			},
			func(res *channeltypes.MsgChannelUpgradeTryResponse, events []abci.Event, err error) {
				suite.Require().ErrorIs(err, porttypes.ErrInvalidRoute)
				suite.Require().Nil(res)

				suite.Require().Empty(events)
			},
		},
		{
			"ibc application does not commit state changes in callback",
			func() {
				suite.chainA.GetSimApp().IBCMockModule.IBCApp.OnChanUpgradeTry = func(ctx context.Context, portID, channelID string, order channeltypes.Order, connectionHops []string, counterpartyVersion string) (string, error) {
					storeKey := suite.chainA.GetSimApp().GetKey(exported.ModuleName)
					sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/5917
					store := sdkCtx.KVStore(storeKey)
					store.Set(ibcmock.TestKey, ibcmock.TestValue)

					sdkCtx.EventManager().EmitEvent(sdk.NewEvent(ibcmock.MockEventType))
					return ibcmock.UpgradeVersion, nil
				}
			},
			func(res *channeltypes.MsgChannelUpgradeTryResponse, events []abci.Event, err error) {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)
				suite.Require().Equal(uint64(1), res.UpgradeSequence)

				storeKey := suite.chainA.GetSimApp().GetKey(exported.ModuleName)
				store := suite.chainA.GetContext().KVStore(storeKey)
				suite.Require().Nil(store.Get(ibcmock.TestKey))

				for _, event := range events {
					if event.GetType() == ibcmock.MockEventType {
						suite.Fail("expected application callback events to be discarded")
					}
				}
			},
		},
	}

	for _, tc := range cases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			// configure the channel upgrade version on testing endpoints
			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = ibcmock.UpgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = ibcmock.UpgradeVersion

			err := path.EndpointA.ChanUpgradeInit()
			suite.Require().NoError(err)

			err = path.EndpointB.UpdateClient()
			suite.Require().NoError(err)

			counterpartySequence := path.EndpointA.GetChannel().UpgradeSequence
			counterpartyUpgrade, found := suite.chainA.GetSimApp().GetIBCKeeper().ChannelKeeper.GetUpgrade(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
			suite.Require().True(found)

			channelProof, upgradeProof, proofHeight := path.EndpointA.QueryChannelUpgradeProof()

			msg = &channeltypes.MsgChannelUpgradeTry{
				PortId:                        path.EndpointB.ChannelConfig.PortID,
				ChannelId:                     path.EndpointB.ChannelID,
				ProposedUpgradeConnectionHops: []string{ibctesting.FirstConnectionID},
				CounterpartyUpgradeSequence:   counterpartySequence,
				CounterpartyUpgradeFields:     counterpartyUpgrade.Fields,
				ProofChannel:                  channelProof,
				ProofUpgrade:                  upgradeProof,
				ProofHeight:                   proofHeight,
				Signer:                        suite.chainB.SenderAccount.GetAddress().String(),
			}

			tc.malleate()

			ctx := suite.chainB.GetContext()
			res, err := suite.chainB.GetSimApp().GetIBCKeeper().ChannelUpgradeTry(ctx, msg)
			events := ctx.EventManager().Events().ToABCIEvents()

			tc.expResult(res, events, err)
		})
	}
}

func (suite *KeeperTestSuite) TestChannelUpgradeAck() {
	var (
		path *ibctesting.Path
		msg  *channeltypes.MsgChannelUpgradeAck
	)

	cases := []struct {
		name      string
		malleate  func()
		expResult func(res *channeltypes.MsgChannelUpgradeAckResponse, events []abci.Event, err error)
	}{
		{
			"success, no pending in-flight packets",
			func() {},
			func(res *channeltypes.MsgChannelUpgradeAckResponse, events []abci.Event, err error) {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)
				suite.Require().Equal(channeltypes.SUCCESS, res.Result)

				channel := path.EndpointA.GetChannel()
				suite.Require().Equal(channeltypes.FLUSHCOMPLETE, channel.State)
				suite.Require().Equal(uint64(1), channel.UpgradeSequence)

				upgrade := path.EndpointA.GetChannelUpgrade()

				expEvents := ibctesting.EventsMap{
					channeltypes.EventTypeChannelUpgradeAck: {
						channeltypes.AttributeKeyPortID:             path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointA.ChannelID,
						channeltypes.AttributeCounterpartyPortID:    path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointB.ChannelID,
						channeltypes.AttributeKeyConnectionHops:     upgrade.Fields.ConnectionHops[0],
						channeltypes.AttributeVersion:               upgrade.Fields.Version,
						channeltypes.AttributeKeyOrdering:           upgrade.Fields.Ordering.String(),
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
					},
					sdk.EventTypeMessage: {
						sdk.AttributeKeyModule: channeltypes.AttributeValueCategory,
					},
				}
				ibctesting.AssertEventsLegacy(&suite.Suite, expEvents, events)
			},
		},
		{
			"success, pending in-flight packets",
			func() {
				portID := path.EndpointA.ChannelConfig.PortID
				channelID := path.EndpointA.ChannelID
				// Set a dummy packet commitment to simulate in-flight packets
				suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.SetPacketCommitment(suite.chainA.GetContext(), portID, channelID, 1, []byte("hash"))
			},
			func(res *channeltypes.MsgChannelUpgradeAckResponse, events []abci.Event, err error) {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)
				suite.Require().Equal(channeltypes.SUCCESS, res.Result)

				channel := path.EndpointA.GetChannel()
				suite.Require().Equal(channeltypes.FLUSHING, channel.State)
				suite.Require().Equal(uint64(1), channel.UpgradeSequence)

				upgrade := path.EndpointA.GetChannelUpgrade()

				expEvents := ibctesting.EventsMap{
					channeltypes.EventTypeChannelUpgradeAck: {
						channeltypes.AttributeKeyPortID:             path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointA.ChannelID,
						channeltypes.AttributeCounterpartyPortID:    path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointB.ChannelID,
						channeltypes.AttributeKeyConnectionHops:     upgrade.Fields.ConnectionHops[0],
						channeltypes.AttributeVersion:               upgrade.Fields.Version,
						channeltypes.AttributeKeyOrdering:           upgrade.Fields.Ordering.String(),
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
					},
					sdk.EventTypeMessage: {
						sdk.AttributeKeyModule: channeltypes.AttributeValueCategory,
					},
				}
				ibctesting.AssertEventsLegacy(&suite.Suite, expEvents, events)
			},
		},
		{
			"module capability not found",
			func() {
				msg.PortId = ibctesting.InvalidID
				msg.ChannelId = ibctesting.InvalidID
			},
			func(res *channeltypes.MsgChannelUpgradeAckResponse, events []abci.Event, err error) {
				suite.Require().Error(err)
				suite.Require().Nil(res)

				suite.Require().ErrorIs(err, capabilitytypes.ErrCapabilityNotFound)
				suite.Require().Empty(events)
			},
		},
		{
			"core handler returns error and no upgrade error receipt is written",
			func() {
				// force an error by overriding the channel state to an invalid value
				channel := path.EndpointA.GetChannel()
				channel.State = channeltypes.CLOSED

				path.EndpointA.SetChannel(channel)
			},
			func(res *channeltypes.MsgChannelUpgradeAckResponse, events []abci.Event, err error) {
				suite.Require().Error(err)
				suite.Require().Nil(res)
				suite.Require().ErrorIs(err, channeltypes.ErrInvalidChannelState)

				errorReceipt, found := suite.chainA.GetSimApp().GetIBCKeeper().ChannelKeeper.GetUpgradeErrorReceipt(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().Empty(errorReceipt)
				suite.Require().False(found)

				suite.Require().Empty(events)
			},
		},
		{
			"core handler returns error and writes upgrade error receipt",
			func() {
				// force an upgrade error by modifying the channel upgrade ordering to an incompatible value
				upgrade := path.EndpointA.GetChannelUpgrade()
				upgrade.Fields.Ordering = channeltypes.NONE

				path.EndpointA.SetChannelUpgrade(upgrade)
			},
			func(res *channeltypes.MsgChannelUpgradeAckResponse, events []abci.Event, err error) {
				suite.Require().NoError(err)

				suite.Require().NotNil(res)
				suite.Require().Equal(channeltypes.FAILURE, res.Result)

				errorReceipt, found := suite.chainA.GetSimApp().GetIBCKeeper().ChannelKeeper.GetUpgradeErrorReceipt(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found)
				suite.Require().Equal(uint64(1), errorReceipt.Sequence)

				channel := path.EndpointB.GetChannel()
				expEvents := ibctesting.EventsMap{
					channeltypes.EventTypeChannelUpgradeError: {
						channeltypes.AttributeKeyPortID:             path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointB.ChannelID,
						channeltypes.AttributeCounterpartyPortID:    path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointA.ChannelID,
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
						// need to manually insert this because the errorReceipt is a string constant as it is written into state
						channeltypes.AttributeKeyErrorReceipt: "expected upgrade ordering (ORDER_NONE_UNSPECIFIED) to match counterparty upgrade ordering (ORDER_UNORDERED): incompatible counterparty upgrade",
					},
					sdk.EventTypeMessage: {
						sdk.AttributeKeyModule: channeltypes.AttributeValueCategory,
					},
				}

				ibctesting.AssertEventsLegacy(&suite.Suite, expEvents, events)
			},
		},
		{
			"application callback returns error and error receipt is written",
			func() {
				suite.chainA.GetSimApp().IBCMockModule.IBCApp.OnChanUpgradeAck = func(
					ctx context.Context, portID, channelID, counterpartyVersion string,
				) error {
					// set arbitrary value in store to mock application state changes
					sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/5917
					store := sdkCtx.KVStore(suite.chainA.GetSimApp().GetKey(exported.ModuleName))
					store.Set([]byte("foo"), []byte("bar"))
					return fmt.Errorf("mock app callback failed")
				}
			},
			func(res *channeltypes.MsgChannelUpgradeAckResponse, events []abci.Event, err error) {
				suite.Require().NoError(err)

				suite.Require().NotNil(res)
				suite.Require().Equal(channeltypes.FAILURE, res.Result)

				errorReceipt, found := suite.chainA.GetSimApp().GetIBCKeeper().ChannelKeeper.GetUpgradeErrorReceipt(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found)
				suite.Require().Equal(uint64(1), errorReceipt.Sequence)

				// assert application state changes are not committed
				store := suite.chainA.GetContext().KVStore(suite.chainA.GetSimApp().GetKey(exported.ModuleName))
				suite.Require().False(store.Has([]byte("foo")))

				channel := path.EndpointB.GetChannel()
				expEvents := ibctesting.EventsMap{
					channeltypes.EventTypeChannelUpgradeError: {
						channeltypes.AttributeKeyPortID:             path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointB.ChannelID,
						channeltypes.AttributeCounterpartyPortID:    path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointA.ChannelID,
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
						// need to manually insert this because the errorReceipt is a string constant as it is written into state
						channeltypes.AttributeKeyErrorReceipt: "mock app callback failed",
					},
					sdk.EventTypeMessage: {
						sdk.AttributeKeyModule: channeltypes.AttributeValueCategory,
					},
				}

				ibctesting.AssertEventsLegacy(&suite.Suite, expEvents, events)
			},
		},
		{
			"ibc application does not implement the UpgradeableModule interface",
			func() {
				path = ibctesting.NewPath(suite.chainA, suite.chainB)
				path.EndpointA.ChannelConfig.PortID = ibcmock.MockBlockUpgrade
				path.EndpointB.ChannelConfig.PortID = ibcmock.MockBlockUpgrade

				suite.coordinator.Setup(path)

				msg.PortId = path.EndpointB.ChannelConfig.PortID
				msg.ChannelId = path.EndpointB.ChannelID
			},
			func(res *channeltypes.MsgChannelUpgradeAckResponse, events []abci.Event, err error) {
				suite.Require().ErrorIs(err, porttypes.ErrInvalidRoute)
				suite.Require().Nil(res)
				suite.Require().Empty(events)
			},
		},
		{
			"application callback returns an upgrade error",
			func() {
				suite.chainA.GetSimApp().IBCMockModule.IBCApp.OnChanUpgradeAck = func(ctx context.Context, portID, channelID, counterpartyVersion string) error {
					return channeltypes.NewUpgradeError(10000000, ibcmock.MockApplicationCallbackError)
				}
			},
			func(res *channeltypes.MsgChannelUpgradeAckResponse, events []abci.Event, err error) {
				suite.Require().Equal(channeltypes.FAILURE, res.Result)
				suite.Require().Equal(uint64(1), path.EndpointA.GetChannel().UpgradeSequence, "application callback upgrade sequence should not be used")
			},
		},
		{
			"ibc application does not commit state changes in callback",
			func() {
				suite.chainA.GetSimApp().IBCMockModule.IBCApp.OnChanUpgradeAck = func(ctx context.Context, portID, channelID, counterpartyVersion string) error {
					storeKey := suite.chainA.GetSimApp().GetKey(exported.ModuleName)
					sdkCtx := sdk.UnwrapSDKContext(ctx) // TODO: https://github.com/cosmos/ibc-go/issues/5917
					store := sdkCtx.KVStore(storeKey)
					store.Set(ibcmock.TestKey, ibcmock.TestValue)

					sdkCtx.EventManager().EmitEvent(sdk.NewEvent(ibcmock.MockEventType))
					return nil
				}
			},
			func(res *channeltypes.MsgChannelUpgradeAckResponse, events []abci.Event, err error) {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)

				storeKey := suite.chainA.GetSimApp().GetKey(exported.ModuleName)
				store := suite.chainA.GetContext().KVStore(storeKey)
				suite.Require().Nil(store.Get(ibcmock.TestKey))

				for _, event := range events {
					if event.GetType() == ibcmock.MockEventType {
						suite.Fail("expected application callback events to be discarded")
					}
				}
			},
		},
	}

	for _, tc := range cases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			// configure the channel upgrade version on testing endpoints
			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = ibcmock.UpgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = ibcmock.UpgradeVersion

			err := path.EndpointA.ChanUpgradeInit()
			suite.Require().NoError(err)

			err = path.EndpointB.ChanUpgradeTry()
			suite.Require().NoError(err)

			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			counterpartyUpgrade := path.EndpointB.GetChannelUpgrade()

			channelProof, upgradeProof, proofHeight := path.EndpointB.QueryChannelUpgradeProof()

			msg = &channeltypes.MsgChannelUpgradeAck{
				PortId:              path.EndpointA.ChannelConfig.PortID,
				ChannelId:           path.EndpointA.ChannelID,
				CounterpartyUpgrade: counterpartyUpgrade,
				ProofChannel:        channelProof,
				ProofUpgrade:        upgradeProof,
				ProofHeight:         proofHeight,
				Signer:              suite.chainA.SenderAccount.GetAddress().String(),
			}

			tc.malleate()

			ctx := suite.chainA.GetContext()
			res, err := suite.chainA.GetSimApp().GetIBCKeeper().ChannelUpgradeAck(ctx, msg)
			events := ctx.EventManager().Events().ToABCIEvents()

			tc.expResult(res, events, err)
		})
	}
}

func (suite *KeeperTestSuite) TestChannelUpgradeConfirm() {
	var (
		path *ibctesting.Path
		msg  *channeltypes.MsgChannelUpgradeConfirm
	)

	cases := []struct {
		name      string
		malleate  func()
		expResult func(res *channeltypes.MsgChannelUpgradeConfirmResponse, events []abci.Event, err error)
	}{
		{
			"success, no pending in-flight packets",
			func() {},
			func(res *channeltypes.MsgChannelUpgradeConfirmResponse, events []abci.Event, err error) {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)
				suite.Require().Equal(channeltypes.SUCCESS, res.Result)

				channel := path.EndpointB.GetChannel()
				suite.Require().Equal(channeltypes.OPEN, channel.State)
				suite.Require().Equal(uint64(1), channel.UpgradeSequence)

				expEvents := ibctesting.EventsMap{
					channeltypes.EventTypeChannelUpgradeConfirm: {
						channeltypes.AttributeKeyPortID:             path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointB.ChannelID,
						channeltypes.AttributeKeyChannelState:       channeltypes.FLUSHCOMPLETE.String(),
						channeltypes.AttributeCounterpartyPortID:    path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointA.ChannelID,
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
					},
					channeltypes.EventTypeChannelUpgradeOpen: {
						channeltypes.AttributeKeyPortID:             path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointB.ChannelID,
						channeltypes.AttributeCounterpartyPortID:    path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointA.ChannelID,
						channeltypes.AttributeKeyChannelState:       channeltypes.OPEN.String(),
						channeltypes.AttributeKeyConnectionHops:     channel.ConnectionHops[0],
						channeltypes.AttributeVersion:               channel.Version,
						channeltypes.AttributeKeyOrdering:           channel.Ordering.String(),
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
					},
					sdk.EventTypeMessage: {
						sdk.AttributeKeyModule: channeltypes.AttributeValueCategory,
					},
				}

				ibctesting.AssertEventsLegacy(&suite.Suite, expEvents, events)
			},
		},
		{
			"success, pending in-flight packets on init chain",
			func() {
				path = ibctesting.NewPath(suite.chainA, suite.chainB)
				suite.coordinator.Setup(path)

				path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = ibcmock.UpgradeVersion
				path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = ibcmock.UpgradeVersion

				err := path.EndpointA.ChanUpgradeInit()
				suite.Require().NoError(err)

				err = path.EndpointB.ChanUpgradeTry()
				suite.Require().NoError(err)

				seq, err := path.EndpointA.SendPacket(path.EndpointB.Chain.GetTimeoutHeight(), 0, ibctesting.MockPacketData)
				suite.Require().Equal(uint64(1), seq)
				suite.Require().NoError(err)

				err = path.EndpointA.ChanUpgradeAck()
				suite.Require().NoError(err)

				err = path.EndpointB.UpdateClient()
				suite.Require().NoError(err)

				counterpartyChannelState := path.EndpointA.GetChannel().State
				counterpartyUpgrade := path.EndpointA.GetChannelUpgrade()

				channelProof, upgradeProof, proofHeight := path.EndpointA.QueryChannelUpgradeProof()

				msg = &channeltypes.MsgChannelUpgradeConfirm{
					PortId:                   path.EndpointB.ChannelConfig.PortID,
					ChannelId:                path.EndpointB.ChannelID,
					CounterpartyChannelState: counterpartyChannelState,
					CounterpartyUpgrade:      counterpartyUpgrade,
					ProofChannel:             channelProof,
					ProofUpgrade:             upgradeProof,
					ProofHeight:              proofHeight,
					Signer:                   suite.chainA.SenderAccount.GetAddress().String(),
				}
			},
			func(res *channeltypes.MsgChannelUpgradeConfirmResponse, events []abci.Event, err error) {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)
				suite.Require().Equal(channeltypes.SUCCESS, res.Result)

				channel := path.EndpointA.GetChannel()
				suite.Require().Equal(channeltypes.FLUSHING, channel.State)
				suite.Require().Equal(uint64(1), channel.UpgradeSequence)

				channel = path.EndpointB.GetChannel()
				suite.Require().Equal(channeltypes.FLUSHCOMPLETE, channel.State)
				suite.Require().Equal(uint64(1), channel.UpgradeSequence)

				expEvents := ibctesting.EventsMap{
					channeltypes.EventTypeChannelUpgradeConfirm: {
						channeltypes.AttributeKeyPortID:             path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointB.ChannelID,
						channeltypes.AttributeKeyChannelState:       channeltypes.FLUSHCOMPLETE.String(),
						channeltypes.AttributeCounterpartyPortID:    path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointA.ChannelID,
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
					},
				}

				ibctesting.AssertEventsLegacy(&suite.Suite, expEvents, events)
			},
		},
		{
			"success, pending in-flight packets on try chain",
			func() {
				portID, channelID := path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID
				suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.SetPacketCommitment(suite.chainB.GetContext(), portID, channelID, 1, []byte("hash"))
			},
			func(res *channeltypes.MsgChannelUpgradeConfirmResponse, events []abci.Event, err error) {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)
				suite.Require().Equal(channeltypes.SUCCESS, res.Result)

				channel := path.EndpointB.GetChannel()
				suite.Require().Equal(channeltypes.FLUSHING, channel.State)
				suite.Require().Equal(uint64(1), channel.UpgradeSequence)

				expEvents := ibctesting.EventsMap{
					channeltypes.EventTypeChannelUpgradeConfirm: {
						channeltypes.AttributeKeyPortID:             path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointB.ChannelID,
						channeltypes.AttributeKeyChannelState:       channeltypes.FLUSHING.String(),
						channeltypes.AttributeCounterpartyPortID:    path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointA.ChannelID,
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
					},
				}

				ibctesting.AssertEventsLegacy(&suite.Suite, expEvents, events)
			},
		},
		{
			"module capability not found",
			func() {
				msg.PortId = ibctesting.InvalidID
				msg.ChannelId = ibctesting.InvalidID
			},
			func(res *channeltypes.MsgChannelUpgradeConfirmResponse, events []abci.Event, err error) {
				suite.Require().Error(err)
				suite.Require().Nil(res)

				suite.Require().ErrorIs(err, capabilitytypes.ErrCapabilityNotFound)
				suite.Require().Empty(events)
			},
		},
		{
			"core handler returns error and no upgrade error receipt is written",
			func() {
				// force an error by overriding the channel state to an invalid value
				channel := path.EndpointB.GetChannel()
				channel.State = channeltypes.CLOSED

				path.EndpointB.SetChannel(channel)
			},
			func(res *channeltypes.MsgChannelUpgradeConfirmResponse, events []abci.Event, err error) {
				suite.Require().Error(err)
				suite.Require().Nil(res)
				suite.Require().ErrorIs(err, channeltypes.ErrInvalidChannelState)

				errorReceipt, found := suite.chainB.GetSimApp().GetIBCKeeper().ChannelKeeper.GetUpgradeErrorReceipt(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
				suite.Require().Empty(errorReceipt)
				suite.Require().False(found)

				suite.Require().Empty(events)
			},
		},
		{
			"core handler returns error and writes upgrade error receipt",
			func() {
				// force an upgrade error by modifying the counterparty channel upgrade timeout to be elapsed
				upgrade := path.EndpointA.GetChannelUpgrade()
				upgrade.Timeout = channeltypes.NewTimeout(clienttypes.ZeroHeight(), uint64(path.EndpointB.Chain.CurrentHeader.Time.UnixNano()))

				path.EndpointA.SetChannelUpgrade(upgrade)

				suite.coordinator.CommitBlock(suite.chainA)

				err := path.EndpointB.UpdateClient()
				suite.Require().NoError(err)

				channelProof, upgradeProof, proofHeight := path.EndpointA.QueryChannelUpgradeProof()

				msg.CounterpartyUpgrade = upgrade
				msg.ProofChannel = channelProof
				msg.ProofUpgrade = upgradeProof
				msg.ProofHeight = proofHeight
			},
			func(res *channeltypes.MsgChannelUpgradeConfirmResponse, events []abci.Event, err error) {
				suite.Require().NoError(err)

				suite.Require().NotNil(res)
				suite.Require().Equal(channeltypes.FAILURE, res.Result)

				errorReceipt, found := suite.chainB.GetSimApp().GetIBCKeeper().ChannelKeeper.GetUpgradeErrorReceipt(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
				suite.Require().True(found)
				suite.Require().Equal(uint64(1), errorReceipt.Sequence)

				channel := path.EndpointB.GetChannel()

				expEvents := ibctesting.EventsMap{
					channeltypes.EventTypeChannelUpgradeError: {
						channeltypes.AttributeKeyPortID:             path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointB.ChannelID,
						channeltypes.AttributeCounterpartyPortID:    path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointA.ChannelID,
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
						// need to manually insert this because the errorReceipt is a string constant as it is written into state
						channeltypes.AttributeKeyErrorReceipt: "counterparty upgrade timeout elapsed: current timestamp: 1578269010000000000, timeout timestamp 1578268995000000000: timeout elapsed",
					},
					sdk.EventTypeMessage: {
						sdk.AttributeKeyModule: channeltypes.AttributeValueCategory,
					},
				}

				ibctesting.AssertEventsLegacy(&suite.Suite, expEvents, events)
			},
		},
		{
			"ibc application does not implement the UpgradeableModule interface",
			func() {
				path = ibctesting.NewPath(suite.chainA, suite.chainB)
				path.EndpointA.ChannelConfig.PortID = ibcmock.MockBlockUpgrade
				path.EndpointB.ChannelConfig.PortID = ibcmock.MockBlockUpgrade

				suite.coordinator.Setup(path)

				msg.PortId = path.EndpointB.ChannelConfig.PortID
				msg.ChannelId = path.EndpointB.ChannelID
			},
			func(res *channeltypes.MsgChannelUpgradeConfirmResponse, events []abci.Event, err error) {
				suite.Require().ErrorIs(err, porttypes.ErrInvalidRoute)
				suite.Require().Nil(res)

				suite.Require().Empty(events)
			},
		},
	}

	for _, tc := range cases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			// configure the channel upgrade version on testing endpoints
			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = ibcmock.UpgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = ibcmock.UpgradeVersion

			err := path.EndpointA.ChanUpgradeInit()
			suite.Require().NoError(err)

			err = path.EndpointB.ChanUpgradeTry()
			suite.Require().NoError(err)

			err = path.EndpointA.ChanUpgradeAck()
			suite.Require().NoError(err)

			err = path.EndpointB.UpdateClient()
			suite.Require().NoError(err)

			counterpartyChannelState := path.EndpointA.GetChannel().State
			counterpartyUpgrade := path.EndpointA.GetChannelUpgrade()

			channelProof, upgradeProof, proofHeight := path.EndpointA.QueryChannelUpgradeProof()

			msg = &channeltypes.MsgChannelUpgradeConfirm{
				PortId:                   path.EndpointB.ChannelConfig.PortID,
				ChannelId:                path.EndpointB.ChannelID,
				CounterpartyChannelState: counterpartyChannelState,
				CounterpartyUpgrade:      counterpartyUpgrade,
				ProofChannel:             channelProof,
				ProofUpgrade:             upgradeProof,
				ProofHeight:              proofHeight,
				Signer:                   suite.chainA.SenderAccount.GetAddress().String(),
			}

			tc.malleate()

			ctx := suite.chainB.GetContext()
			res, err := suite.chainB.GetSimApp().GetIBCKeeper().ChannelUpgradeConfirm(ctx, msg)
			events := ctx.EventManager().Events().ToABCIEvents()

			tc.expResult(res, events, err)
		})
	}
}

func (suite *KeeperTestSuite) TestChannelUpgradeOpen() {
	var (
		path *ibctesting.Path
		msg  *channeltypes.MsgChannelUpgradeOpen
	)

	cases := []struct {
		name      string
		malleate  func()
		expResult func(res *channeltypes.MsgChannelUpgradeOpenResponse, events []abci.Event, err error)
	}{
		{
			"success",
			func() {},
			func(res *channeltypes.MsgChannelUpgradeOpenResponse, events []abci.Event, err error) {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)

				channel := path.EndpointA.GetChannel()
				suite.Require().Equal(channeltypes.OPEN, channel.State)

				expEvents := ibctesting.EventsMap{
					channeltypes.EventTypeChannelUpgradeOpen: {
						channeltypes.AttributeKeyPortID:             path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointA.ChannelID,
						channeltypes.AttributeCounterpartyPortID:    path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointB.ChannelID,
						channeltypes.AttributeKeyChannelState:       channeltypes.OPEN.String(),
						channeltypes.AttributeKeyConnectionHops:     channel.ConnectionHops[0],
						channeltypes.AttributeVersion:               channel.Version,
						channeltypes.AttributeKeyOrdering:           channel.Ordering.String(),
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
					},
					sdk.EventTypeMessage: {
						sdk.AttributeKeyModule: channeltypes.AttributeValueCategory,
					},
				}
				ibctesting.AssertEventsLegacy(&suite.Suite, expEvents, events)
			},
		},
		{
			"success with counterparty at greater upgrade sequence",
			func() {
				// create reason to upgrade
				path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = ibcmock.UpgradeVersion + "additional upgrade"

				err := path.EndpointB.ChanUpgradeInit()
				suite.Require().NoError(err)

				err = path.EndpointA.UpdateClient()
				suite.Require().NoError(err)

				counterpartyChannel := path.EndpointB.GetChannel()
				channelKey := host.ChannelKey(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
				channelProof, proofHeight := path.EndpointB.QueryProof(channelKey)

				msg.ProofChannel = channelProof
				msg.ProofHeight = proofHeight
				msg.CounterpartyUpgradeSequence = counterpartyChannel.UpgradeSequence
			},
			func(res *channeltypes.MsgChannelUpgradeOpenResponse, events []abci.Event, err error) {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)

				channel := path.EndpointA.GetChannel()
				suite.Require().Equal(channeltypes.OPEN, channel.State)

				expEvents := ibctesting.EventsMap{
					channeltypes.EventTypeChannelUpgradeOpen: {
						channeltypes.AttributeKeyPortID:             path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointA.ChannelID,
						channeltypes.AttributeCounterpartyPortID:    path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointB.ChannelID,
						channeltypes.AttributeKeyChannelState:       channeltypes.OPEN.String(),
						channeltypes.AttributeKeyConnectionHops:     channel.ConnectionHops[0],
						channeltypes.AttributeVersion:               channel.Version,
						channeltypes.AttributeKeyOrdering:           channel.Ordering.String(),
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
					},
					sdk.EventTypeMessage: {
						sdk.AttributeKeyModule: channeltypes.AttributeValueCategory,
					},
				}
				ibctesting.AssertEventsLegacy(&suite.Suite, expEvents, events)
			},
		},
		{
			"module capability not found",
			func() {
				msg.PortId = ibctesting.InvalidID
				msg.ChannelId = ibctesting.InvalidID
			},
			func(res *channeltypes.MsgChannelUpgradeOpenResponse, events []abci.Event, err error) {
				suite.Require().Error(err)
				suite.Require().Nil(res)

				suite.Require().ErrorIs(err, capabilitytypes.ErrCapabilityNotFound)
				suite.Require().Empty(events)
			},
		},
		{
			"core handler fails",
			func() {
				channel := path.EndpointA.GetChannel()
				channel.State = channeltypes.FLUSHING
				path.EndpointA.SetChannel(channel)
			},
			func(res *channeltypes.MsgChannelUpgradeOpenResponse, events []abci.Event, err error) {
				suite.Require().Error(err)
				suite.Require().Nil(res)

				suite.Require().ErrorIs(err, channeltypes.ErrInvalidChannelState)
				suite.Require().Empty(events)
			},
		},
		{
			"ibc application does not implement the UpgradeableModule interface",
			func() {
				path = ibctesting.NewPath(suite.chainA, suite.chainB)
				path.EndpointA.ChannelConfig.PortID = ibcmock.MockBlockUpgrade
				path.EndpointB.ChannelConfig.PortID = ibcmock.MockBlockUpgrade

				suite.coordinator.Setup(path)

				msg.PortId = path.EndpointB.ChannelConfig.PortID
				msg.ChannelId = path.EndpointB.ChannelID
			},
			func(res *channeltypes.MsgChannelUpgradeOpenResponse, events []abci.Event, err error) {
				suite.Require().ErrorIs(err, porttypes.ErrInvalidRoute)
				suite.Require().Nil(res)

				suite.Require().Empty(events)
			},
		},
	}

	for _, tc := range cases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			// configure the channel upgrade version on testing endpoints
			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = ibcmock.UpgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = ibcmock.UpgradeVersion

			err := path.EndpointA.ChanUpgradeInit()
			suite.Require().NoError(err)

			err = path.EndpointB.ChanUpgradeTry()
			suite.Require().NoError(err)

			err = path.EndpointA.ChanUpgradeAck()
			suite.Require().NoError(err)

			err = path.EndpointB.ChanUpgradeConfirm()
			suite.Require().NoError(err)

			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			counterpartyChannel := path.EndpointB.GetChannel()
			channelKey := host.ChannelKey(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
			channelProof, proofHeight := path.EndpointB.QueryProof(channelKey)

			msg = &channeltypes.MsgChannelUpgradeOpen{
				PortId:                      path.EndpointA.ChannelConfig.PortID,
				ChannelId:                   path.EndpointA.ChannelID,
				CounterpartyChannelState:    counterpartyChannel.State,
				CounterpartyUpgradeSequence: counterpartyChannel.UpgradeSequence,
				ProofChannel:                channelProof,
				ProofHeight:                 proofHeight,
				Signer:                      suite.chainA.SenderAccount.GetAddress().String(),
			}

			tc.malleate()

			ctx := suite.chainA.GetContext()
			res, err := suite.chainA.GetSimApp().GetIBCKeeper().ChannelUpgradeOpen(ctx, msg)
			events := ctx.EventManager().Events().ToABCIEvents()

			tc.expResult(res, events, err)
		})
	}
}

func (suite *KeeperTestSuite) TestChannelUpgradeCancel() {
	var (
		path *ibctesting.Path
		msg  *channeltypes.MsgChannelUpgradeCancel
	)

	cases := []struct {
		name      string
		malleate  func()
		expResult func(res *channeltypes.MsgChannelUpgradeCancelResponse, events []abci.Event, err error)
	}{
		{
			"success: keeper is not authority, valid error receipt so channnel changed to match error receipt seq",
			func() {},
			func(res *channeltypes.MsgChannelUpgradeCancelResponse, events []abci.Event, err error) {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)

				channel := path.EndpointA.GetChannel()
				// Channel state should be reverted back to open.
				suite.Require().Equal(channeltypes.OPEN, channel.State)
				// Upgrade sequence should be changed to match sequence on error receipt.
				suite.Require().Equal(uint64(2), channel.UpgradeSequence)

				// we need to find the event values from the proposed upgrade as the actual upgrade has been deleted.
				proposedUpgrade := path.EndpointA.GetProposedUpgrade()
				expEvents := ibctesting.EventsMap{
					channeltypes.EventTypeChannelUpgradeCancel: {
						channeltypes.AttributeKeyPortID:             path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointA.ChannelID,
						channeltypes.AttributeCounterpartyPortID:    path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointB.ChannelID,
						channeltypes.AttributeKeyConnectionHops:     proposedUpgrade.Fields.ConnectionHops[0],
						channeltypes.AttributeVersion:               proposedUpgrade.Fields.Version,
						channeltypes.AttributeKeyOrdering:           proposedUpgrade.Fields.Ordering.String(),
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
					},
					channeltypes.EventTypeChannelUpgradeError: {
						channeltypes.AttributeKeyPortID:             path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointA.ChannelID,
						channeltypes.AttributeCounterpartyPortID:    path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointB.ChannelID,
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
						// need to manually insert this because the errorReceipt is a string constant as it is written into state
						channeltypes.AttributeKeyErrorReceipt: "invalid upgrade",
					},
					sdk.EventTypeMessage: {
						sdk.AttributeKeyModule: channeltypes.AttributeValueCategory,
					},
				}
				ibctesting.AssertEventsLegacy(&suite.Suite, expEvents, events)
			},
		},
		{
			"success: keeper is authority & channel state in FLUSHING, so error receipt is ignored and channel is restored to initial upgrade sequence",
			func() {
				msg.Signer = suite.chainA.App.GetIBCKeeper().GetAuthority()

				channel := path.EndpointA.GetChannel()
				channel.State = channeltypes.FLUSHING
				channel.UpgradeSequence = uint64(3)
				path.EndpointA.SetChannel(channel)
			},
			func(res *channeltypes.MsgChannelUpgradeCancelResponse, events []abci.Event, err error) {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)

				channel := path.EndpointA.GetChannel()
				// Channel state should be reverted back to open.
				suite.Require().Equal(channeltypes.OPEN, channel.State)
				// Upgrade sequence should be changed to match initial upgrade sequence.
				suite.Require().Equal(uint64(3), channel.UpgradeSequence)

				// we need to find the event values from the proposed upgrade as the actual upgrade has been deleted.
				proposedUpgrade := path.EndpointA.GetProposedUpgrade()
				expEvents := ibctesting.EventsMap{
					channeltypes.EventTypeChannelUpgradeCancel: {
						channeltypes.AttributeKeyPortID:             path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointA.ChannelID,
						channeltypes.AttributeCounterpartyPortID:    path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointB.ChannelID,
						channeltypes.AttributeKeyConnectionHops:     proposedUpgrade.Fields.ConnectionHops[0],
						channeltypes.AttributeVersion:               proposedUpgrade.Fields.Version,
						channeltypes.AttributeKeyOrdering:           proposedUpgrade.Fields.Ordering.String(),
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
					},
					channeltypes.EventTypeChannelUpgradeError: {
						channeltypes.AttributeKeyPortID:             path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointA.ChannelID,
						channeltypes.AttributeCounterpartyPortID:    path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointB.ChannelID,
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
						// need to manually insert this because the errorReceipt is a string constant as it is written into state
						channeltypes.AttributeKeyErrorReceipt: "invalid upgrade",
					},
					sdk.EventTypeMessage: {
						sdk.AttributeKeyModule: channeltypes.AttributeValueCategory,
					},
				}
				ibctesting.AssertEventsLegacy(&suite.Suite, expEvents, events)
			},
		},
		{
			"success: keeper is authority & channel state in FLUSHING, can be cancelled even with invalid error receipt",
			func() {
				msg.Signer = suite.chainA.App.GetIBCKeeper().GetAuthority()
				msg.ProofErrorReceipt = []byte("invalid proof")

				channel := path.EndpointA.GetChannel()
				channel.State = channeltypes.FLUSHING
				channel.UpgradeSequence = uint64(1)
				path.EndpointA.SetChannel(channel)
			},
			func(res *channeltypes.MsgChannelUpgradeCancelResponse, events []abci.Event, err error) {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)

				channel := path.EndpointA.GetChannel()
				// Channel state should be reverted back to open.
				suite.Require().Equal(channeltypes.OPEN, channel.State)
				// Upgrade sequence should be changed to match initial upgrade sequence.
				suite.Require().Equal(uint64(1), channel.UpgradeSequence)

				// we need to find the event values from the proposed upgrade as the actual upgrade has been deleted.
				proposedUpgrade := path.EndpointA.GetProposedUpgrade()
				expEvents := ibctesting.EventsMap{
					channeltypes.EventTypeChannelUpgradeCancel: {
						channeltypes.AttributeKeyPortID:             path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointA.ChannelID,
						channeltypes.AttributeCounterpartyPortID:    path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointB.ChannelID,
						channeltypes.AttributeKeyConnectionHops:     proposedUpgrade.Fields.ConnectionHops[0],
						channeltypes.AttributeVersion:               proposedUpgrade.Fields.Version,
						channeltypes.AttributeKeyOrdering:           proposedUpgrade.Fields.Ordering.String(),
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
					},
					channeltypes.EventTypeChannelUpgradeError: {
						channeltypes.AttributeKeyPortID:             path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointA.ChannelID,
						channeltypes.AttributeCounterpartyPortID:    path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointB.ChannelID,
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
						// need to manually insert this because the errorReceipt is a string constant as it is written into state
						channeltypes.AttributeKeyErrorReceipt: "invalid upgrade",
					},
					sdk.EventTypeMessage: {
						sdk.AttributeKeyModule: channeltypes.AttributeValueCategory,
					},
				}
				ibctesting.AssertEventsLegacy(&suite.Suite, expEvents, events)
			},
		},
		{
			"success: keeper is authority & channel state in FLUSHING, can be cancelled even with empty error receipt",
			func() {
				msg.Signer = suite.chainA.App.GetIBCKeeper().GetAuthority()
				msg.ProofErrorReceipt = nil

				channel := path.EndpointA.GetChannel()
				channel.State = channeltypes.FLUSHING
				channel.UpgradeSequence = uint64(1)
				path.EndpointA.SetChannel(channel)
			},
			func(res *channeltypes.MsgChannelUpgradeCancelResponse, events []abci.Event, err error) {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)

				channel := path.EndpointA.GetChannel()
				// Channel state should be reverted back to open.
				suite.Require().Equal(channeltypes.OPEN, channel.State)
				// Upgrade sequence should be changed to match initial upgrade sequence.
				suite.Require().Equal(uint64(1), channel.UpgradeSequence)

				// we need to find the event values from the proposed upgrade as the actual upgrade has been deleted.
				proposedUpgrade := path.EndpointA.GetProposedUpgrade()
				expEvents := ibctesting.EventsMap{
					channeltypes.EventTypeChannelUpgradeCancel: {
						channeltypes.AttributeKeyPortID:             path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointA.ChannelID,
						channeltypes.AttributeCounterpartyPortID:    path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointB.ChannelID,
						channeltypes.AttributeKeyConnectionHops:     proposedUpgrade.Fields.ConnectionHops[0],
						channeltypes.AttributeVersion:               proposedUpgrade.Fields.Version,
						channeltypes.AttributeKeyOrdering:           proposedUpgrade.Fields.Ordering.String(),
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
					},
					channeltypes.EventTypeChannelUpgradeError: {
						channeltypes.AttributeKeyPortID:             path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointA.ChannelID,
						channeltypes.AttributeCounterpartyPortID:    path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointB.ChannelID,
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
						// need to manually insert this because the errorReceipt is a string constant as it is written into state
						channeltypes.AttributeKeyErrorReceipt: "invalid upgrade",
					},
					sdk.EventTypeMessage: {
						sdk.AttributeKeyModule: channeltypes.AttributeValueCategory,
					},
				}
				ibctesting.AssertEventsLegacy(&suite.Suite, expEvents, events)
			},
		},
		{
			"success: keeper is authority but channel state in FLUSHCOMPLETE, requires valid error receipt",
			func() {
				msg.Signer = suite.chainA.App.GetIBCKeeper().GetAuthority()

				channel := path.EndpointA.GetChannel()
				channel.State = channeltypes.FLUSHCOMPLETE
				channel.UpgradeSequence = uint64(2) // When in FLUSHCOMPLETE the sequence of the error receipt and the channel must match
				path.EndpointA.SetChannel(channel)
			},
			func(res *channeltypes.MsgChannelUpgradeCancelResponse, events []abci.Event, err error) {
				suite.Require().NoError(err)
				suite.Require().NotNil(res)

				channel := path.EndpointA.GetChannel()
				// Channel state should be reverted back to open.
				suite.Require().Equal(channeltypes.OPEN, channel.State)
				// Upgrade sequence should not be changed.
				suite.Require().Equal(uint64(2), channel.UpgradeSequence)

				// we need to find the event values from the proposed upgrade as the actual upgrade has been deleted.
				proposedUpgrade := path.EndpointA.GetProposedUpgrade()
				expEvents := ibctesting.EventsMap{
					channeltypes.EventTypeChannelUpgradeCancel: {
						channeltypes.AttributeKeyPortID:             path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointA.ChannelID,
						channeltypes.AttributeCounterpartyPortID:    path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointB.ChannelID,
						channeltypes.AttributeKeyConnectionHops:     proposedUpgrade.Fields.ConnectionHops[0],
						channeltypes.AttributeVersion:               proposedUpgrade.Fields.Version,
						channeltypes.AttributeKeyOrdering:           proposedUpgrade.Fields.Ordering.String(),
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
					},
					channeltypes.EventTypeChannelUpgradeError: {
						channeltypes.AttributeKeyPortID:             path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointA.ChannelID,
						channeltypes.AttributeCounterpartyPortID:    path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointB.ChannelID,
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
						// need to manually insert this because the errorReceipt is a string constant as it is written into state
						channeltypes.AttributeKeyErrorReceipt: "invalid upgrade",
					},
					sdk.EventTypeMessage: {
						sdk.AttributeKeyModule: channeltypes.AttributeValueCategory,
					},
				}
				ibctesting.AssertEventsLegacy(&suite.Suite, expEvents, events)
			},
		},
		{
			"failure: keeper is authority and channel state in FLUSHCOMPLETE, but error receipt and channel upgrade sequences do not match",
			func() {
				msg.Signer = suite.chainA.App.GetIBCKeeper().GetAuthority()

				suite.Require().NoError(path.EndpointA.SetChannelState(channeltypes.FLUSHCOMPLETE))
			},
			func(res *channeltypes.MsgChannelUpgradeCancelResponse, events []abci.Event, err error) {
				suite.Require().Error(err)
				suite.Require().Nil(res)
				suite.Require().ErrorIs(err, channeltypes.ErrInvalidUpgradeSequence)

				channel := path.EndpointA.GetChannel()
				// Channel state should not be reverted back to open.
				suite.Require().Equal(channeltypes.FLUSHCOMPLETE, channel.State)
				// Upgrade sequence should not be changed.
				suite.Require().Equal(uint64(1), channel.UpgradeSequence)
			},
		},
		{
			"core handler fails: invalid proof",
			func() {
				msg.ProofErrorReceipt = []byte("invalid proof")

				// Force set to STATE_FLUSHCOMPLETE to check that state is not changed.
				channel := path.EndpointA.GetChannel()
				channel.State = channeltypes.FLUSHCOMPLETE
				channel.UpgradeSequence = uint64(2) // When in FLUSHCOMPLETE the sequence of the error receipt and the channel must match
				path.EndpointA.SetChannel(channel)
			},
			func(res *channeltypes.MsgChannelUpgradeCancelResponse, events []abci.Event, err error) {
				suite.Require().Error(err)
				suite.Require().Nil(res)

				channel := path.EndpointA.GetChannel()
				suite.Require().ErrorIs(err, commitmenttypes.ErrInvalidProof)
				// Channel state should not be changed.
				suite.Require().Equal(channeltypes.FLUSHCOMPLETE, channel.State)
				// Upgrade sequence should not be changed.
				suite.Require().Equal(uint64(2), channel.UpgradeSequence)

				suite.Require().Empty(events)
			},
		},
	}

	for _, tc := range cases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			// configure the channel upgrade version on testing endpoints
			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = ibcmock.UpgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = ibcmock.UpgradeVersion

			err := path.EndpointA.ChanUpgradeInit()
			suite.Require().NoError(err)

			// cause the upgrade to fail on chain b so an error receipt is written.
			// if the counterparty (chain A) upgrade sequence is less than the current sequence, (chain B)
			// an upgrade error will be returned by chain B during ChanUpgradeTry.
			channel := path.EndpointA.GetChannel()
			channel.UpgradeSequence = 1
			path.EndpointA.SetChannel(channel)

			channel = path.EndpointB.GetChannel()
			channel.UpgradeSequence = 2
			path.EndpointB.SetChannel(channel)

			suite.Require().NoError(path.EndpointA.UpdateClient())
			suite.Require().NoError(path.EndpointB.UpdateClient())

			suite.Require().NoError(path.EndpointB.ChanUpgradeTry())

			suite.Require().NoError(path.EndpointA.UpdateClient())

			upgradeErrorReceiptKey := host.ChannelUpgradeErrorKey(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
			errorReceiptProof, proofHeight := path.EndpointB.QueryProof(upgradeErrorReceiptKey)

			errorReceipt, ok := suite.chainB.GetSimApp().IBCKeeper.ChannelKeeper.GetUpgradeErrorReceipt(suite.chainB.GetContext(), path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID)
			suite.Require().True(ok)

			msg = &channeltypes.MsgChannelUpgradeCancel{
				PortId:            path.EndpointA.ChannelConfig.PortID,
				ChannelId:         path.EndpointA.ChannelID,
				ErrorReceipt:      errorReceipt,
				ProofErrorReceipt: errorReceiptProof,
				ProofHeight:       proofHeight,
				Signer:            suite.chainA.SenderAccount.GetAddress().String(),
			}

			tc.malleate()

			ctx := suite.chainA.GetContext()
			res, err := suite.chainA.GetSimApp().GetIBCKeeper().ChannelUpgradeCancel(ctx, msg)
			events := ctx.EventManager().Events().ToABCIEvents()

			tc.expResult(res, events, err)
		})
	}
}

func (suite *KeeperTestSuite) TestChannelUpgradeTimeout() {
	var (
		path *ibctesting.Path
		msg  *channeltypes.MsgChannelUpgradeTimeout
	)

	timeoutUpgrade := func() {
		upgrade := path.EndpointA.GetProposedUpgrade()
		upgrade.Timeout = channeltypes.NewTimeout(clienttypes.ZeroHeight(), 1)
		path.EndpointA.SetChannelUpgrade(upgrade)
		suite.Require().NoError(path.EndpointB.UpdateClient())
	}

	cases := []struct {
		name      string
		malleate  func()
		expResult func(res *channeltypes.MsgChannelUpgradeTimeoutResponse, events []abci.Event, err error)
	}{
		{
			"success",
			func() {
				timeoutUpgrade()

				suite.Require().NoError(path.EndpointA.UpdateClient())

				channelKey := host.ChannelKey(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				channelProof, proofHeight := path.EndpointB.QueryProof(channelKey)

				msg.ProofChannel = channelProof
				msg.ProofHeight = proofHeight
			},
			func(res *channeltypes.MsgChannelUpgradeTimeoutResponse, events []abci.Event, err error) {
				channel := path.EndpointA.GetChannel()

				suite.Require().Equalf(channeltypes.OPEN, channel.State, "channel state should be %s", channeltypes.OPEN.String())

				_, found := path.EndpointA.Chain.GetSimApp().IBCKeeper.ChannelKeeper.GetUpgrade(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().False(found, "channel upgrade should be nil")

				suite.Require().NotNil(res)
				suite.Require().NoError(err)

				errorReceipt, found := suite.chainA.GetSimApp().GetIBCKeeper().ChannelKeeper.GetUpgradeErrorReceipt(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found)
				suite.Require().Equal(uint64(1), errorReceipt.Sequence)

				// we need to find the event values from the proposed upgrade as the actual upgrade has been deleted.
				proposedUpgrade := path.EndpointA.GetProposedUpgrade()
				// use the timeout we set in the malleate function
				timeout := channeltypes.NewTimeout(clienttypes.ZeroHeight(), 1)
				expEvents := ibctesting.EventsMap{
					channeltypes.EventTypeChannelUpgradeTimeout: {
						channeltypes.AttributeKeyPortID:                  path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:               path.EndpointA.ChannelID,
						channeltypes.AttributeCounterpartyPortID:         path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID:      path.EndpointB.ChannelID,
						channeltypes.AttributeKeyConnectionHops:          proposedUpgrade.Fields.ConnectionHops[0],
						channeltypes.AttributeVersion:                    proposedUpgrade.Fields.Version,
						channeltypes.AttributeKeyOrdering:                proposedUpgrade.Fields.Ordering.String(),
						channeltypes.AttributeKeyUpgradeTimeoutHeight:    timeout.Height.String(),
						channeltypes.AttributeKeyUpgradeTimeoutTimestamp: fmt.Sprintf("%d", timeout.Timestamp),
						channeltypes.AttributeKeyUpgradeSequence:         fmt.Sprintf("%d", channel.UpgradeSequence),
					},
					channeltypes.EventTypeChannelUpgradeError: {
						channeltypes.AttributeKeyPortID:             path.EndpointA.ChannelConfig.PortID,
						channeltypes.AttributeKeyChannelID:          path.EndpointA.ChannelID,
						channeltypes.AttributeCounterpartyPortID:    path.EndpointB.ChannelConfig.PortID,
						channeltypes.AttributeCounterpartyChannelID: path.EndpointB.ChannelID,
						channeltypes.AttributeKeyUpgradeSequence:    fmt.Sprintf("%d", channel.UpgradeSequence),
						// need to manually insert this because the errorReceipt is a string constant as it is written into state
						channeltypes.AttributeKeyErrorReceipt: "upgrade timed-out",
					},
					sdk.EventTypeMessage: {
						sdk.AttributeKeyModule: channeltypes.AttributeValueCategory,
					},
				}
				ibctesting.AssertEventsLegacy(&suite.Suite, expEvents, events)
			},
		},
		{
			"core handler fails: invalid proof",
			func() {
				timeoutUpgrade()

				suite.Require().NoError(path.EndpointA.UpdateClient())

				_, _, proofHeight := path.EndpointB.QueryChannelUpgradeProof()

				msg.ProofHeight = proofHeight
				msg.ProofChannel = []byte("invalid proof")
			},
			func(res *channeltypes.MsgChannelUpgradeTimeoutResponse, events []abci.Event, err error) {
				suite.Require().Error(err)
				suite.Require().ErrorIs(err, commitmenttypes.ErrInvalidProof)

				channel := path.EndpointA.GetChannel()
				suite.Require().Equalf(channeltypes.FLUSHCOMPLETE, channel.State, "channel state should be %s", channeltypes.OPEN)
				suite.Require().Equal(uint64(1), channel.UpgradeSequence, "channel upgrade sequence should not incremented")

				_, found := path.EndpointA.Chain.GetSimApp().IBCKeeper.ChannelKeeper.GetUpgrade(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found, "channel upgrade should not be nil")

				suite.Require().Empty(events)
			},
		},
	}

	for _, tc := range cases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = ibcmock.UpgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = ibcmock.UpgradeVersion

			suite.Require().NoError(path.EndpointA.ChanUpgradeInit())
			suite.Require().NoError(path.EndpointB.ChanUpgradeTry())
			suite.Require().NoError(path.EndpointA.ChanUpgradeAck())

			channelKey := host.ChannelKey(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
			channelProof, proofHeight := path.EndpointB.QueryProof(channelKey)

			msg = &channeltypes.MsgChannelUpgradeTimeout{
				PortId:              path.EndpointA.ChannelConfig.PortID,
				ChannelId:           path.EndpointA.ChannelID,
				CounterpartyChannel: path.EndpointB.GetChannel(),
				ProofChannel:        channelProof,
				ProofHeight:         proofHeight,
				Signer:              suite.chainA.SenderAccount.GetAddress().String(),
			}

			tc.malleate()

			ctx := suite.chainA.GetContext()
			res, err := suite.chainA.GetSimApp().GetIBCKeeper().ChannelUpgradeTimeout(ctx, msg)
			events := ctx.EventManager().Events().ToABCIEvents()

			tc.expResult(res, events, err)
		})
	}
}

// TestIBCSoftwareUpgrade tests the IBCSoftwareUpgrade rpc handler
func (suite *KeeperTestSuite) TestIBCSoftwareUpgrade() {
	var msg *clienttypes.MsgIBCSoftwareUpgrade
	testCases := []struct {
		name     string
		malleate func()
		expError error
	}{
		{
			"success: valid authority and client upgrade",
			func() {},
			nil,
		},
		{
			"failure: invalid authority address",
			func() {
				msg.Signer = suite.chainA.SenderAccount.GetAddress().String()
			},
			ibcerrors.ErrUnauthorized,
		},
		{
			"failure: invalid clientState",
			func() {
				msg.UpgradedClientState = nil
			},
			clienttypes.ErrInvalidClientType,
		},
		{
			"failure: failed to schedule client upgrade",
			func() {
				msg.Plan.Height = 0
			},
			sdkerrors.ErrInvalidRequest,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			path := ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.SetupClients(path)
			validAuthority := suite.chainA.App.GetIBCKeeper().GetAuthority()
			plan := upgradetypes.Plan{
				Name:   "upgrade IBC clients",
				Height: 1000,
			}
			// update trusting period
			clientState := path.EndpointB.GetClientState()
			clientState.(*ibctm.ClientState).TrustingPeriod += 100

			var err error
			msg, err = clienttypes.NewMsgIBCSoftwareUpgrade(
				validAuthority,
				plan,
				clientState,
			)

			suite.Require().NoError(err)

			tc.malleate()

			_, err = keeper.Keeper.IBCSoftwareUpgrade(*suite.chainA.App.GetIBCKeeper(), suite.chainA.GetContext(), msg)

			if tc.expError == nil {
				suite.Require().NoError(err)
				// upgrade plan is stored
				storedPlan, err := suite.chainA.GetSimApp().UpgradeKeeper.GetUpgradePlan(suite.chainA.GetContext())
				suite.Require().NoError(err)
				suite.Require().Equal(plan, storedPlan)

				// upgraded client state is stored
				bz, err := suite.chainA.GetSimApp().UpgradeKeeper.GetUpgradedClient(suite.chainA.GetContext(), plan.Height)
				suite.Require().NoError(err)
				upgradedClientState, err := clienttypes.UnmarshalClientState(suite.chainA.App.AppCodec(), bz)
				suite.Require().NoError(err)
				suite.Require().Equal(clientState.ZeroCustomFields(), upgradedClientState)
			} else {
				suite.Require().True(errors.Is(err, tc.expError))
			}
		})
	}
}

// TestUpdateClientParams tests the UpdateClientParams rpc handler
func (suite *KeeperTestSuite) TestUpdateClientParams() {
	signer := suite.chainA.App.GetIBCKeeper().GetAuthority()
	testCases := []struct {
		name    string
		msg     *clienttypes.MsgUpdateParams
		expPass bool
	}{
		{
			"success: valid signer and default params",
			clienttypes.NewMsgUpdateParams(signer, clienttypes.DefaultParams()),
			true,
		},
		{
			"failure: malformed signer address",
			clienttypes.NewMsgUpdateParams(ibctesting.InvalidID, clienttypes.DefaultParams()),
			false,
		},
		{
			"failure: empty signer address",
			clienttypes.NewMsgUpdateParams("", clienttypes.DefaultParams()),
			false,
		},
		{
			"failure: whitespace signer address",
			clienttypes.NewMsgUpdateParams("    ", clienttypes.DefaultParams()),
			false,
		},
		{
			"failure: unauthorized signer address",
			clienttypes.NewMsgUpdateParams(ibctesting.TestAccAddress, clienttypes.DefaultParams()),
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()
			_, err := keeper.Keeper.UpdateClientParams(*suite.chainA.App.GetIBCKeeper(), suite.chainA.GetContext(), tc.msg)
			if tc.expPass {
				suite.Require().NoError(err)
				p := suite.chainA.App.GetIBCKeeper().ClientKeeper.GetParams(suite.chainA.GetContext())
				suite.Require().Equal(tc.msg.Params, p)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

// TestUpdateConnectionParams tests the UpdateConnectionParams rpc handler
func (suite *KeeperTestSuite) TestUpdateConnectionParams() {
	signer := suite.chainA.App.GetIBCKeeper().GetAuthority()
	testCases := []struct {
		name    string
		msg     *connectiontypes.MsgUpdateParams
		expPass bool
	}{
		{
			"success: valid signer and default params",
			connectiontypes.NewMsgUpdateParams(signer, connectiontypes.DefaultParams()),
			true,
		},
		{
			"failure: malformed signer address",
			connectiontypes.NewMsgUpdateParams(ibctesting.InvalidID, connectiontypes.DefaultParams()),
			false,
		},
		{
			"failure: empty signer address",
			connectiontypes.NewMsgUpdateParams("", connectiontypes.DefaultParams()),
			false,
		},
		{
			"failure: whitespace signer address",
			connectiontypes.NewMsgUpdateParams("    ", connectiontypes.DefaultParams()),
			false,
		},
		{
			"failure: unauthorized signer address",
			connectiontypes.NewMsgUpdateParams(ibctesting.TestAccAddress, connectiontypes.DefaultParams()),
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()
			_, err := keeper.Keeper.UpdateConnectionParams(*suite.chainA.App.GetIBCKeeper(), suite.chainA.GetContext(), tc.msg)
			if tc.expPass {
				suite.Require().NoError(err)
				p := suite.chainA.App.GetIBCKeeper().ConnectionKeeper.GetParams(suite.chainA.GetContext())
				suite.Require().Equal(tc.msg.Params, p)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

// TestUpdateChannelParams tests the UpdateChannelParams rpc handler
func (suite *KeeperTestSuite) TestUpdateChannelParams() {
	authority := suite.chainA.App.GetIBCKeeper().GetAuthority()
	testCases := []struct {
		name     string
		msg      *channeltypes.MsgUpdateParams
		expError error
	}{
		{
			"success: valid authority and default params",
			channeltypes.NewMsgUpdateChannelParams(authority, channeltypes.DefaultParams()),
			nil,
		},
		{
			"failure: malformed authority address",
			channeltypes.NewMsgUpdateChannelParams(ibctesting.InvalidID, channeltypes.DefaultParams()),
			ibcerrors.ErrUnauthorized,
		},
		{
			"failure: empty authority address",
			channeltypes.NewMsgUpdateChannelParams("", channeltypes.DefaultParams()),
			ibcerrors.ErrUnauthorized,
		},
		{
			"failure: empty signer",
			channeltypes.NewMsgUpdateChannelParams("", channeltypes.DefaultParams()),
			ibcerrors.ErrUnauthorized,
		},
		{
			"failure: unauthorized authority address",
			channeltypes.NewMsgUpdateChannelParams(ibctesting.TestAccAddress, channeltypes.DefaultParams()),
			ibcerrors.ErrUnauthorized,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()
			resp, err := keeper.Keeper.UpdateChannelParams(*suite.chainA.App.GetIBCKeeper(), suite.chainA.GetContext(), tc.msg)
			expPass := tc.expError == nil
			if expPass {
				suite.Require().NoError(err)
				suite.Require().NotNil(resp)
				p := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetParams(suite.chainA.GetContext())
				suite.Require().Equal(tc.msg.Params, p)
			} else {
				suite.Require().Nil(resp)
				suite.Require().ErrorIs(tc.expError, err)
			}
		})
	}
}

func (suite *KeeperTestSuite) TestPruneAcknowledgements() {
	var msg *channeltypes.MsgPruneAcknowledgements

	testCases := []struct {
		name     string
		malleate func()
		expErr   error
	}{
		{
			"success",
			func() {},
			nil,
		},
		{
			"failure: core keeper function fails, pruning sequence end not found",
			func() {
				msg.PortId = "portidone"
			},
			channeltypes.ErrRecvStartSequenceNotFound,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			path := ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			// configure the channel upgrade version on testing endpoints
			path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = ibcmock.UpgradeVersion
			path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = ibcmock.UpgradeVersion

			err := path.EndpointA.ChanUpgradeInit()
			suite.Require().NoError(err)

			err = path.EndpointB.ChanUpgradeTry()
			suite.Require().NoError(err)

			err = path.EndpointA.ChanUpgradeAck()
			suite.Require().NoError(err)

			err = path.EndpointB.ChanUpgradeConfirm()
			suite.Require().NoError(err)

			err = path.EndpointA.ChanUpgradeOpen()
			suite.Require().NoError(err)

			err = path.EndpointA.UpdateClient()
			suite.Require().NoError(err)

			msg = channeltypes.NewMsgPruneAcknowledgements(
				path.EndpointA.ChannelConfig.PortID,
				path.EndpointA.ChannelID,
				10,
				suite.chainA.SenderAccount.GetAddress().String(),
			)

			tc.malleate()

			resp, err := suite.chainA.App.GetIBCKeeper().PruneAcknowledgements(suite.chainA.GetContext(), msg)

			if tc.expErr == nil {
				suite.Require().NoError(err)
				suite.Require().NotNil(resp)
				suite.Require().Equal(uint64(0), resp.TotalPrunedSequences)
				suite.Require().Equal(uint64(0), resp.TotalRemainingSequences)
			} else {
				suite.Require().Error(err)
				suite.Require().Nil(resp)
			}
		})
	}
}
