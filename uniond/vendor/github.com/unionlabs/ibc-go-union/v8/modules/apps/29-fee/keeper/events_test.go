package keeper_test

import (
	sdkmath "cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"

	abcitypes "github.com/cometbft/cometbft/abci/types"

	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/types"
	transfertypes "github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

func (suite *KeeperTestSuite) TestIncentivizePacketEvent() {
	var (
		expRecvFees    sdk.Coins
		expAckFees     sdk.Coins
		expTimeoutFees sdk.Coins
	)

	suite.coordinator.Setup(suite.path)

	fee := types.NewFee(defaultRecvFee, defaultAckFee, defaultTimeoutFee)
	msg := types.NewMsgPayPacketFee(
		fee,
		suite.path.EndpointA.ChannelConfig.PortID,
		suite.path.EndpointA.ChannelID,
		suite.chainA.SenderAccount.GetAddress().String(),
		nil,
	)

	expRecvFees = expRecvFees.Add(fee.RecvFee...)
	expAckFees = expAckFees.Add(fee.AckFee...)
	expTimeoutFees = expTimeoutFees.Add(fee.TimeoutFee...)

	result, err := suite.chainA.SendMsgs(msg)
	suite.Require().NoError(err)

	var incentivizedPacketEvent abcitypes.Event
	for _, event := range result.Events {
		if event.Type == types.EventTypeIncentivizedPacket {
			incentivizedPacketEvent = event
		}
	}

	for _, attr := range incentivizedPacketEvent.Attributes {
		switch attr.Key {
		case types.AttributeKeyRecvFee:
			suite.Require().Equal(expRecvFees.String(), attr.Value)

		case types.AttributeKeyAckFee:
			suite.Require().Equal(expAckFees.String(), attr.Value)

		case types.AttributeKeyTimeoutFee:
			suite.Require().Equal(expTimeoutFees.String(), attr.Value)
		}
	}

	// send the same messages again a few times
	for i := 0; i < 3; i++ {
		expRecvFees = expRecvFees.Add(fee.RecvFee...)
		expAckFees = expAckFees.Add(fee.AckFee...)
		expTimeoutFees = expTimeoutFees.Add(fee.TimeoutFee...)

		result, err = suite.chainA.SendMsgs(msg)
		suite.Require().NoError(err)
	}

	for _, event := range result.Events {
		if event.Type == types.EventTypeIncentivizedPacket {
			incentivizedPacketEvent = event
		}
	}

	for _, attr := range incentivizedPacketEvent.Attributes {
		switch attr.Key {
		case types.AttributeKeyRecvFee:
			suite.Require().Equal(expRecvFees.String(), attr.Value)

		case types.AttributeKeyAckFee:
			suite.Require().Equal(expAckFees.String(), attr.Value)

		case types.AttributeKeyTimeoutFee:
			suite.Require().Equal(expTimeoutFees.String(), attr.Value)
		}
	}
}

func (suite *KeeperTestSuite) TestDistributeFeeEvent() {
	// create an incentivized transfer path
	path := ibctesting.NewPath(suite.chainA, suite.chainB)
	feeTransferVersion := string(types.ModuleCdc.MustMarshalJSON(&types.Metadata{FeeVersion: types.Version, AppVersion: transfertypes.Version}))
	path.EndpointA.ChannelConfig.Version = feeTransferVersion
	path.EndpointB.ChannelConfig.Version = feeTransferVersion
	path.EndpointA.ChannelConfig.PortID = transfertypes.PortID
	path.EndpointB.ChannelConfig.PortID = transfertypes.PortID

	suite.coordinator.Setup(path)

	// send a new MsgPayPacketFee and MsgTransfer to chainA
	fee := types.NewFee(defaultRecvFee, defaultAckFee, defaultTimeoutFee)
	msgPayPacketFee := types.NewMsgPayPacketFee(
		fee,
		path.EndpointA.ChannelConfig.PortID,
		path.EndpointA.ChannelID,
		suite.chainA.SenderAccount.GetAddress().String(),
		nil,
	)

	msgTransfer := transfertypes.NewMsgTransfer(
		path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID,
		sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100)), suite.chainA.SenderAccount.GetAddress().String(), suite.chainB.SenderAccount.GetAddress().String(),
		clienttypes.NewHeight(1, 100), 0, "",
	)

	res, err := suite.chainA.SendMsgs(msgPayPacketFee, msgTransfer)
	suite.Require().NoError(err)
	suite.Require().NotNil(res)

	// parse the packet from result events and recv packet on chainB
	packet, err := ibctesting.ParsePacketFromEvents(res.Events)
	suite.Require().NoError(err)
	suite.Require().NotNil(packet)

	err = path.EndpointB.UpdateClient()
	suite.Require().NoError(err)

	res, err = path.EndpointB.RecvPacketWithResult(packet)
	suite.Require().NoError(err)
	suite.Require().NotNil(res)

	// parse the acknowledgement from result events and acknowledge packet on chainA
	ack, err := ibctesting.ParseAckFromEvents(res.Events)
	suite.Require().NoError(err)
	suite.Require().NotNil(ack)

	packetKey := host.PacketAcknowledgementKey(packet.GetDestPort(), packet.GetDestChannel(), packet.GetSequence())
	proof, proofHeight := path.EndpointA.Counterparty.QueryProof(packetKey)

	msgAcknowledgement := channeltypes.NewMsgAcknowledgement(packet, ack, proof, proofHeight, path.EndpointA.Chain.SenderAccount.GetAddress().String())
	res, err = suite.chainA.SendMsgs(msgAcknowledgement)
	suite.Require().NoError(err)
	suite.Require().NotNil(res)

	events := res.Events
	expectedEvents := sdk.Events{
		sdk.NewEvent(
			types.EventTypeDistributeFee,
			sdk.NewAttribute(types.AttributeKeyReceiver, suite.chainA.SenderAccount.GetAddress().String()),
			sdk.NewAttribute(types.AttributeKeyFee, defaultRecvFee.String()),
		),
		sdk.NewEvent(
			types.EventTypeDistributeFee,
			sdk.NewAttribute(types.AttributeKeyReceiver, suite.chainA.SenderAccount.GetAddress().String()),
			sdk.NewAttribute(types.AttributeKeyFee, defaultAckFee.String()),
		),
		sdk.NewEvent(
			types.EventTypeDistributeFee,
			sdk.NewAttribute(types.AttributeKeyReceiver, suite.chainA.SenderAccount.GetAddress().String()),
			sdk.NewAttribute(types.AttributeKeyFee, sdk.NewCoins().String()),
		),
	}.ToABCIEvents()

	var indexSet map[string]struct{}
	expectedEvents = sdk.MarkEventsToIndex(expectedEvents, indexSet)
	ibctesting.AssertEvents(&suite.Suite, expectedEvents, events)
}
