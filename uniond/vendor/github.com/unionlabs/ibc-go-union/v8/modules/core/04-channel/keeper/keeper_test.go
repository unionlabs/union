package keeper_test

import (
	"fmt"
	"reflect"
	"testing"

	testifysuite "github.com/stretchr/testify/suite"

	transfertypes "github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
	ibcmock "github.com/cosmos/ibc-go/v8/testing/mock"
)

// KeeperTestSuite is a testing suite to test keeper functions.
type KeeperTestSuite struct {
	testifysuite.Suite

	coordinator *ibctesting.Coordinator

	// testing chains used for convenience and readability
	chainA *ibctesting.TestChain
	chainB *ibctesting.TestChain
}

// TestKeeperTestSuite runs all the tests within this package.
func TestKeeperTestSuite(t *testing.T) {
	testifysuite.Run(t, new(KeeperTestSuite))
}

// SetupTest creates a coordinator with 2 test chains.
func (suite *KeeperTestSuite) SetupTest() {
	suite.coordinator = ibctesting.NewCoordinator(suite.T(), 2)
	suite.chainA = suite.coordinator.GetChain(ibctesting.GetChainID(1))
	suite.chainB = suite.coordinator.GetChain(ibctesting.GetChainID(2))
	// commit some blocks so that QueryProof returns valid proof (cannot return valid query if height <= 1)
	suite.coordinator.CommitNBlocks(suite.chainA, 2)
	suite.coordinator.CommitNBlocks(suite.chainB, 2)
}

// TestSetChannel create clients and connections on both chains. It tests for the non-existence
// and existence of a channel in INIT on chainA.
func (suite *KeeperTestSuite) TestSetChannel() {
	// create client and connections on both chains
	path := ibctesting.NewPath(suite.chainA, suite.chainB)
	suite.coordinator.SetupConnections(path)

	// check for channel to be created on chainA
	found := suite.chainA.App.GetIBCKeeper().ChannelKeeper.HasChannel(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
	suite.False(found)

	path.SetChannelOrdered()

	// init channel
	err := path.EndpointA.ChanOpenInit()
	suite.NoError(err)

	storedChannel, found := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetChannel(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
	// counterparty channel id is empty after open init
	expectedCounterparty := types.NewCounterparty(path.EndpointB.ChannelConfig.PortID, "")

	suite.True(found)
	suite.Equal(types.INIT, storedChannel.State)
	suite.Equal(types.ORDERED, storedChannel.Ordering)
	suite.Equal(expectedCounterparty, storedChannel.Counterparty)
}

func (suite *KeeperTestSuite) TestGetAppVersion() {
	// create client and connections on both chains
	path := ibctesting.NewPath(suite.chainA, suite.chainB)
	suite.coordinator.SetupConnections(path)

	version, found := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetAppVersion(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
	suite.Require().False(found)
	suite.Require().Empty(version)

	// init channel
	err := path.EndpointA.ChanOpenInit()
	suite.NoError(err)

	channelVersion, found := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetAppVersion(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
	suite.Require().True(found)
	suite.Require().Equal(ibcmock.Version, channelVersion)
}

// TestGetAllChannelsWithPortPrefix verifies ports are filtered correctly using a port prefix.
func (suite *KeeperTestSuite) TestGetAllChannelsWithPortPrefix() {
	const (
		secondChannelID        = "channel-1"
		differentChannelPortID = "different-portid"
	)

	allChannels := []types.IdentifiedChannel{
		types.NewIdentifiedChannel(transfertypes.PortID, ibctesting.FirstChannelID, types.Channel{}),
		types.NewIdentifiedChannel(differentChannelPortID, secondChannelID, types.Channel{}),
	}

	tests := []struct {
		name             string
		prefix           string
		allChannels      []types.IdentifiedChannel
		expectedChannels []types.IdentifiedChannel
	}{
		{
			name:             "transfer channel is retrieved with prefix",
			prefix:           "tra",
			allChannels:      allChannels,
			expectedChannels: []types.IdentifiedChannel{types.NewIdentifiedChannel(transfertypes.PortID, ibctesting.FirstChannelID, types.Channel{})},
		},
		{
			name:             "matches port with full name as prefix",
			prefix:           transfertypes.PortID,
			allChannels:      allChannels,
			expectedChannels: []types.IdentifiedChannel{types.NewIdentifiedChannel(transfertypes.PortID, ibctesting.FirstChannelID, types.Channel{})},
		},
		{
			name:             "no ports match prefix",
			prefix:           "wont-match-anything",
			allChannels:      allChannels,
			expectedChannels: nil,
		},
		{
			name:             "empty prefix matches everything",
			prefix:           "",
			allChannels:      allChannels,
			expectedChannels: allChannels,
		},
	}

	for _, tc := range tests {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest()

			for _, ch := range tc.allChannels {
				suite.chainA.GetSimApp().GetIBCKeeper().ChannelKeeper.SetChannel(suite.chainA.GetContext(), ch.PortId, ch.ChannelId, types.Channel{})
			}

			ctxA := suite.chainA.GetContext()

			actualChannels := suite.chainA.GetSimApp().GetIBCKeeper().ChannelKeeper.GetAllChannelsWithPortPrefix(ctxA, tc.prefix)

			suite.Require().True(containsAll(tc.expectedChannels, actualChannels))
		})
	}
}

// containsAll verifies if all elements in the expected slice exist in the actual slice
// independent of order.
func containsAll(expected, actual []types.IdentifiedChannel) bool {
	for _, expectedChannel := range expected {
		foundMatch := false
		for _, actualChannel := range actual {
			if reflect.DeepEqual(actualChannel, expectedChannel) {
				foundMatch = true
				break
			}
		}
		if !foundMatch {
			return false
		}
	}
	return true
}

// TestGetAllChannels creates multiple channels on chain A through various connections
// and tests their retrieval. 2 channels are on connA0 and 1 channel is on connA1
func (suite *KeeperTestSuite) TestGetAllChannels() { //nolint:govet // this is a test, we are okay with copying locks
	path := ibctesting.NewPath(suite.chainA, suite.chainB)
	suite.coordinator.Setup(path)
	// channel0 on first connection on chainA
	counterparty0 := types.Counterparty{
		PortId:    path.EndpointB.ChannelConfig.PortID,
		ChannelId: path.EndpointB.ChannelID,
	}

	// path1 creates a second channel on first connection on chainA
	path1 := ibctesting.NewPath(suite.chainA, suite.chainB)
	path1.SetChannelOrdered()
	path1.EndpointA.ClientID = path.EndpointA.ClientID
	path1.EndpointB.ClientID = path.EndpointB.ClientID
	path1.EndpointA.ConnectionID = path.EndpointA.ConnectionID
	path1.EndpointB.ConnectionID = path.EndpointB.ConnectionID

	suite.coordinator.CreateMockChannels(path1)
	counterparty1 := types.Counterparty{
		PortId:    path1.EndpointB.ChannelConfig.PortID,
		ChannelId: path1.EndpointB.ChannelID,
	}

	path2 := ibctesting.NewPath(suite.chainA, suite.chainB)
	suite.coordinator.SetupConnections(path2)

	// path2 creates a second channel on chainA
	err := path2.EndpointA.ChanOpenInit()
	suite.Require().NoError(err)

	// counterparty channel id is empty after open init
	counterparty2 := types.Counterparty{
		PortId:    path2.EndpointB.ChannelConfig.PortID,
		ChannelId: "",
	}

	channel0 := types.NewChannel(
		types.OPEN, types.UNORDERED,
		counterparty0, []string{path.EndpointA.ConnectionID}, path.EndpointA.ChannelConfig.Version,
	)
	channel1 := types.NewChannel(
		types.OPEN, types.ORDERED,
		counterparty1, []string{path1.EndpointA.ConnectionID}, path1.EndpointA.ChannelConfig.Version,
	)
	channel2 := types.NewChannel(
		types.INIT, types.UNORDERED,
		counterparty2, []string{path2.EndpointA.ConnectionID}, path2.EndpointA.ChannelConfig.Version,
	)

	expChannels := []types.IdentifiedChannel{
		types.NewIdentifiedChannel(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, channel0),
		types.NewIdentifiedChannel(path1.EndpointA.ChannelConfig.PortID, path1.EndpointA.ChannelID, channel1),
		types.NewIdentifiedChannel(path2.EndpointA.ChannelConfig.PortID, path2.EndpointA.ChannelID, channel2),
	}

	ctxA := suite.chainA.GetContext()

	channels := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetAllChannels(ctxA)
	suite.Require().Len(channels, len(expChannels))
	suite.Require().Equal(expChannels, channels)
}

// TestGetAllSequences sets all packet sequences for two different channels on chain A and
// tests their retrieval.
func (suite *KeeperTestSuite) TestGetAllSequences() { //nolint:govet // this is a test, we are okay with copying locks
	path := ibctesting.NewPath(suite.chainA, suite.chainB)
	suite.coordinator.Setup(path)

	path1 := ibctesting.NewPath(suite.chainA, suite.chainB)
	path1.SetChannelOrdered()
	path1.EndpointA.ClientID = path.EndpointA.ClientID
	path1.EndpointB.ClientID = path.EndpointB.ClientID
	path1.EndpointA.ConnectionID = path.EndpointA.ConnectionID
	path1.EndpointB.ConnectionID = path.EndpointB.ConnectionID

	suite.coordinator.CreateMockChannels(path1)

	seq1 := types.NewPacketSequence(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, 1)
	seq2 := types.NewPacketSequence(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, 2)
	seq3 := types.NewPacketSequence(path1.EndpointA.ChannelConfig.PortID, path1.EndpointA.ChannelID, 3)

	// seq1 should be overwritten by seq2
	expSeqs := []types.PacketSequence{seq2, seq3}

	ctxA := suite.chainA.GetContext()

	for _, seq := range []types.PacketSequence{seq1, seq2, seq3} {
		suite.chainA.App.GetIBCKeeper().ChannelKeeper.SetNextSequenceSend(ctxA, seq.PortId, seq.ChannelId, seq.Sequence)
		suite.chainA.App.GetIBCKeeper().ChannelKeeper.SetNextSequenceRecv(ctxA, seq.PortId, seq.ChannelId, seq.Sequence)
		suite.chainA.App.GetIBCKeeper().ChannelKeeper.SetNextSequenceAck(ctxA, seq.PortId, seq.ChannelId, seq.Sequence)
	}

	sendSeqs := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetAllPacketSendSeqs(ctxA)
	recvSeqs := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetAllPacketRecvSeqs(ctxA)
	ackSeqs := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetAllPacketAckSeqs(ctxA)
	suite.Len(sendSeqs, 2)
	suite.Len(recvSeqs, 2)
	suite.Len(ackSeqs, 2)

	suite.Equal(expSeqs, sendSeqs)
	suite.Equal(expSeqs, recvSeqs)
	suite.Equal(expSeqs, ackSeqs)
}

// TestGetAllPacketState creates a set of acks, packet commitments, and receipts on two different
// channels on chain A and tests their retrieval.
func (suite *KeeperTestSuite) TestGetAllPacketState() { //nolint:govet // this is a test, we are okay with copying locks
	path := ibctesting.NewPath(suite.chainA, suite.chainB)
	suite.coordinator.Setup(path)

	path1 := ibctesting.NewPath(suite.chainA, suite.chainB)
	path1.EndpointA.ClientID = path.EndpointA.ClientID
	path1.EndpointB.ClientID = path.EndpointB.ClientID
	path1.EndpointA.ConnectionID = path.EndpointA.ConnectionID
	path1.EndpointB.ConnectionID = path.EndpointB.ConnectionID

	suite.coordinator.CreateMockChannels(path1)

	// channel 0 acks
	ack1 := types.NewPacketState(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, 1, []byte("ack"))
	ack2 := types.NewPacketState(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, 2, []byte("ack"))

	// duplicate ack
	ack2dup := types.NewPacketState(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, 2, []byte("ack"))

	// channel 1 acks
	ack3 := types.NewPacketState(path1.EndpointA.ChannelConfig.PortID, path1.EndpointA.ChannelID, 1, []byte("ack"))

	// create channel 0 receipts
	receipt := string([]byte{byte(1)})
	rec1 := types.NewPacketState(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, 1, []byte(receipt))
	rec2 := types.NewPacketState(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, 2, []byte(receipt))

	// channel 1 receipts
	rec3 := types.NewPacketState(path1.EndpointA.ChannelConfig.PortID, path1.EndpointA.ChannelID, 1, []byte(receipt))
	rec4 := types.NewPacketState(path1.EndpointA.ChannelConfig.PortID, path1.EndpointA.ChannelID, 2, []byte(receipt))

	// channel 0 packet commitments
	comm1 := types.NewPacketState(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, 1, []byte("hash"))
	comm2 := types.NewPacketState(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, 2, []byte("hash"))

	// channel 1 packet commitments
	comm3 := types.NewPacketState(path1.EndpointA.ChannelConfig.PortID, path1.EndpointA.ChannelID, 1, []byte("hash"))
	comm4 := types.NewPacketState(path1.EndpointA.ChannelConfig.PortID, path1.EndpointA.ChannelID, 2, []byte("hash"))

	expAcks := []types.PacketState{ack1, ack2, ack3}
	expReceipts := []types.PacketState{rec1, rec2, rec3, rec4}
	expCommitments := []types.PacketState{comm1, comm2, comm3, comm4}

	ctxA := suite.chainA.GetContext()

	// set acknowledgements
	for _, ack := range []types.PacketState{ack1, ack2, ack2dup, ack3} {
		suite.chainA.App.GetIBCKeeper().ChannelKeeper.SetPacketAcknowledgement(ctxA, ack.PortId, ack.ChannelId, ack.Sequence, ack.Data)
	}

	// set packet receipts
	for _, rec := range expReceipts {
		suite.chainA.App.GetIBCKeeper().ChannelKeeper.SetPacketReceipt(ctxA, rec.PortId, rec.ChannelId, rec.Sequence)
	}

	// set packet commitments
	for _, comm := range expCommitments {
		suite.chainA.App.GetIBCKeeper().ChannelKeeper.SetPacketCommitment(ctxA, comm.PortId, comm.ChannelId, comm.Sequence, comm.Data)
	}

	acks := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetAllPacketAcks(ctxA)
	receipts := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetAllPacketReceipts(ctxA)
	commitments := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetAllPacketCommitments(ctxA)

	suite.Require().Len(acks, len(expAcks))
	suite.Require().Len(commitments, len(expCommitments))
	suite.Require().Len(receipts, len(expReceipts))

	suite.Require().Equal(expAcks, acks)
	suite.Require().Equal(expReceipts, receipts)
	suite.Require().Equal(expCommitments, commitments)
}

// TestSetSequence verifies that the keeper correctly sets the sequence counters.
func (suite *KeeperTestSuite) TestSetSequence() {
	path := ibctesting.NewPath(suite.chainA, suite.chainB)
	suite.coordinator.Setup(path)

	ctxA := suite.chainA.GetContext()
	one := uint64(1)

	// initialized channel has next send seq of 1
	seq, found := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetNextSequenceSend(ctxA, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
	suite.True(found)
	suite.Equal(one, seq)

	// initialized channel has next seq recv of 1
	seq, found = suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetNextSequenceRecv(ctxA, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
	suite.True(found)
	suite.Equal(one, seq)

	// initialized channel has next seq ack of
	seq, found = suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetNextSequenceAck(ctxA, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
	suite.True(found)
	suite.Equal(one, seq)

	nextSeqSend, nextSeqRecv, nextSeqAck := uint64(10), uint64(10), uint64(10)
	suite.chainA.App.GetIBCKeeper().ChannelKeeper.SetNextSequenceSend(ctxA, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, nextSeqSend)
	suite.chainA.App.GetIBCKeeper().ChannelKeeper.SetNextSequenceRecv(ctxA, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, nextSeqRecv)
	suite.chainA.App.GetIBCKeeper().ChannelKeeper.SetNextSequenceAck(ctxA, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, nextSeqAck)

	storedNextSeqSend, found := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetNextSequenceSend(ctxA, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
	suite.True(found)
	suite.Equal(nextSeqSend, storedNextSeqSend)

	storedNextSeqRecv, found := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetNextSequenceSend(ctxA, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
	suite.True(found)
	suite.Equal(nextSeqRecv, storedNextSeqRecv)

	storedNextSeqAck, found := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetNextSequenceAck(ctxA, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
	suite.True(found)
	suite.Equal(nextSeqAck, storedNextSeqAck)
}

// TestGetAllPacketCommitmentsAtChannel verifies that the keeper returns all stored packet
// commitments for a specific channel. The test will store consecutive commitments up to the
// value of "seq" and then add non-consecutive up to the value of "maxSeq". A final commitment
// with the value maxSeq + 1 is set on a different channel.
func (suite *KeeperTestSuite) TestGetAllPacketCommitmentsAtChannel() {
	path := ibctesting.NewPath(suite.chainA, suite.chainB)
	suite.coordinator.Setup(path)

	// create second channel
	path1 := ibctesting.NewPath(suite.chainA, suite.chainB)
	path1.SetChannelOrdered()
	path1.EndpointA.ClientID = path.EndpointA.ClientID
	path1.EndpointB.ClientID = path.EndpointB.ClientID
	path1.EndpointA.ConnectionID = path.EndpointA.ConnectionID
	path1.EndpointB.ConnectionID = path.EndpointB.ConnectionID

	suite.coordinator.CreateMockChannels(path1)

	ctxA := suite.chainA.GetContext()
	expectedSeqs := make(map[uint64]bool)
	hash := []byte("commitment")

	seq := uint64(15)
	maxSeq := uint64(25)
	suite.Require().Greater(maxSeq, seq)

	// create consecutive commitments
	for i := uint64(1); i < seq; i++ {
		suite.chainA.App.GetIBCKeeper().ChannelKeeper.SetPacketCommitment(ctxA, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, i, hash)
		expectedSeqs[i] = true
	}

	// add non-consecutive commitments
	for i := seq; i < maxSeq; i += 2 {
		suite.chainA.App.GetIBCKeeper().ChannelKeeper.SetPacketCommitment(ctxA, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, i, hash)
		expectedSeqs[i] = true
	}

	// add sequence on different channel/port
	suite.chainA.App.GetIBCKeeper().ChannelKeeper.SetPacketCommitment(ctxA, path1.EndpointA.ChannelConfig.PortID, path1.EndpointA.ChannelID, maxSeq+1, hash)

	commitments := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetAllPacketCommitmentsAtChannel(ctxA, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)

	suite.Equal(len(expectedSeqs), len(commitments))
	// ensure above for loops occurred
	suite.NotEqual(0, len(commitments))

	// verify that all the packet commitments were stored
	for _, packet := range commitments {
		suite.True(expectedSeqs[packet.Sequence])
		suite.Equal(path.EndpointA.ChannelConfig.PortID, packet.PortId)
		suite.Equal(path.EndpointA.ChannelID, packet.ChannelId)
		suite.Equal(hash, packet.Data)

		// prevent duplicates from passing checks
		expectedSeqs[packet.Sequence] = false
	}
}

// TestSetPacketAcknowledgement verifies that packet acknowledgements are correctly
// set in the keeper.
func (suite *KeeperTestSuite) TestSetPacketAcknowledgement() {
	path := ibctesting.NewPath(suite.chainA, suite.chainB)
	suite.coordinator.Setup(path)

	ctxA := suite.chainA.GetContext()
	seq := uint64(10)

	storedAckHash, found := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetPacketAcknowledgement(ctxA, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, seq)
	suite.Require().False(found)
	suite.Require().Nil(storedAckHash)

	ackHash := []byte("ackhash")
	suite.chainA.App.GetIBCKeeper().ChannelKeeper.SetPacketAcknowledgement(ctxA, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, seq, ackHash)

	storedAckHash, found = suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetPacketAcknowledgement(ctxA, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, seq)
	suite.Require().True(found)
	suite.Require().Equal(ackHash, storedAckHash)
	suite.Require().True(suite.chainA.App.GetIBCKeeper().ChannelKeeper.HasPacketAcknowledgement(ctxA, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, seq))
}

func (suite *KeeperTestSuite) TestSetUpgradeErrorReceipt() {
	path := ibctesting.NewPath(suite.chainA, suite.chainB)
	suite.coordinator.SetupConnections(path)
	suite.coordinator.CreateChannels(path)

	errorReceipt, found := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetUpgradeErrorReceipt(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
	suite.Require().False(found)
	suite.Require().Empty(errorReceipt)

	expError := types.NewUpgradeError(1, fmt.Errorf("testing"))
	suite.chainA.App.GetIBCKeeper().ChannelKeeper.WriteErrorReceipt(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, expError)

	errorReceipt, found = suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetUpgradeErrorReceipt(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
	suite.Require().True(found)
	suite.Require().Equal(expError.GetErrorReceipt(), errorReceipt)
}

// TestDefaultSetParams tests the default params set are what is expected
func (suite *KeeperTestSuite) TestDefaultSetParams() {
	expParams := types.DefaultParams()

	channelKeeper := suite.chainA.App.GetIBCKeeper().ChannelKeeper
	params := channelKeeper.GetParams(suite.chainA.GetContext())

	suite.Require().Equal(expParams, params)
	suite.Require().Equal(expParams.UpgradeTimeout, channelKeeper.GetParams(suite.chainA.GetContext()).UpgradeTimeout)
}

// TestParams tests that Param setting and retrieval works properly
func (suite *KeeperTestSuite) TestParams() {
	testCases := []struct {
		name    string
		input   types.Params
		expPass bool
	}{
		{"success: set default params", types.DefaultParams(), true},
		{"success: zero timeout height", types.NewParams(types.NewTimeout(clienttypes.ZeroHeight(), 10000)), true},
		{"fail: zero timeout timestamp", types.NewParams(types.NewTimeout(clienttypes.NewHeight(1, 1000), 0)), false},
		{"fail: zero timeout", types.NewParams(types.NewTimeout(clienttypes.ZeroHeight(), 0)), false},
	}

	for _, tc := range testCases {
		tc := tc

		suite.Run(tc.name, func() {
			suite.SetupTest() // reset
			ctx := suite.chainA.GetContext()
			err := tc.input.Validate()
			suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.SetParams(ctx, tc.input)
			if tc.expPass {
				suite.Require().NoError(err)
				expected := tc.input
				p := suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetParams(ctx)
				suite.Require().Equal(expected, p)
			} else {
				suite.Require().Error(err)
			}
		})
	}
}

// TestUnsetParams tests that trying to get params that are not set panics.
func (suite *KeeperTestSuite) TestUnsetParams() {
	suite.SetupTest()
	ctx := suite.chainA.GetContext()
	store := ctx.KVStore(suite.chainA.GetSimApp().GetKey(exported.StoreKey))
	store.Delete([]byte(types.ParamsKey))

	suite.Require().Panics(func() {
		suite.chainA.GetSimApp().IBCKeeper.ChannelKeeper.GetParams(ctx)
	})
}

func (suite *KeeperTestSuite) TestPruneAcknowledgements() {
	var (
		path          *ibctesting.Path
		limit         uint64
		upgradeFields types.UpgradeFields

		// postPruneExpState is a helper function to verify the expected state after pruning. Argument expLeft
		// denotes the expected amount of packet acks and receipts left after pruning. Argument expSequenceStart
		// denotes the expected value of PruneSequenceStart.
		postPruneExpState = func(expAcksLen, expReceiptsLen, expPruningSequenceStart uint64) {
			acks := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetAllPacketAcks(suite.chainA.GetContext())
			suite.Require().Len(acks, int(expAcksLen))

			receipts := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetAllPacketReceipts(suite.chainA.GetContext())
			suite.Require().Len(receipts, int(expReceiptsLen))

			start, found := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetPruningSequenceStart(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
			suite.Require().True(found)
			suite.Require().Equal(start, expPruningSequenceStart)
		}
	)

	testCases := []struct {
		name     string
		pre      func()
		malleate func()
		post     func(pruned, left uint64)
		expError error
	}{
		{
			"success: no packets sent, no stale packet state pruned",
			func() {},
			func() {},
			func(pruned, left uint64) {
				// Assert that PruneSequenceStart and PruneSequenceEnd are both set to 1.
				start, found := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetPruningSequenceStart(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found)
				end, found := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetRecvStartSequence(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found)

				suite.Require().Equal(uint64(1), start)
				suite.Require().Equal(uint64(1), end)

				// We expect 0 to be pruned and 0 left.
				suite.Require().Equal(uint64(0), pruned)
				suite.Require().Equal(uint64(0), left)
			},
			nil,
		},
		{
			"success: stale packet state pruned up to limit",
			func() {
				// Send 10 packets from B -> A, creating 10 packet receipts and 10 packet acks on A.
				suite.sendMockPackets(path, 10, true)
			},
			func() {},
			func(pruned, left uint64) {
				sequenceEnd, found := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetRecvStartSequence(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found)

				// We expect nothing to be left and sequenceStart == sequenceEnd.
				postPruneExpState(0, 0, sequenceEnd)

				// We expect 10 to be pruned and 0 left.
				suite.Require().Equal(uint64(10), pruned)
				suite.Require().Equal(uint64(0), left)
			},
			nil,
		},
		{
			"success: stale packet state partially pruned",
			func() {
				// Send 10 packets from B -> A, creating 10 packet receipts and 10 packet acks on A.
				suite.sendMockPackets(path, 10, true)
			},
			func() {
				// Prune only 6 packet acks.
				limit = 6
			},
			func(pruned, left uint64) {
				// We expect 4 to be left and sequenceStart == 7.
				postPruneExpState(4, 4, 7)

				// We expect 6 to be pruned and 4 left.
				suite.Require().Equal(uint64(6), pruned)
				suite.Require().Equal(uint64(4), left)
			},
			nil,
		},
		{
			"success: stale packet state with a higher limit",
			func() {
				// Send 10 packets from B -> A, creating 10 packet receipts and 10 packet acks on A.
				suite.sendMockPackets(path, 10, true)
			},
			func() {
				// Prune 13 packets acks > 10 packets sent.
				limit = 13
			},
			func(pruned, left uint64) {
				// We expect 0 to be left and sequenceStart == 11.
				postPruneExpState(0, 0, 11)

				// We expect 10 to be pruned and 0 left.
				suite.Require().Equal(uint64(10), pruned)
				suite.Require().Equal(uint64(0), left)
			},
			nil,
		},
		{
			"success: stale packet state pruned, two upgrades",
			func() {
				// Send 10 packets from B -> A, creating 10 packet receipts and 10 packet acks on A.
				// This is _before_ the first upgrade.
				suite.sendMockPackets(path, 10, true)
			},
			func() {
				// Previous upgrade is complete, send additional packets and do yet another upgrade.
				// This is _after_ the first upgrade.
				suite.sendMockPackets(path, 5, true)

				// Do another upgrade.
				upgradeFields = types.UpgradeFields{Version: fmt.Sprintf("%s-v3", ibcmock.Version)}
				suite.UpgradeChannel(path, upgradeFields)

				// set limit to 15, get them all in one go.
				limit = 15
			},
			func(pruned, left uint64) {
				sequenceEnd, found := suite.chainA.App.GetIBCKeeper().ChannelKeeper.GetRecvStartSequence(suite.chainA.GetContext(), path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID)
				suite.Require().True(found)

				// We expect nothing to be left and sequenceStart == sequenceEnd.
				postPruneExpState(0, 0, sequenceEnd)

				// We expect 15 to be pruned and 0 left.
				suite.Require().Equal(uint64(15), pruned)
				suite.Require().Equal(uint64(0), left)
			},
			nil,
		},
		{
			"success: stale packet state partially pruned, upgrade, prune again",
			func() {
				// Send 10 packets from B -> A, creating 10 packet receipts and 10 packet acks on A.
				// This is _before_ the first upgrade.
				suite.sendMockPackets(path, 10, true)
			},
			func() {
				// Prune 5 on A.
				pruned, left, err := suite.chainA.App.GetIBCKeeper().ChannelKeeper.PruneAcknowledgements(
					suite.chainA.GetContext(),
					path.EndpointA.ChannelConfig.PortID,
					path.EndpointA.ChannelID,
					4, // limit == 4
				)
				suite.Require().NoError(err)

				// We expect 4 to be pruned and 6 left.
				suite.Require().Equal(uint64(4), pruned)
				suite.Require().Equal(uint64(6), left)

				// Check state post-prune
				postPruneExpState(6, 6, 5)

				// Previous upgrade is complete, send additional packets and do yet another upgrade.
				// This is _after_ the first upgrade.
				suite.sendMockPackets(path, 10, true)

				// Do another upgrade.
				upgradeFields = types.UpgradeFields{Version: fmt.Sprintf("%s-v3", ibcmock.Version)}
				suite.UpgradeChannel(path, upgradeFields)

				// A total of 16 stale acks/receipts exist on A. Prune 10 of them (default in test).
			},
			func(pruned, left uint64) {
				// Expected state should be 6 acks/receipts left, sequenceStart == 15.
				postPruneExpState(6, 6, 15)

				// We expect 10 to be pruned and 6 left.
				suite.Require().Equal(uint64(10), pruned)
				suite.Require().Equal(uint64(6), left)
			},
			nil,
		},
		{
			"success: unordered -> ordered -> unordered, acksLen != receiptsLen after packet sends",
			func() {
				// Send 5 packets from B -> A, creating 5 packet receipts and 5 packet acks on A.
				// This is _before_ the first upgrade.
				suite.sendMockPackets(path, 5, true)

				// Set Order for upgrade to Ordered.
				upgradeFields = types.UpgradeFields{Version: fmt.Sprintf("%s-v2", ibcmock.Version), Ordering: types.ORDERED}
			},
			func() {
				// Previous upgrade is complete, send additional packets now on ordered channel (only acks!)
				suite.sendMockPackets(path, 10, true)

				// Do another upgrade (go back to Unordered)
				upgradeFields = types.UpgradeFields{Version: fmt.Sprintf("%s-v3", ibcmock.Version), Ordering: types.UNORDERED}
				suite.UpgradeChannel(path, upgradeFields)
			},
			func(pruned, left uint64) {
				// After pruning 10 sequences we should be left with 5 acks and zero receipts.
				postPruneExpState(5, 0, 11)

				// We expect 10 to be pruned and 5 left.
				suite.Require().Equal(uint64(10), pruned)
				suite.Require().Equal(uint64(5), left)
			},
			nil,
		},
		{
			"success: packets sent before upgrade are pruned, after upgrade are not",
			func() {
				// Send 5 packets from B -> A, creating 5 packet receipts and 5 packet acks on A.
				suite.sendMockPackets(path, 5, true)
			},
			func() {},
			func(pruned, left uint64) {
				// We expect 5 to be pruned and 0 left.
				suite.Require().Equal(uint64(5), pruned)
				suite.Require().Equal(uint64(0), left)

				// channel upgraded, send additional packets and try and prune.
				suite.sendMockPackets(path, 12, true)

				// attempt to prune 5.
				pruned, left, err := suite.chainA.App.GetIBCKeeper().ChannelKeeper.PruneAcknowledgements(
					suite.chainA.GetContext(),
					path.EndpointA.ChannelConfig.PortID,
					path.EndpointA.ChannelID,
					5,
				)
				suite.Require().NoError(err)
				// We expect 0 to be pruned and 0 left.
				suite.Require().Equal(uint64(0), pruned)
				suite.Require().Equal(uint64(0), left)

				// we _do not_ expect error, simply a fast return
				postPruneExpState(12, 12, 6)
			},
			nil,
		},
		{
			"success: packets sent with 2 middle sequences that don't have an ack stored",
			func() {
				// Send 12 packets from B -> A with 2 packets that timeout
				suite.sendMockPackets(path, 5, true)
				suite.sendMockPackets(path, 2, false)
				suite.sendMockPackets(path, 5, true)
			},
			func() {
				limit = 7
			},
			func(pruned, left uint64) {
				// After pruning 7 sequences we should be left with 5 acks and 5 receipts,
				// because the 2 packets that timeout are still counted as pruned, even though
				// there was nothing to prune since both packets timed out
				postPruneExpState(5, 5, 8)

				// We expect 7 to be pruned and 5 left.
				suite.Require().Equal(uint64(7), pruned)
				suite.Require().Equal(uint64(5), left)
			},
			nil,
		},
		{
			"failure: packet sequence start not set",
			func() {},
			func() {
				path.EndpointA.ChannelConfig.PortID = "portidone"
			},
			func(_, _ uint64) {},
			types.ErrPruningSequenceStartNotFound,
		},
		{
			"failure: packet sequence end not set",
			func() {},
			func() {
				store := suite.chainA.GetContext().KVStore(suite.chainA.GetSimApp().GetKey(exported.StoreKey))
				store.Delete(host.RecvStartSequenceKey(path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID))
			},
			func(_, _ uint64) {},
			types.ErrRecvStartSequenceNotFound,
		},
	}

	for _, tc := range testCases {
		tc := tc
		suite.Run(tc.name, func() {
			suite.SetupTest() // reset

			path = ibctesting.NewPath(suite.chainA, suite.chainB)
			suite.coordinator.Setup(path)

			// Defaults will be filled in for rest.
			upgradeFields = types.UpgradeFields{Version: ibcmock.UpgradeVersion}
			limit = 10

			// perform pre upgrade ops.
			tc.pre()

			suite.UpgradeChannel(path, upgradeFields)

			tc.malleate()

			pruned, left, err := suite.chainA.App.GetIBCKeeper().ChannelKeeper.PruneAcknowledgements(
				suite.chainA.GetContext(),
				path.EndpointA.ChannelConfig.PortID,
				path.EndpointA.ChannelID,
				limit,
			)

			suite.Require().ErrorIs(err, tc.expError)

			// check on post state.
			tc.post(pruned, left)
		})
	}
}

// UpgradeChannel performs a channel upgrade given a specific set of upgrade fields.
// Question(jim): setup.coordinator.UpgradeChannel() wen?
func (suite *KeeperTestSuite) UpgradeChannel(path *ibctesting.Path, upgradeFields types.UpgradeFields) {
	// configure the channel upgrade version on testing endpoints
	path.EndpointA.ChannelConfig.ProposedUpgrade.Fields = upgradeFields
	path.EndpointB.ChannelConfig.ProposedUpgrade.Fields = upgradeFields

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
}

// sendMockPacket sends a packet from source to dest and acknowledges it on the source (completing the packet lifecycle)
// if acknowledge is true. If acknowledge is false, then the packet will be sent, but timed out.
// Question(jim): find a nicer home for this?
func (suite *KeeperTestSuite) sendMockPackets(path *ibctesting.Path, numPackets int, acknowledge bool) {
	for i := 0; i < numPackets; i++ {

		timeoutHeight := clienttypes.NewHeight(1, 1000)
		timeoutTimestamp := disabledTimeoutTimestamp
		if !acknowledge {
			timeoutTimestamp = uint64(suite.chainA.GetContext().BlockTime().UnixNano())
		}

		sequence, err := path.EndpointB.SendPacket(timeoutHeight, timeoutTimestamp, ibctesting.MockPacketData)
		suite.Require().NoError(err)

		packet := types.NewPacket(ibctesting.MockPacketData, sequence, path.EndpointB.ChannelConfig.PortID, path.EndpointB.ChannelID, path.EndpointA.ChannelConfig.PortID, path.EndpointA.ChannelID, timeoutHeight, timeoutTimestamp)

		if acknowledge {
			err = path.RelayPacket(packet)
			suite.Require().NoError(err)
		} else {
			err = path.EndpointB.UpdateClient()
			suite.Require().NoError(err)

			err = path.EndpointB.TimeoutPacket(packet)
			suite.Require().NoError(err)
		}
	}
}
