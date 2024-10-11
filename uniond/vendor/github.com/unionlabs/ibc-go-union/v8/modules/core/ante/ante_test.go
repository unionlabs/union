package ante_test

import (
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	testifysuite "github.com/stretchr/testify/suite"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/ante"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	ibctm "github.com/cosmos/ibc-go/v8/modules/light-clients/07-tendermint"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

type AnteTestSuite struct {
	testifysuite.Suite

	coordinator *ibctesting.Coordinator

	// testing chains used for convenience and readability
	chainA *ibctesting.TestChain
	chainB *ibctesting.TestChain

	path *ibctesting.Path
}

// SetupTest creates a coordinator with 2 test chains.
func (suite *AnteTestSuite) SetupTest() {
	suite.coordinator = ibctesting.NewCoordinator(suite.T(), 2)
	suite.chainA = suite.coordinator.GetChain(ibctesting.GetChainID(1))
	suite.chainB = suite.coordinator.GetChain(ibctesting.GetChainID(2))
	// commit some blocks so that QueryProof returns valid proof (cannot return valid query if height <= 1)
	suite.coordinator.CommitNBlocks(suite.chainA, 2)
	suite.coordinator.CommitNBlocks(suite.chainB, 2)
	suite.path = ibctesting.NewPath(suite.chainA, suite.chainB)
	suite.coordinator.Setup(suite.path)
}

// TestAnteTestSuite runs all the tests within this package.
func TestAnteTestSuite(t *testing.T) {
	testifysuite.Run(t, new(AnteTestSuite))
}

// createRecvPacketMessage creates a RecvPacket message for a packet sent from chain A to chain B.
func (suite *AnteTestSuite) createRecvPacketMessage(isRedundant bool) *channeltypes.MsgRecvPacket {
	sequence, err := suite.path.EndpointA.SendPacket(clienttypes.NewHeight(2, 0), 0, ibctesting.MockPacketData)
	suite.Require().NoError(err)

	packet := channeltypes.NewPacket(ibctesting.MockPacketData, sequence,
		suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID,
		suite.path.EndpointB.ChannelConfig.PortID, suite.path.EndpointB.ChannelID,
		clienttypes.NewHeight(2, 0), 0)

	if isRedundant {
		err = suite.path.EndpointB.RecvPacket(packet)
		suite.Require().NoError(err)
	}

	err = suite.path.EndpointB.UpdateClient()
	suite.Require().NoError(err)

	packetKey := host.PacketCommitmentKey(packet.GetSourcePort(), packet.GetSourceChannel(), packet.GetSequence())
	proof, proofHeight := suite.chainA.QueryProof(packetKey)

	return channeltypes.NewMsgRecvPacket(packet, proof, proofHeight, suite.path.EndpointA.Chain.SenderAccount.GetAddress().String())
}

// createAcknowledgementMessage creates an Acknowledgement message for a packet sent from chain B to chain A.
func (suite *AnteTestSuite) createAcknowledgementMessage(isRedundant bool) sdk.Msg {
	sequence, err := suite.path.EndpointB.SendPacket(clienttypes.NewHeight(2, 0), 0, ibctesting.MockPacketData)
	suite.Require().NoError(err)

	packet := channeltypes.NewPacket(ibctesting.MockPacketData, sequence,
		suite.path.EndpointB.ChannelConfig.PortID, suite.path.EndpointB.ChannelID,
		suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID,
		clienttypes.NewHeight(2, 0), 0)
	err = suite.path.EndpointA.RecvPacket(packet)
	suite.Require().NoError(err)

	if isRedundant {
		err = suite.path.EndpointB.AcknowledgePacket(packet, ibctesting.MockAcknowledgement)
		suite.Require().NoError(err)
	}

	packetKey := host.PacketAcknowledgementKey(packet.GetSourcePort(), packet.GetSourceChannel(), packet.GetSequence())
	proof, proofHeight := suite.chainA.QueryProof(packetKey)

	return channeltypes.NewMsgAcknowledgement(packet, ibctesting.MockAcknowledgement, proof, proofHeight, suite.path.EndpointA.Chain.SenderAccount.GetAddress().String())
}

// createTimeoutMessage creates an Timeout message for a packet sent from chain B to chain A.
func (suite *AnteTestSuite) createTimeoutMessage(isRedundant bool) sdk.Msg {
	height := suite.chainA.LastHeader.GetHeight()
	timeoutHeight := clienttypes.NewHeight(height.GetRevisionNumber(), height.GetRevisionHeight()+1)

	sequence, err := suite.path.EndpointB.SendPacket(timeoutHeight, 0, ibctesting.MockPacketData)
	suite.Require().NoError(err)

	suite.coordinator.CommitNBlocks(suite.chainA, 3)

	err = suite.path.EndpointB.UpdateClient()
	suite.Require().NoError(err)

	packet := channeltypes.NewPacket(ibctesting.MockPacketData, sequence,
		suite.path.EndpointB.ChannelConfig.PortID, suite.path.EndpointB.ChannelID,
		suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID,
		timeoutHeight, 0)

	if isRedundant {
		err = suite.path.EndpointB.TimeoutPacket(packet)
		suite.Require().NoError(err)
	}

	packetKey := host.PacketReceiptKey(packet.GetSourcePort(), packet.GetSourceChannel(), packet.GetSequence())
	proof, proofHeight := suite.chainA.QueryProof(packetKey)

	return channeltypes.NewMsgTimeout(packet, sequence, proof, proofHeight, suite.path.EndpointA.Chain.SenderAccount.GetAddress().String())
}

// createTimeoutOnCloseMessage creates an TimeoutOnClose message for a packet sent from chain B to chain A.
func (suite *AnteTestSuite) createTimeoutOnCloseMessage(isRedundant bool) sdk.Msg {
	height := suite.chainA.LastHeader.GetHeight()
	timeoutHeight := clienttypes.NewHeight(height.GetRevisionNumber(), height.GetRevisionHeight()+1)

	sequence, err := suite.path.EndpointB.SendPacket(timeoutHeight, 0, ibctesting.MockPacketData)
	suite.Require().NoError(err)
	err = suite.path.EndpointA.SetChannelState(channeltypes.CLOSED)
	suite.Require().NoError(err)

	packet := channeltypes.NewPacket(ibctesting.MockPacketData, sequence,
		suite.path.EndpointB.ChannelConfig.PortID, suite.path.EndpointB.ChannelID,
		suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID,
		timeoutHeight, 0)

	if isRedundant {
		err = suite.path.EndpointB.TimeoutOnClose(packet)
		suite.Require().NoError(err)
	}

	packetKey := host.PacketReceiptKey(packet.GetDestPort(), packet.GetDestChannel(), packet.GetSequence())
	proof, proofHeight := suite.chainA.QueryProof(packetKey)

	channelKey := host.ChannelKey(packet.GetDestPort(), packet.GetDestChannel())
	closedProof, _ := suite.chainA.QueryProof(channelKey)

	return channeltypes.NewMsgTimeoutOnClose(packet, 1, proof, closedProof, proofHeight, suite.path.EndpointA.Chain.SenderAccount.GetAddress().String())
}

func (suite *AnteTestSuite) createUpdateClientMessage() sdk.Msg {
	endpoint := suite.path.EndpointB

	// ensure counterparty has committed state
	endpoint.Chain.Coordinator.CommitBlock(endpoint.Counterparty.Chain)

	var header exported.ClientMessage

	switch endpoint.ClientConfig.GetClientType() {
	case exported.Tendermint:
		tmClientState, ok := suite.path.EndpointB.GetClientState().(*ibctm.ClientState)
		suite.Require().True(ok)
		trustedHeight := tmClientState.LatestHeight
		header, _ = endpoint.Chain.IBCClientHeader(endpoint.Counterparty.Chain.LastHeader, trustedHeight)

	default:
	}

	msg, err := clienttypes.NewMsgUpdateClient(
		endpoint.ClientID, header,
		endpoint.Chain.SenderAccount.GetAddress().String(),
	)
	require.NoError(endpoint.Chain.TB, err)

	return msg
}

func (suite *AnteTestSuite) TestAnteDecoratorCheckTx() {
	testCases := []struct {
		name     string
		malleate func(suite *AnteTestSuite) []sdk.Msg
		expPass  bool
	}{
		{
			"success on one new RecvPacket message",
			func(suite *AnteTestSuite) []sdk.Msg {
				// the RecvPacket message has not been submitted to the chain yet, so it will succeed
				return []sdk.Msg{suite.createRecvPacketMessage(false)}
			},
			true,
		},
		{
			"success on one new Acknowledgement message",
			func(suite *AnteTestSuite) []sdk.Msg {
				// the Acknowledgement message has not been submitted to the chain yet, so it will succeed
				return []sdk.Msg{suite.createAcknowledgementMessage(false)}
			},
			true,
		},
		{
			"success on one new Timeout message",
			func(suite *AnteTestSuite) []sdk.Msg {
				// the Timeout message has not been submitted to the chain yet, so it will succeed
				return []sdk.Msg{suite.createTimeoutMessage(false)}
			},
			true,
		},
		{
			"success on one new TimeoutOnClose message",
			func(suite *AnteTestSuite) []sdk.Msg {
				// the TimeoutOnClose message has not been submitted to the chain yet, so it will succeed
				return []sdk.Msg{suite.createTimeoutOnCloseMessage(false)}
			},
			true,
		},
		{
			"success on three new messages of each type",
			func(suite *AnteTestSuite) []sdk.Msg {
				var msgs []sdk.Msg

				// none of the messages of each type has been submitted to the chain yet,
				// the first message is succeed and the next two of each type will be rejected
				// because they are redundant.

				// from A to B
				for i := 1; i <= 3; i++ {
					msgs = append(msgs, suite.createRecvPacketMessage(false))
				}

				// from B to A
				for i := 1; i <= 9; i++ {
					switch {
					case i >= 1 && i <= 3:
						msgs = append(msgs, suite.createAcknowledgementMessage(false))
					case i >= 4 && i <= 6:
						msgs = append(msgs, suite.createTimeoutMessage(false))
					case i >= 7 && i <= 9:
						msgs = append(msgs, suite.createTimeoutOnCloseMessage(false))
					}
				}
				return msgs
			},
			true,
		},
		{
			"success on three redundant messages of RecvPacket, Acknowledgement and TimeoutOnClose, and one new Timeout message",
			func(suite *AnteTestSuite) []sdk.Msg {
				var msgs []sdk.Msg

				// we pass three messages of RecvPacket, Acknowledgement and TimeoutOnClose that
				// are all redundant (i.e. those messages have already been submitted and
				// processed by the chain). But these messages will not be rejected because the
				// Timeout message is new.

				// from A to B
				for i := 1; i <= 3; i++ {
					msgs = append(msgs, suite.createRecvPacketMessage(true))
				}

				// from B to A
				for i := 1; i <= 7; i++ {
					switch {
					case i >= 1 && i <= 3:
						msgs = append(msgs, suite.createAcknowledgementMessage(true))
					case i == 4:
						msgs = append(msgs, suite.createTimeoutMessage(false))
					case i >= 5 && i <= 7:
						msgs = append(msgs, suite.createTimeoutOnCloseMessage(true))
					}
				}
				return msgs
			},
			true,
		},
		{
			"success on one new message and two redundant messages of each type",
			func(suite *AnteTestSuite) []sdk.Msg {
				var msgs []sdk.Msg

				// For each type there is a new message and two messages that are redundant
				// (i.e. they have been already submitted and processed by the chain). But all
				// the redundant messages will not be rejected because there is a new message
				// of each type.

				// from A to B
				for i := 1; i <= 3; i++ {
					msgs = append(msgs, suite.createRecvPacketMessage(i != 2))
				}

				// from B to A
				for i := 1; i <= 9; i++ {
					switch {
					case i >= 1 && i <= 3:
						msgs = append(msgs, suite.createAcknowledgementMessage(i != 2))
					case i >= 4 && i <= 6:
						msgs = append(msgs, suite.createTimeoutMessage(i != 5))
					case i >= 7 && i <= 9:
						msgs = append(msgs, suite.createTimeoutOnCloseMessage(i != 8))
					}
				}
				return msgs
			},
			true,
		},
		{
			"success on one new UpdateClient message",
			func(suite *AnteTestSuite) []sdk.Msg {
				return []sdk.Msg{suite.createUpdateClientMessage()}
			},
			true,
		},
		{
			"success on three new UpdateClient messages",
			func(suite *AnteTestSuite) []sdk.Msg {
				return []sdk.Msg{suite.createUpdateClientMessage(), suite.createUpdateClientMessage(), suite.createUpdateClientMessage()}
			},
			true,
		},
		{
			"success on three new Updateclient messages and one new RecvPacket message",
			func(suite *AnteTestSuite) []sdk.Msg {
				return []sdk.Msg{
					suite.createUpdateClientMessage(),
					suite.createUpdateClientMessage(),
					suite.createUpdateClientMessage(),
					suite.createRecvPacketMessage(false),
				}
			},
			true,
		},
		{
			"success on three redundant RecvPacket messages and one SubmitMisbehaviour message",
			func(suite *AnteTestSuite) []sdk.Msg {
				msgs := []sdk.Msg{suite.createUpdateClientMessage()}

				for i := 1; i <= 3; i++ {
					msgs = append(msgs, suite.createRecvPacketMessage(true))
				}

				// append non packet and update message to msgs to ensure multimsg tx should pass
				msgs = append(msgs, &clienttypes.MsgSubmitMisbehaviour{}) //nolint:staticcheck // we're using the deprecated message for testing
				return msgs
			},
			true,
		},
		{
			"success on app callback error, app callbacks are skipped for performance",
			func(suite *AnteTestSuite) []sdk.Msg {
				suite.chainB.GetSimApp().IBCMockModule.IBCApp.OnRecvPacket = func(
					ctx context.Context, packet channeltypes.Packet, relayer sdk.AccAddress,
				) exported.Acknowledgement {
					panic(fmt.Errorf("failed OnRecvPacket mock callback"))
				}

				// the RecvPacket message has not been submitted to the chain yet, so it will succeed
				return []sdk.Msg{suite.createRecvPacketMessage(false)}
			},
			true,
		},
		{
			"success on new UpdateClient messages: solomachine misbehaviour",
			func(suite *AnteTestSuite) []sdk.Msg {
				solomachine := ibctesting.NewSolomachine(suite.T(), suite.chainB.Codec, "06-solomachine-0", "testing", 1)
				suite.chainB.GetSimApp().GetIBCKeeper().ClientKeeper.SetClientState(suite.chainB.GetContext(), solomachine.ClientID, solomachine.ClientState())

				msgUpdateClient, err := clienttypes.NewMsgUpdateClient(solomachine.ClientID, solomachine.CreateMisbehaviour(), suite.chainB.SenderAccount.GetAddress().String())
				suite.Require().NoError(err)

				msgs := []sdk.Msg{msgUpdateClient}

				return msgs
			},
			true,
		},
		{
			"success on new UpdateClient messages: solomachine multisig misbehaviour",
			func(suite *AnteTestSuite) []sdk.Msg {
				solomachine := ibctesting.NewSolomachine(suite.T(), suite.chainA.Codec, "06-solomachine-0", "testing", 4)
				suite.chainB.GetSimApp().GetIBCKeeper().ClientKeeper.SetClientState(suite.chainB.GetContext(), solomachine.ClientID, solomachine.ClientState())

				msgUpdateClient, err := clienttypes.NewMsgUpdateClient(solomachine.ClientID, solomachine.CreateMisbehaviour(), suite.chainB.SenderAccount.GetAddress().String())
				suite.Require().NoError(err)

				msgs := []sdk.Msg{msgUpdateClient}

				return msgs
			},
			true,
		},
		{
			"success on new UpdateClient messages: tendermint misbehaviour",
			func(suite *AnteTestSuite) []sdk.Msg {
				trustedHeight := suite.path.EndpointB.GetClientState().GetLatestHeight().(clienttypes.Height)

				trustedVals, found := suite.chainA.TrustedValidators[trustedHeight.RevisionHeight]
				suite.Require().True(found)

				err := suite.path.EndpointB.UpdateClient()
				suite.Require().NoError(err)

				height := suite.path.EndpointB.GetClientState().GetLatestHeight().(clienttypes.Height)

				// construct valid fork misbehaviour: two headers at the same height with different time
				misbehaviour := &ibctm.Misbehaviour{
					Header1: suite.chainA.CreateTMClientHeader(suite.chainA.ChainID, int64(height.RevisionHeight), trustedHeight, suite.chainA.CurrentHeader.Time.Add(time.Minute), suite.chainA.Vals, suite.chainA.NextVals, trustedVals, suite.chainA.Signers),
					Header2: suite.chainA.CreateTMClientHeader(suite.chainA.ChainID, int64(height.RevisionHeight), trustedHeight, suite.chainA.CurrentHeader.Time, suite.chainA.Vals, suite.chainA.NextVals, trustedVals, suite.chainA.Signers),
				}

				msgUpdateClient, err := clienttypes.NewMsgUpdateClient(suite.path.EndpointB.ClientID, misbehaviour, suite.chainB.SenderAccount.GetAddress().String())
				suite.Require().NoError(err)

				msgs := []sdk.Msg{msgUpdateClient}

				return msgs
			},
			true,
		},
		{
			"no success on one redundant RecvPacket message",
			func(suite *AnteTestSuite) []sdk.Msg {
				return []sdk.Msg{suite.createRecvPacketMessage(true)}
			},
			false,
		},
		{
			"no success on three redundant messages of each type",
			func(suite *AnteTestSuite) []sdk.Msg {
				var msgs []sdk.Msg

				// from A to B
				for i := 1; i <= 3; i++ {
					msgs = append(msgs, suite.createRecvPacketMessage(true))
				}

				// from B to A
				for i := 1; i <= 9; i++ {
					switch {
					case i >= 1 && i <= 3:
						msgs = append(msgs, suite.createAcknowledgementMessage(true))
					case i >= 4 && i <= 6:
						msgs = append(msgs, suite.createTimeoutMessage(true))
					case i >= 7 && i <= 9:
						msgs = append(msgs, suite.createTimeoutOnCloseMessage(true))
					}
				}
				return msgs
			},
			false,
		},
		{
			"no success on one new UpdateClient message and three redundant RecvPacket messages",
			func(suite *AnteTestSuite) []sdk.Msg {
				msgs := []sdk.Msg{&clienttypes.MsgUpdateClient{}}

				for i := 1; i <= 3; i++ {
					msgs = append(msgs, suite.createRecvPacketMessage(true))
				}

				return msgs
			},
			false,
		},
		{
			"no success on one new UpdateClient message: invalid client identifier",
			func(suite *AnteTestSuite) []sdk.Msg {
				clientMsg, err := codectypes.NewAnyWithValue(&ibctm.Header{})
				suite.Require().NoError(err)

				msgs := []sdk.Msg{&clienttypes.MsgUpdateClient{ClientId: ibctesting.InvalidID, ClientMessage: clientMsg}}
				return msgs
			},
			false,
		},
		{
			"no success on one new UpdateClient message: client module not found",
			func(suite *AnteTestSuite) []sdk.Msg {
				clientMsg, err := codectypes.NewAnyWithValue(&ibctm.Header{})
				suite.Require().NoError(err)

				msgs := []sdk.Msg{&clienttypes.MsgUpdateClient{ClientId: clienttypes.FormatClientIdentifier("08-wasm", 1), ClientMessage: clientMsg}}
				return msgs
			},
			false,
		},
		{
			"no success on one new UpdateClient message: no consensus state for trusted height",
			func(suite *AnteTestSuite) []sdk.Msg {
				clientMsg, err := codectypes.NewAnyWithValue(&ibctm.Header{TrustedHeight: clienttypes.NewHeight(1, 10000)})
				suite.Require().NoError(err)

				msgs := []sdk.Msg{&clienttypes.MsgUpdateClient{ClientId: suite.path.EndpointA.ClientID, ClientMessage: clientMsg}}
				return msgs
			},
			false,
		},
		{
			"no success on three new UpdateClient messages and three redundant messages of each type",
			func(suite *AnteTestSuite) []sdk.Msg {
				msgs := []sdk.Msg{suite.createUpdateClientMessage(), suite.createUpdateClientMessage(), suite.createUpdateClientMessage()}

				// from A to B
				for i := 1; i <= 3; i++ {
					msgs = append(msgs, suite.createRecvPacketMessage(true))
				}

				// from B to A
				for i := 1; i <= 9; i++ {
					switch {
					case i >= 1 && i <= 3:
						msgs = append(msgs, suite.createAcknowledgementMessage(true))
					case i >= 4 && i <= 6:
						msgs = append(msgs, suite.createTimeoutMessage(true))
					case i >= 7 && i <= 9:
						msgs = append(msgs, suite.createTimeoutOnCloseMessage(true))
					}
				}
				return msgs
			},
			false,
		},
		{
			"no success on one new message and one invalid message",
			func(suite *AnteTestSuite) []sdk.Msg {
				packet := channeltypes.NewPacket(ibctesting.MockPacketData, 2,
					suite.path.EndpointA.ChannelConfig.PortID, suite.path.EndpointA.ChannelID,
					suite.path.EndpointB.ChannelConfig.PortID, suite.path.EndpointB.ChannelID,
					clienttypes.NewHeight(2, 0), 0)

				return []sdk.Msg{
					suite.createRecvPacketMessage(false),
					channeltypes.NewMsgRecvPacket(packet, []byte("proof"), clienttypes.NewHeight(1, 1), "signer"),
				}
			},
			false,
		},
		{
			"no success on one new message and one redundant message in the same block",
			func(suite *AnteTestSuite) []sdk.Msg {
				msg := suite.createRecvPacketMessage(false)

				// We want to be able to run check tx with the non-redundant message without
				// committing it to a block, so that the when check tx runs with the redundant
				// message they are both in the same block
				k := suite.chainB.App.GetIBCKeeper()
				decorator := ante.NewRedundantRelayDecorator(k)
				checkCtx := suite.chainB.GetContext().WithIsCheckTx(true)
				next := func(ctx sdk.Context, tx sdk.Tx, simulate bool) (newCtx sdk.Context, err error) { return ctx, nil }
				txBuilder := suite.chainB.TxConfig.NewTxBuilder()
				err := txBuilder.SetMsgs([]sdk.Msg{msg}...)
				suite.Require().NoError(err)
				tx := txBuilder.GetTx()

				_, err = decorator.AnteHandle(checkCtx, tx, false, next)
				suite.Require().NoError(err)

				return []sdk.Msg{msg}
			},
			false,
		},
		{
			"no success on recvPacket checkTx, no capability found",
			func(suite *AnteTestSuite) []sdk.Msg {
				msg := suite.createRecvPacketMessage(false)
				msg.Packet.DestinationPort = "invalid-port"
				return []sdk.Msg{msg}
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			// reset suite
			suite.SetupTest()

			k := suite.chainB.App.GetIBCKeeper()
			decorator := ante.NewRedundantRelayDecorator(k)

			msgs := tc.malleate(suite)

			deliverCtx := suite.chainB.GetContext().WithIsCheckTx(false)
			checkCtx := suite.chainB.GetContext().WithIsCheckTx(true)

			// create multimsg tx
			txBuilder := suite.chainB.TxConfig.NewTxBuilder()
			err := txBuilder.SetMsgs(msgs...)
			suite.Require().NoError(err)
			tx := txBuilder.GetTx()

			next := func(ctx sdk.Context, tx sdk.Tx, simulate bool) (newCtx sdk.Context, err error) { return ctx, nil }

			_, err = decorator.AnteHandle(deliverCtx, tx, false, next)
			suite.Require().NoError(err, "antedecorator should not error on DeliverTx")

			_, err = decorator.AnteHandle(checkCtx, tx, false, next)
			if tc.expPass {
				suite.Require().NoError(err, "non-strict decorator did not pass as expected")
			} else {
				suite.Require().Error(err, "non-strict antehandler did not return error as expected")
			}
		})
	}
}

func (suite *AnteTestSuite) TestAnteDecoratorReCheckTx() {
	testCases := []struct {
		name     string
		malleate func(suite *AnteTestSuite) []sdk.Msg
		expError error
	}{
		{
			"success on one new RecvPacket message",
			func(suite *AnteTestSuite) []sdk.Msg {
				// the RecvPacket message has not been submitted to the chain yet, so it will succeed
				return []sdk.Msg{suite.createRecvPacketMessage(false)}
			},
			nil,
		},
		{
			"success on one redundant and one new RecvPacket message",
			func(suite *AnteTestSuite) []sdk.Msg {
				return []sdk.Msg{
					suite.createRecvPacketMessage(true),
					suite.createRecvPacketMessage(false),
				}
			},
			nil,
		},
		{
			"success on invalid proof (proof checks occur in checkTx)",
			func(suite *AnteTestSuite) []sdk.Msg {
				msg := suite.createRecvPacketMessage(false)
				msg.ProofCommitment = []byte("invalid-proof")
				return []sdk.Msg{msg}
			},
			nil,
		},
		{
			"success on app callback error, app callbacks are skipped for performance",
			func(suite *AnteTestSuite) []sdk.Msg {
				suite.chainB.GetSimApp().IBCMockModule.IBCApp.OnRecvPacket = func(
					ctx context.Context, packet channeltypes.Packet, relayer sdk.AccAddress,
				) exported.Acknowledgement {
					panic(fmt.Errorf("failed OnRecvPacket mock callback"))
				}

				// the RecvPacket message has not been submitted to the chain yet, so it will succeed
				return []sdk.Msg{suite.createRecvPacketMessage(false)}
			},
			nil,
		},
		{
			"no success on one redundant RecvPacket message",
			func(suite *AnteTestSuite) []sdk.Msg {
				return []sdk.Msg{suite.createRecvPacketMessage(true)}
			},
			channeltypes.ErrRedundantTx,
		},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			// reset suite
			suite.SetupTest()

			k := suite.chainB.App.GetIBCKeeper()
			decorator := ante.NewRedundantRelayDecorator(k)

			msgs := tc.malleate(suite)

			deliverCtx := suite.chainB.GetContext().WithIsCheckTx(false)
			reCheckCtx := suite.chainB.GetContext().WithIsReCheckTx(true)

			// create multimsg tx
			txBuilder := suite.chainB.TxConfig.NewTxBuilder()
			err := txBuilder.SetMsgs(msgs...)
			suite.Require().NoError(err)
			tx := txBuilder.GetTx()

			next := func(ctx sdk.Context, tx sdk.Tx, simulate bool) (newCtx sdk.Context, err error) { return ctx, nil }

			_, err = decorator.AnteHandle(deliverCtx, tx, false, next)
			suite.Require().NoError(err, "antedecorator should not error on DeliverTx")

			_, err = decorator.AnteHandle(reCheckCtx, tx, false, next)
			if tc.expError == nil {
				suite.Require().NoError(err, "non-strict decorator did not pass as expected")
			} else {
				suite.Require().ErrorIs(err, tc.expError, "non-strict antehandler did not return error as expected")
			}
		})
	}
}
