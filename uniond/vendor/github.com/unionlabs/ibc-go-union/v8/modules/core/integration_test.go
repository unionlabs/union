package ibc_test

import (
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
	ibcmock "github.com/cosmos/ibc-go/v8/testing/mock"
)

// If packet receipts are pruned, it may be possible to double spend via a
// replay attack by resubmitting the same proof used to process the original receive.
// Core IBC performs an additional check to ensure that any packet being received
// MUST NOT be in the range of packet receipts which are allowed to be pruned thus
// adding replay protection for upgraded channels.
// This test has been added to ensure we have replay protection after
// pruning stale state upon the successful completion of a channel upgrade.
func (suite *IBCTestSuite) TestReplayProtectionAfterReceivePruning() {
	var path *ibctesting.Path

	testCases := []struct {
		name     string
		malleate func()
	}{
		{
			"unordered channel upgrades version",
			func() {
				path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Version = ibcmock.UpgradeVersion
				path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Version = ibcmock.UpgradeVersion
			},
		},
		{
			"ordered channel upgrades to unordered channel",
			func() {
				path.EndpointA.ChannelConfig.Order = channeltypes.ORDERED
				path.EndpointB.ChannelConfig.Order = channeltypes.ORDERED

				path.EndpointA.ChannelConfig.ProposedUpgrade.Fields.Ordering = channeltypes.UNORDERED
				path.EndpointB.ChannelConfig.ProposedUpgrade.Fields.Ordering = channeltypes.UNORDERED
			},
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()
			path = ibctesting.NewPath(suite.chainA, suite.chainB)

			tc.malleate()

			suite.coordinator.Setup(path)

			// Setup replay attack by sending a packet. We will save the receive
			// proof to replay relaying after the channel upgrade compeletes.
			disabledTimeoutTimestamp := uint64(0)
			timeoutHeight := clienttypes.NewHeight(1, 110)
			sequence, err := path.EndpointA.SendPacket(timeoutHeight, disabledTimeoutTimestamp, ibctesting.MockPacketData)
			suite.Require().NoError(err)
			packet := channeltypes.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, timeoutHeight, disabledTimeoutTimestamp)

			// save receive proof for replay submission
			packetKey := host.PacketCommitmentKey(packet.GetSourcePort(), packet.GetSourceChannel(), packet.GetSequence())
			proof, proofHeight := path.EndpointA.Chain.QueryProof(packetKey)
			recvMsg := channeltypes.NewMsgRecvPacket(packet, proof, proofHeight, path.EndpointB.Chain.SenderAccount.GetAddress().String())

			err = path.RelayPacket(packet)
			suite.Require().NoError(err)

			// perform upgrade
			err = path.EndpointA.ChanUpgradeInit()
			suite.Require().NoError(err)

			err = path.EndpointB.ChanUpgradeTry()
			suite.Require().NoError(err)

			err = path.EndpointA.ChanUpgradeAck()
			suite.Require().NoError(err)

			err = path.EndpointB.ChanUpgradeConfirm()
			suite.Require().NoError(err)

			err = path.EndpointA.ChanUpgradeOpen()
			suite.Require().NoError(err)

			// prune stale receive state
			msgPrune := channeltypes.NewMsgPruneAcknowledgements(path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, 1, path.EndpointB.Chain.SenderAccount.GetAddress().String())
			res, err := path.EndpointB.Chain.SendMsgs(msgPrune)
			suite.Require().NotNil(res)
			suite.Require().NoError(err)

			// replay initial packet send
			res, err = path.EndpointB.Chain.SendMsgs(recvMsg)
			suite.Require().NotNil(res)
			suite.Require().ErrorContains(err, channeltypes.ErrPacketReceived.Error(), "replay protection missing")
		})
	}
}
