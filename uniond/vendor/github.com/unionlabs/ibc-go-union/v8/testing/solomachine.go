package ibctesting

import (
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	sdkmath "cosmossdk.io/math"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	kmultisig "github.com/cosmos/cosmos-sdk/crypto/keys/multisig"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/crypto/types/multisig"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"

	transfertypes "github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	solomachine "github.com/cosmos/ibc-go/v8/modules/light-clients/06-solomachine"
	ibctm "github.com/cosmos/ibc-go/v8/modules/light-clients/07-tendermint"
)

var (
	clientIDSolomachine     = "client-on-solomachine"     // clientID generated on solo machine side
	connectionIDSolomachine = "connection-on-solomachine" // connectionID generated on solo machine side
	channelIDSolomachine    = "channel-on-solomachine"    // channelID generated on solo machine side
)

// DefaultSolomachineClientID is the default solo machine client id used for testing
var DefaultSolomachineClientID = "06-solomachine-0"

// Solomachine is a testing helper used to simulate a counterparty
// solo machine client.
type Solomachine struct {
	t *testing.T

	cdc         codec.BinaryCodec
	ClientID    string
	PrivateKeys []cryptotypes.PrivKey // keys used for signing
	PublicKeys  []cryptotypes.PubKey  // keys used for generating solo machine pub key
	PublicKey   cryptotypes.PubKey    // key used for verification
	Sequence    uint64
	Time        uint64
	Diversifier string
}

// NewSolomachine returns a new solomachine instance with an `nKeys` amount of
// generated private/public key pairs and a sequence starting at 1. If nKeys
// is greater than 1 then a multisig public key is used.
func NewSolomachine(t *testing.T, cdc codec.BinaryCodec, clientID, diversifier string, nKeys uint64) *Solomachine {
	t.Helper()
	privKeys, pubKeys, pk := GenerateKeys(t, nKeys)

	return &Solomachine{
		t:           t,
		cdc:         cdc,
		ClientID:    clientID,
		PrivateKeys: privKeys,
		PublicKeys:  pubKeys,
		PublicKey:   pk,
		Sequence:    1,
		Time:        10,
		Diversifier: diversifier,
	}
}

// GenerateKeys generates a new set of secp256k1 private keys and public keys.
// If the number of keys is greater than one then the public key returned represents
// a multisig public key. The private keys are used for signing, the public
// keys are used for generating the public key and the public key is used for
// solo machine verification. The usage of secp256k1 is entirely arbitrary.
// The key type can be swapped for any key type supported by the PublicKey
// interface, if needed. The same is true for the amino based Multisignature
// public key.
func GenerateKeys(t *testing.T, n uint64) ([]cryptotypes.PrivKey, []cryptotypes.PubKey, cryptotypes.PubKey) {
	t.Helper()
	require.NotEqual(t, uint64(0), n, "generation of zero keys is not allowed")

	privKeys := make([]cryptotypes.PrivKey, n)
	pubKeys := make([]cryptotypes.PubKey, n)
	for i := uint64(0); i < n; i++ {
		privKeys[i] = secp256k1.GenPrivKey()
		pubKeys[i] = privKeys[i].PubKey()
	}

	var pk cryptotypes.PubKey
	if len(privKeys) > 1 {
		// generate multi sig pk
		pk = kmultisig.NewLegacyAminoPubKey(int(n), pubKeys)
	} else {
		pk = privKeys[0].PubKey()
	}

	return privKeys, pubKeys, pk
}

// ClientState returns a new solo machine ClientState instance.
func (solo *Solomachine) ClientState() *solomachine.ClientState {
	return solomachine.NewClientState(solo.Sequence, solo.ConsensusState())
}

// ConsensusState returns a new solo machine ConsensusState instance
func (solo *Solomachine) ConsensusState() *solomachine.ConsensusState {
	publicKey, err := codectypes.NewAnyWithValue(solo.PublicKey)
	require.NoError(solo.t, err)

	return &solomachine.ConsensusState{
		PublicKey:   publicKey,
		Diversifier: solo.Diversifier,
		Timestamp:   solo.Time,
	}
}

// GetHeight returns an exported.Height with Sequence as RevisionHeight
func (solo *Solomachine) GetHeight() exported.Height {
	return clienttypes.NewHeight(0, solo.Sequence)
}

// CreateClient creates an on-chain client on the provided chain.
func (solo *Solomachine) CreateClient(chain *TestChain) string {
	msgCreateClient, err := clienttypes.NewMsgCreateClient(solo.ClientState(), solo.ConsensusState(), chain.SenderAccount.GetAddress().String())
	require.NoError(solo.t, err)

	res, err := chain.SendMsgs(msgCreateClient)
	require.NoError(solo.t, err)
	require.NotNil(solo.t, res)

	clientID, err := ParseClientIDFromEvents(res.Events)
	require.NoError(solo.t, err)

	return clientID
}

// UpdateClient sends a MsgUpdateClient to the provided chain and updates the given clientID.
func (solo *Solomachine) UpdateClient(chain *TestChain, clientID string) {
	smHeader := solo.CreateHeader(solo.Diversifier)
	msgUpdateClient, err := clienttypes.NewMsgUpdateClient(clientID, smHeader, chain.SenderAccount.GetAddress().String())
	require.NoError(solo.t, err)

	res, err := chain.SendMsgs(msgUpdateClient)
	require.NoError(solo.t, err)
	require.NotNil(solo.t, res)
}

// CreateHeader generates a new private/public key pair and creates the
// necessary signature to construct a valid solo machine header.
// A new diversifier will be used as well
func (solo *Solomachine) CreateHeader(newDiversifier string) *solomachine.Header {
	// generate new private keys and signature for header
	newPrivKeys, newPubKeys, newPubKey := GenerateKeys(solo.t, uint64(len(solo.PrivateKeys)))

	publicKey, err := codectypes.NewAnyWithValue(newPubKey)
	require.NoError(solo.t, err)

	data := &solomachine.HeaderData{
		NewPubKey:      publicKey,
		NewDiversifier: newDiversifier,
	}

	dataBz, err := solo.cdc.Marshal(data)
	require.NoError(solo.t, err)

	signBytes := &solomachine.SignBytes{
		Sequence:    solo.Sequence,
		Timestamp:   solo.Time,
		Diversifier: solo.Diversifier,
		Path:        []byte(solomachine.SentinelHeaderPath),
		Data:        dataBz,
	}

	bz, err := solo.cdc.Marshal(signBytes)
	require.NoError(solo.t, err)

	sig := solo.GenerateSignature(bz)

	header := &solomachine.Header{
		Timestamp:      solo.Time,
		Signature:      sig,
		NewPublicKey:   publicKey,
		NewDiversifier: newDiversifier,
	}

	// assumes successful header update
	solo.Sequence++
	solo.Time++
	solo.PrivateKeys = newPrivKeys
	solo.PublicKeys = newPubKeys
	solo.PublicKey = newPubKey
	solo.Diversifier = newDiversifier

	return header
}

// CreateMisbehaviour constructs testing misbehaviour for the solo machine client
// by signing over two different data bytes at the same sequence.
func (solo *Solomachine) CreateMisbehaviour() *solomachine.Misbehaviour {
	merklePath := commitmenttypes.NewMerklePath(host.FullClientStatePath("counterparty"))
	path, err := solo.cdc.Marshal(&merklePath)
	require.NoError(solo.t, err)

	data, err := solo.cdc.Marshal(solo.ClientState())
	require.NoError(solo.t, err)

	signBytes := &solomachine.SignBytes{
		Sequence:    solo.Sequence,
		Timestamp:   solo.Time,
		Diversifier: solo.Diversifier,
		Path:        path,
		Data:        data,
	}

	bz, err := solo.cdc.Marshal(signBytes)
	require.NoError(solo.t, err)

	sig := solo.GenerateSignature(bz)
	signatureOne := solomachine.SignatureAndData{
		Signature: sig,
		Path:      path,
		Data:      data,
		Timestamp: solo.Time,
	}

	// misbehaviour signaturess can have different timestamps
	solo.Time++

	merklePath = commitmenttypes.NewMerklePath(host.FullConsensusStatePath("counterparty", clienttypes.NewHeight(0, 1)))
	path, err = solo.cdc.Marshal(&merklePath)
	require.NoError(solo.t, err)

	data, err = solo.cdc.Marshal(solo.ConsensusState())
	require.NoError(solo.t, err)

	signBytes = &solomachine.SignBytes{
		Sequence:    solo.Sequence,
		Timestamp:   solo.Time,
		Diversifier: solo.Diversifier,
		Path:        path,
		Data:        data,
	}

	bz, err = solo.cdc.Marshal(signBytes)
	require.NoError(solo.t, err)

	sig = solo.GenerateSignature(bz)
	signatureTwo := solomachine.SignatureAndData{
		Signature: sig,
		Path:      path,
		Data:      data,
		Timestamp: solo.Time,
	}

	return &solomachine.Misbehaviour{
		Sequence:     solo.Sequence,
		SignatureOne: &signatureOne,
		SignatureTwo: &signatureTwo,
	}
}

// ConnOpenInit initializes a connection on the provided chain given a solo machine clientID.
func (solo *Solomachine) ConnOpenInit(chain *TestChain, clientID string) string {
	msgConnOpenInit := connectiontypes.NewMsgConnectionOpenInit(
		clientID,
		clientIDSolomachine, // clientID generated on solo machine side
		chain.GetPrefix(), DefaultOpenInitVersion, DefaultDelayPeriod,
		chain.SenderAccount.GetAddress().String(),
	)

	res, err := chain.SendMsgs(msgConnOpenInit)
	require.NoError(solo.t, err)
	require.NotNil(solo.t, res)

	connectionID, err := ParseConnectionIDFromEvents(res.Events)
	require.NoError(solo.t, err)

	return connectionID
}

// ConnOpenAck performs the connection open ack handshake step on the tendermint chain for the associated
// solo machine client.
func (solo *Solomachine) ConnOpenAck(chain *TestChain, clientID, connectionID string) {
	tryProof := solo.GenerateConnOpenTryProof(clientID, connectionID)

	clientState := ibctm.NewClientState(chain.ChainID, DefaultTrustLevel, TrustingPeriod, UnbondingPeriod, MaxClockDrift, chain.LastHeader.GetHeight().(clienttypes.Height), commitmenttypes.GetSDKSpecs(), UpgradePath)

	msgConnOpenAck := connectiontypes.NewMsgConnectionOpenAck(
		connectionID, connectionIDSolomachine, clientState,
		tryProof, nil, nil,
		clienttypes.ZeroHeight(), clientState.GetLatestHeight().(clienttypes.Height),
		ConnectionVersion,
		chain.SenderAccount.GetAddress().String(),
	)

	res, err := chain.SendMsgs(msgConnOpenAck)
	require.NoError(solo.t, err)
	require.NotNil(solo.t, res)
}

// ChanOpenInit initializes a channel on the provided chain given a solo machine connectionID.
func (solo *Solomachine) ChanOpenInit(chain *TestChain, connectionID string) string {
	msgChanOpenInit := channeltypes.NewMsgChannelOpenInit(
		transfertypes.PortID,
		transfertypes.Version,
		channeltypes.UNORDERED,
		[]string{connectionID},
		transfertypes.PortID,
		chain.SenderAccount.GetAddress().String(),
	)

	res, err := chain.SendMsgs(msgChanOpenInit)
	require.NoError(solo.t, err)
	require.NotNil(solo.t, res)

	channelID, err := ParseChannelIDFromEvents(res.Events)
	require.NoError(solo.t, err)

	return channelID
}

// ChanOpenAck performs the channel open ack handshake step on the tendermint chain for the associated
// solo machine client.
func (solo *Solomachine) ChanOpenAck(chain *TestChain, channelID string) {
	tryProof := solo.GenerateChanOpenTryProof(transfertypes.PortID, transfertypes.Version, channelID)
	msgChanOpenAck := channeltypes.NewMsgChannelOpenAck(
		transfertypes.PortID,
		channelID,
		channelIDSolomachine,
		transfertypes.Version,
		tryProof,
		clienttypes.ZeroHeight(),
		chain.SenderAccount.GetAddress().String(),
	)

	res, err := chain.SendMsgs(msgChanOpenAck)
	require.NoError(solo.t, err)
	require.NotNil(solo.t, res)
}

// ChanCloseConfirm performs the channel close confirm handshake step on the tendermint chain for the associated
// solo machine client.
func (solo *Solomachine) ChanCloseConfirm(chain *TestChain, portID, channelID string) {
	initProof := solo.GenerateChanClosedProof(portID, transfertypes.Version, channelID)
	msgChanCloseConfirm := channeltypes.NewMsgChannelCloseConfirm(
		portID,
		channelID,
		initProof,
		clienttypes.ZeroHeight(),
		chain.SenderAccount.GetAddress().String(),
	)

	res, err := chain.SendMsgs(msgChanCloseConfirm)
	require.NoError(solo.t, err)
	require.NotNil(solo.t, res)
}

// SendTransfer constructs a MsgTransfer and sends the message to the given chain. Any number of optional
// functions can be provided which will modify the MsgTransfer before SendMsgs is called.
func (solo *Solomachine) SendTransfer(chain *TestChain, portID, channelID string, fns ...func(*transfertypes.MsgTransfer)) channeltypes.Packet {
	msgTransfer := transfertypes.MsgTransfer{
		SourcePort:       portID,
		SourceChannel:    channelID,
		Token:            sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100)),
		Sender:           chain.SenderAccount.GetAddress().String(),
		Receiver:         chain.SenderAccount.GetAddress().String(),
		TimeoutHeight:    clienttypes.ZeroHeight(),
		TimeoutTimestamp: uint64(chain.GetContext().BlockTime().Add(time.Hour).UnixNano()),
	}

	for _, fn := range fns {
		fn(&msgTransfer)
	}

	res, err := chain.SendMsgs(&msgTransfer)
	require.NoError(solo.t, err)

	packet, err := ParsePacketFromEvents(res.Events)
	require.NoError(solo.t, err)

	return packet
}

// RecvPacket creates a commitment proof and broadcasts a new MsgRecvPacket.
func (solo *Solomachine) RecvPacket(chain *TestChain, packet channeltypes.Packet) {
	proofCommitment := solo.GenerateCommitmentProof(packet)
	msgRecvPacket := channeltypes.NewMsgRecvPacket(
		packet,
		proofCommitment,
		clienttypes.ZeroHeight(),
		chain.SenderAccount.GetAddress().String(),
	)

	res, err := chain.SendMsgs(msgRecvPacket)
	require.NoError(solo.t, err)
	require.NotNil(solo.t, res)
}

// AcknowledgePacket creates an acknowledgement proof and broadcasts a MsgAcknowledgement.
func (solo *Solomachine) AcknowledgePacket(chain *TestChain, packet channeltypes.Packet) {
	ackProof := solo.GenerateAcknowledgementProof(packet)
	transferAck := channeltypes.NewResultAcknowledgement([]byte{byte(1)}).Acknowledgement()
	msgAcknowledgement := channeltypes.NewMsgAcknowledgement(
		packet, transferAck,
		ackProof,
		clienttypes.ZeroHeight(),
		chain.SenderAccount.GetAddress().String(),
	)

	res, err := chain.SendMsgs(msgAcknowledgement)
	require.NoError(solo.t, err)
	require.NotNil(solo.t, res)
}

// TimeoutPacket creates a unreceived packet proof and broadcasts a MsgTimeout.
func (solo *Solomachine) TimeoutPacket(chain *TestChain, packet channeltypes.Packet) {
	unreceivedProof := solo.GenerateReceiptAbsenceProof(packet)
	msgTimeout := channeltypes.NewMsgTimeout(
		packet,
		1, // nextSequenceRecv is unused for UNORDERED channels
		unreceivedProof,
		clienttypes.ZeroHeight(),
		chain.SenderAccount.GetAddress().String(),
	)

	res, err := chain.SendMsgs(msgTimeout)
	require.NoError(solo.t, err)
	require.NotNil(solo.t, res)
}

// TimeoutPacket creates a channel closed and unreceived packet proof and broadcasts a MsgTimeoutOnClose.
func (solo *Solomachine) TimeoutPacketOnClose(chain *TestChain, packet channeltypes.Packet, channelID string) {
	closedProof := solo.GenerateChanClosedProof(transfertypes.PortID, transfertypes.Version, channelID)
	unreceivedProof := solo.GenerateReceiptAbsenceProof(packet)
	msgTimeout := channeltypes.NewMsgTimeoutOnClose(
		packet,
		1, // nextSequenceRecv is unused for UNORDERED channels
		unreceivedProof,
		closedProof,
		clienttypes.ZeroHeight(),
		chain.SenderAccount.GetAddress().String(),
	)

	res, err := chain.SendMsgs(msgTimeout)
	require.NoError(solo.t, err)
	require.NotNil(solo.t, res)
}

// GenerateSignature uses the stored private keys to generate a signature
// over the sign bytes with each key. If the amount of keys is greater than
// 1 then a multisig data type is returned.
func (solo *Solomachine) GenerateSignature(signBytes []byte) []byte {
	sigs := make([]signing.SignatureData, len(solo.PrivateKeys))
	for i, key := range solo.PrivateKeys {
		sig, err := key.Sign(signBytes)
		require.NoError(solo.t, err)

		sigs[i] = &signing.SingleSignatureData{
			Signature: sig,
		}
	}

	var sigData signing.SignatureData
	if len(sigs) == 1 {
		// single public key
		sigData = sigs[0]
	} else {
		// generate multi signature data
		multiSigData := multisig.NewMultisig(len(sigs))
		for i, sig := range sigs {
			multisig.AddSignature(multiSigData, sig, i)
		}

		sigData = multiSigData
	}

	protoSigData := signing.SignatureDataToProto(sigData)
	bz, err := solo.cdc.Marshal(protoSigData)
	require.NoError(solo.t, err)

	return bz
}

// GenerateProof takes in solo machine sign bytes, generates a signature and marshals it as a proof.
// The solo machine sequence is incremented.
func (solo *Solomachine) GenerateProof(signBytes *solomachine.SignBytes) []byte {
	bz, err := solo.cdc.Marshal(signBytes)
	require.NoError(solo.t, err)

	sig := solo.GenerateSignature(bz)
	signatureDoc := &solomachine.TimestampedSignatureData{
		SignatureData: sig,
		Timestamp:     solo.Time,
	}
	proof, err := solo.cdc.Marshal(signatureDoc)
	require.NoError(solo.t, err)

	solo.Sequence++

	return proof
}

// GenerateClientStateProof generates the proof of the client state required for the connection open try and ack handshake steps.
// The client state should be the self client states of the tendermint chain.
func (solo *Solomachine) GenerateClientStateProof(clientState exported.ClientState) []byte {
	data, err := clienttypes.MarshalClientState(solo.cdc, clientState)
	require.NoError(solo.t, err)

	path := host.FullClientStateKey(clientIDSolomachine)
	signBytes := &solomachine.SignBytes{
		Sequence:    solo.Sequence,
		Timestamp:   solo.Time,
		Diversifier: solo.Diversifier,
		Path:        path,
		Data:        data,
	}

	return solo.GenerateProof(signBytes)
}

// GenerateConsensusStateProof generates the proof of the consensus state required for the connection open try and ack handshake steps.
// The consensus state should be the self consensus states of the tendermint chain.
func (solo *Solomachine) GenerateConsensusStateProof(consensusState exported.ConsensusState, consensusHeight exported.Height) []byte {
	data, err := clienttypes.MarshalConsensusState(solo.cdc, consensusState)
	require.NoError(solo.t, err)

	path := host.FullConsensusStateKey(clientIDSolomachine, consensusHeight)
	signBytes := &solomachine.SignBytes{
		Sequence:    solo.Sequence,
		Timestamp:   solo.Time,
		Diversifier: solo.Diversifier,
		Path:        path,
		Data:        data,
	}

	return solo.GenerateProof(signBytes)
}

// GenerateConnOpenTryProof generates the proofTry required for the connection open ack handshake step.
// The clientID, connectionID provided represent the clientID and connectionID created on the counterparty chain, that is the tendermint chain.
func (solo *Solomachine) GenerateConnOpenTryProof(counterpartyClientID, counterpartyConnectionID string) []byte {
	counterparty := connectiontypes.NewCounterparty(counterpartyClientID, counterpartyConnectionID, prefix)
	connection := connectiontypes.NewConnectionEnd(connectiontypes.TRYOPEN, clientIDSolomachine, counterparty, []*connectiontypes.Version{ConnectionVersion}, DefaultDelayPeriod)

	data, err := solo.cdc.Marshal(&connection)
	require.NoError(solo.t, err)

	path := host.ConnectionKey(connectionIDSolomachine)
	signBytes := &solomachine.SignBytes{
		Sequence:    solo.Sequence,
		Timestamp:   solo.Time,
		Diversifier: solo.Diversifier,
		Path:        path,
		Data:        data,
	}

	return solo.GenerateProof(signBytes)
}

// GenerateChanOpenTryProof generates the proofTry required for the channel open ack handshake step.
// The channelID provided represents the channelID created on the counterparty chain, that is the tendermint chain.
func (solo *Solomachine) GenerateChanOpenTryProof(portID, version, counterpartyChannelID string) []byte {
	counterparty := channeltypes.NewCounterparty(portID, counterpartyChannelID)
	channel := channeltypes.NewChannel(channeltypes.TRYOPEN, channeltypes.UNORDERED, counterparty, []string{connectionIDSolomachine}, version)

	data, err := solo.cdc.Marshal(&channel)
	require.NoError(solo.t, err)

	path := host.ChannelKey(portID, channelIDSolomachine)
	signBytes := &solomachine.SignBytes{
		Sequence:    solo.Sequence,
		Timestamp:   solo.Time,
		Diversifier: solo.Diversifier,
		Path:        path,
		Data:        data,
	}

	return solo.GenerateProof(signBytes)
}

// GenerateChanClosedProof generates a channel closed proof.
// The channelID provided represents the channelID created on the counterparty chain, that is the tendermint chain.
func (solo *Solomachine) GenerateChanClosedProof(portID, version, counterpartyChannelID string) []byte {
	counterparty := channeltypes.NewCounterparty(portID, counterpartyChannelID)
	channel := channeltypes.NewChannel(channeltypes.CLOSED, channeltypes.UNORDERED, counterparty, []string{connectionIDSolomachine}, version)

	data, err := solo.cdc.Marshal(&channel)
	require.NoError(solo.t, err)

	path := host.ChannelKey(portID, channelIDSolomachine)
	signBytes := &solomachine.SignBytes{
		Sequence:    solo.Sequence,
		Timestamp:   solo.Time,
		Diversifier: solo.Diversifier,
		Path:        path,
		Data:        data,
	}

	return solo.GenerateProof(signBytes)
}

// GenerateCommitmentProof generates a commitment proof for the provided packet.
func (solo *Solomachine) GenerateCommitmentProof(packet channeltypes.Packet) []byte {
	commitment := channeltypes.CommitPacket(solo.cdc, packet)

	path := host.PacketCommitmentKey(packet.GetSourcePort(), packet.GetSourceChannel(), packet.GetSequence())
	signBytes := &solomachine.SignBytes{
		Sequence:    solo.Sequence,
		Timestamp:   solo.Time,
		Diversifier: solo.Diversifier,
		Path:        path,
		Data:        commitment,
	}

	return solo.GenerateProof(signBytes)
}

// GenerateAcknowledgementProof generates an acknowledgement proof.
func (solo *Solomachine) GenerateAcknowledgementProof(packet channeltypes.Packet) []byte {
	transferAck := channeltypes.NewResultAcknowledgement([]byte{byte(1)}).Acknowledgement()

	path := host.PacketAcknowledgementKey(packet.GetDestPort(), packet.GetDestChannel(), packet.GetSequence())
	signBytes := &solomachine.SignBytes{
		Sequence:    solo.Sequence,
		Timestamp:   solo.Time,
		Diversifier: solo.Diversifier,
		Path:        path,
		Data:        channeltypes.CommitAcknowledgement(transferAck),
	}

	return solo.GenerateProof(signBytes)
}

// GenerateReceiptAbsenceProof generates a receipt absence proof for the provided packet.
func (solo *Solomachine) GenerateReceiptAbsenceProof(packet channeltypes.Packet) []byte {
	path := host.PacketReceiptKey(packet.GetDestPort(), packet.GetDestChannel(), packet.GetSequence())
	signBytes := &solomachine.SignBytes{
		Sequence:    solo.Sequence,
		Timestamp:   solo.Time,
		Diversifier: solo.Diversifier,
		Path:        path,
		Data:        nil,
	}
	return solo.GenerateProof(signBytes)
}

// GetClientStatePath returns the commitment path for the client state.
func (solo *Solomachine) GetClientStatePath(counterpartyClientIdentifier string) commitmenttypes.MerklePath {
	path, err := commitmenttypes.ApplyPrefix(prefix, commitmenttypes.NewMerklePath(host.FullClientStatePath(counterpartyClientIdentifier)))
	require.NoError(solo.t, err)

	return path
}

// GetConsensusStatePath returns the commitment path for the consensus state.
func (solo *Solomachine) GetConsensusStatePath(counterpartyClientIdentifier string, consensusHeight exported.Height) commitmenttypes.MerklePath {
	path, err := commitmenttypes.ApplyPrefix(prefix, commitmenttypes.NewMerklePath(host.FullConsensusStatePath(counterpartyClientIdentifier, consensusHeight)))
	require.NoError(solo.t, err)

	return path
}

// GetConnectionStatePath returns the commitment path for the connection state.
func (solo *Solomachine) GetConnectionStatePath(connID string) commitmenttypes.MerklePath {
	connectionPath := commitmenttypes.NewMerklePath(host.ConnectionPath(connID))
	path, err := commitmenttypes.ApplyPrefix(prefix, connectionPath)
	require.NoError(solo.t, err)

	return path
}

// GetChannelStatePath returns the commitment path for that channel state.
func (solo *Solomachine) GetChannelStatePath(portID, channelID string) commitmenttypes.MerklePath {
	channelPath := commitmenttypes.NewMerklePath(host.ChannelPath(portID, channelID))
	path, err := commitmenttypes.ApplyPrefix(prefix, channelPath)
	require.NoError(solo.t, err)

	return path
}

// GetPacketCommitmentPath returns the commitment path for a packet commitment.
func (solo *Solomachine) GetPacketCommitmentPath(portID, channelID string, sequence uint64) commitmenttypes.MerklePath {
	commitmentPath := commitmenttypes.NewMerklePath(host.PacketCommitmentPath(portID, channelID, sequence))
	path, err := commitmenttypes.ApplyPrefix(prefix, commitmentPath)
	require.NoError(solo.t, err)

	return path
}

// GetPacketAcknowledgementPath returns the commitment path for a packet acknowledgement.
func (solo *Solomachine) GetPacketAcknowledgementPath(portID, channelID string, sequence uint64) commitmenttypes.MerklePath {
	ackPath := commitmenttypes.NewMerklePath(host.PacketAcknowledgementPath(portID, channelID, sequence))
	path, err := commitmenttypes.ApplyPrefix(prefix, ackPath)
	require.NoError(solo.t, err)

	return path
}

// GetPacketReceiptPath returns the commitment path for a packet receipt
// and an absent receipts.
func (solo *Solomachine) GetPacketReceiptPath(portID, channelID string, sequence uint64) commitmenttypes.MerklePath {
	receiptPath := commitmenttypes.NewMerklePath(host.PacketReceiptPath(portID, channelID, sequence))
	path, err := commitmenttypes.ApplyPrefix(prefix, receiptPath)
	require.NoError(solo.t, err)

	return path
}

// GetNextSequenceRecvPath returns the commitment path for the next sequence recv counter.
func (solo *Solomachine) GetNextSequenceRecvPath(portID, channelID string) commitmenttypes.MerklePath {
	nextSequenceRecvPath := commitmenttypes.NewMerklePath(host.NextSequenceRecvPath(portID, channelID))
	path, err := commitmenttypes.ApplyPrefix(prefix, nextSequenceRecvPath)
	require.NoError(solo.t, err)

	return path
}
