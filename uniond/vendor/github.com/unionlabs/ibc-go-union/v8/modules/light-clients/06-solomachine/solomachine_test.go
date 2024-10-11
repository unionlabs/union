package solomachine_test

import (
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	testifysuite "github.com/stretchr/testify/suite"

	storetypes "cosmossdk.io/store/types"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"

	transfertypes "github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	solomachine "github.com/cosmos/ibc-go/v8/modules/light-clients/06-solomachine"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
	"github.com/cosmos/ibc-go/v8/testing/mock"
)

var channelIDSolomachine = "channel-on-solomachine" // channelID generated on solo machine side

type SoloMachineTestSuite struct {
	testifysuite.Suite

	solomachine      *ibctesting.Solomachine // singlesig public key
	solomachineMulti *ibctesting.Solomachine // multisig public key
	coordinator      *ibctesting.Coordinator

	// testing chain used for convenience and readability
	chainA *ibctesting.TestChain
	chainB *ibctesting.TestChain

	store storetypes.KVStore
}

func (suite *SoloMachineTestSuite) SetupTest() {
	suite.coordinator = ibctesting.NewCoordinator(suite.T(), 2)
	suite.chainA = suite.coordinator.GetChain(ibctesting.GetChainID(1))
	suite.chainB = suite.coordinator.GetChain(ibctesting.GetChainID(2))

	suite.solomachine = ibctesting.NewSolomachine(suite.T(), suite.chainA.Codec, "solomachinesingle", "testing", 1)
	suite.solomachineMulti = ibctesting.NewSolomachine(suite.T(), suite.chainA.Codec, "solomachinemulti", "testing", 4)

	suite.store = suite.chainA.App.GetIBCKeeper().ClientKeeper.ClientStore(suite.chainA.GetContext(), exported.Solomachine)
}

func TestSoloMachineTestSuite(t *testing.T) {
	testifysuite.Run(t, new(SoloMachineTestSuite))
}

func (suite *SoloMachineTestSuite) SetupSolomachine() string {
	clientID := suite.solomachine.CreateClient(suite.chainA)

	connectionID := suite.solomachine.ConnOpenInit(suite.chainA, clientID)

	// open try is not necessary as the solo machine implementation is mocked

	suite.solomachine.ConnOpenAck(suite.chainA, clientID, connectionID)

	// open confirm is not necessary as the solo machine implementation is mocked

	channelID := suite.solomachine.ChanOpenInit(suite.chainA, connectionID)

	// open try is not necessary as the solo machine implementation is mocked

	suite.solomachine.ChanOpenAck(suite.chainA, channelID)

	// open confirm is not necessary as the solo machine implementation is mocked

	return channelID
}

func (suite *SoloMachineTestSuite) TestRecvPacket() {
	channelID := suite.SetupSolomachine()
	packet := channeltypes.NewPacket(
		mock.MockPacketData,
		1,
		transfertypes.PortID,
		channelIDSolomachine,
		transfertypes.PortID,
		channelID,
		clienttypes.ZeroHeight(),
		uint64(suite.chainA.GetContext().BlockTime().Add(time.Hour).UnixNano()),
	)

	// send packet is not necessary as the solo machine implementation is mocked

	suite.solomachine.RecvPacket(suite.chainA, packet)

	// close init is not necessary as the solomachine implementation is mocked

	suite.solomachine.ChanCloseConfirm(suite.chainA, transfertypes.PortID, channelID)
}

func (suite *SoloMachineTestSuite) TestAcknowledgePacket() {
	channelID := suite.SetupSolomachine()

	packet := suite.solomachine.SendTransfer(suite.chainA, transfertypes.PortID, channelID)

	// recv packet is not necessary as the solo machine implementation is mocked

	suite.solomachine.AcknowledgePacket(suite.chainA, packet)

	// close init is not necessary as the solomachine implementation is mocked

	suite.solomachine.ChanCloseConfirm(suite.chainA, transfertypes.PortID, channelID)
}

func (suite *SoloMachineTestSuite) TestTimeout() {
	channelID := suite.SetupSolomachine()
	packet := suite.solomachine.SendTransfer(suite.chainA, transfertypes.PortID, channelID, func(msg *transfertypes.MsgTransfer) {
		msg.TimeoutTimestamp = suite.solomachine.Time + 1
	})

	// simulate solomachine time increment
	suite.solomachine.Time++

	suite.solomachine.UpdateClient(suite.chainA, ibctesting.DefaultSolomachineClientID)

	suite.solomachine.TimeoutPacket(suite.chainA, packet)

	suite.solomachine.ChanCloseConfirm(suite.chainA, transfertypes.PortID, channelID)
}

func (suite *SoloMachineTestSuite) TestTimeoutOnClose() {
	channelID := suite.SetupSolomachine()

	packet := suite.solomachine.SendTransfer(suite.chainA, transfertypes.PortID, channelID)

	suite.solomachine.TimeoutPacketOnClose(suite.chainA, packet, channelID)
}

func (suite *SoloMachineTestSuite) GetSequenceFromStore() uint64 {
	bz := suite.store.Get(host.ClientStateKey())
	suite.Require().NotNil(bz)

	var clientState exported.ClientState
	err := suite.chainA.Codec.UnmarshalInterface(bz, &clientState)
	suite.Require().NoError(err)
	return clientState.GetLatestHeight().GetRevisionHeight()
}

func (suite *SoloMachineTestSuite) GetInvalidProof() []byte {
	invalidProof, err := suite.chainA.Codec.Marshal(&solomachine.TimestampedSignatureData{Timestamp: suite.solomachine.Time})
	suite.Require().NoError(err)

	return invalidProof
}

func TestUnpackInterfaces_Header(t *testing.T) {
	registry := testdata.NewTestInterfaceRegistry()
	cryptocodec.RegisterInterfaces(registry)

	pk := secp256k1.GenPrivKey().PubKey()
	protoAny, err := codectypes.NewAnyWithValue(pk)
	require.NoError(t, err)

	header := solomachine.Header{
		NewPublicKey: protoAny,
	}
	bz, err := header.Marshal()
	require.NoError(t, err)

	var header2 solomachine.Header
	err = header2.Unmarshal(bz)
	require.NoError(t, err)

	err = codectypes.UnpackInterfaces(header2, registry)
	require.NoError(t, err)

	require.Equal(t, pk, header2.NewPublicKey.GetCachedValue())
}

func TestUnpackInterfaces_HeaderData(t *testing.T) {
	registry := testdata.NewTestInterfaceRegistry()
	cryptocodec.RegisterInterfaces(registry)

	pk := secp256k1.GenPrivKey().PubKey()
	protoAny, err := codectypes.NewAnyWithValue(pk)
	require.NoError(t, err)

	hd := solomachine.HeaderData{
		NewPubKey: protoAny,
	}
	bz, err := hd.Marshal()
	require.NoError(t, err)

	var hd2 solomachine.HeaderData
	err = hd2.Unmarshal(bz)
	require.NoError(t, err)

	err = codectypes.UnpackInterfaces(hd2, registry)
	require.NoError(t, err)

	require.Equal(t, pk, hd2.NewPubKey.GetCachedValue())
}
