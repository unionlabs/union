package ibc

import (
	"context"
	"fmt"
	"math"
	"strconv"
	"strings"

	"cosmossdk.io/core/store"
	storetypes "cosmossdk.io/store/types"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	porttypes "github.com/cosmos/ibc-go/v8/modules/core/05-port/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
	"github.com/ethereum/go-ethereum/accounts/abi"
	"golang.org/x/crypto/sha3"
)

const (
	StoreKey = "ethibc"

	EthConnectionCommitmentPrefix = 0x02
	EthChannelCommitmentPrefix    = 0x03
	EthPacketsCommitmentPrefix     = 0x04
	EthPacketAcksCommitmentPrefix = 0x05
	EthNextSeqSendPrefix          = 0x06
	EthNextSeqRecvPrefix          = 0x07
	EthNextSeqAckPrefix           = 0x08
)

var (
	_ porttypes.ICS4Wrapper = IBCDoubleCommitService{}

	EthUint256, _    = abi.NewType("uint256", "", nil)
	EthUint32, _     = abi.NewType("uint32", "", nil)
	EthUint16, _     = abi.NewType("uint16", "", nil)
	EthUint8, _      = abi.NewType("uint8", "", nil)
	EthString, _     = abi.NewType("string", "", nil)
	EthBool, _       = abi.NewType("bool", "", nil)
	EthBytes, _      = abi.NewType("bytes", "", nil)
	EthBytes32, _    = abi.NewType("bytes32", "", nil)
	EthAddress, _    = abi.NewType("address", "", nil)
	EthUint64Arr, _  = abi.NewType("uint64[]", "", nil)
	EthAddressArr, _ = abi.NewType("address[]", "", nil)
	EthInt8, _       = abi.NewType("int8", "", nil)
	EthConnection, _ = abi.NewType("IBCConnection", "", []abi.ArgumentMarshaling{
		{Name: "state", Type: "uint8"},
		{Name: "clientId", Type: "uint32"},
		{Name: "counterpartyClientId", Type: "uint32"},
		{Name: "counterpartyConnectionId", Type: "uint32"},
	})
	EthChannel, _ = abi.NewType("IBCChannel", "", []abi.ArgumentMarshaling{
		{Name: "state", Type: "uint8"},
		{Name: "ordering", Type: "uint8"},
		{Name: "connectionId", Type: "uint32"},
		{Name: "counterpartyChannelId", Type: "uint32"},
		{Name: "version", Type: "bytes32"},
	})
	EthPacket, _ = abi.NewType("IBCPacket", "", []abi.ArgumentMarshaling{
		{Name: "sequence", Type: "uint64"},
		{Name: "sourceChannel", Type: "uint32"},
		{Name: "destinationChannel", Type: "uint32"},
		{Name: "data", Type: "uint32"},
		{Name: "timeoutHeight", Type: "uint64"},
		{Name: "timeoutTimestamp", Type: "uint64"},
	})
)

type ethConnection struct {
	state                    uint8
	clientId                 uint32
	counterpartyClientId     uint32
	counterpartyConnectionId uint32
}

type ethChannel struct {
	state                 uint8
	ordering              uint8
	connectionId          uint32
	counterpartyChannelId uint32
	version               [32]byte
}

type IBCDoubleCommitService struct {
	cdc           codec.Codec
	commitKey     *storetypes.KVStoreKey
	ibcKey        *storetypes.KVStoreKey
	ics4Wrapper   porttypes.ICS4Wrapper
	sendingPacket *channeltypes.Packet
}

func NewIBCDoubleCommitService(
	codec codec.Codec,
	ics4Wrapper porttypes.ICS4Wrapper,
	commitKey *storetypes.KVStoreKey,
	ibcKey *storetypes.KVStoreKey,
) store.KVStoreService {
	return &IBCDoubleCommitService{
		cdc:           codec,
		commitKey:     commitKey,
		ibcKey:        ibcKey,
		ics4Wrapper:   ics4Wrapper,
		sendingPacket: nil,
	}
}

func (t IBCDoubleCommitService) SendPacket(
	ctx context.Context,
	chanCap *capabilitytypes.Capability,
	sourcePort string,
	sourceChannel string,
	timeoutHeight clienttypes.Height,
	timeoutTimestamp uint64,
	data []byte,
) (sequence uint64, err error) {
	t.sendingPacket = &channeltypes.Packet{
		Sequence:           0,          // parsed from the callback
		SourcePort:         sourcePort, // not needed
		SourceChannel:      sourceChannel,
		DestinationPort:    "", // not needed
		DestinationChannel: "", // not needed
		TimeoutHeight:      timeoutHeight,
		TimeoutTimestamp:   timeoutTimestamp,
		Data:               data,
	}
	seq, err := t.ics4Wrapper.SendPacket(ctx, chanCap, sourcePort, sourceChannel, timeoutHeight, timeoutTimestamp, data)
	t.sendingPacket = nil
	return seq, err
}

func (t IBCDoubleCommitService) WriteAcknowledgement(
	ctx context.Context,
	chanCap *capabilitytypes.Capability,
	packet exported.PacketI,
	ack exported.Acknowledgement,
) error {
	return t.ics4Wrapper.WriteAcknowledgement(ctx, chanCap, packet, ack)
}

func (t IBCDoubleCommitService) GetAppVersion(
	ctx context.Context,
	portID,
	channelID string,
) (string, bool) {
	return t.ics4Wrapper.GetAppVersion(ctx, portID, channelID)
}

func (t IBCDoubleCommitService) OpenKVStore(ctx context.Context) store.KVStore {
	return newKVStore(t.cdc, sdk.UnwrapSDKContext(ctx).KVStore(t.commitKey), sdk.UnwrapSDKContext(ctx).KVStore(t.ibcKey))
}

type coreDoubleCommitStore struct {
	cdc           codec.Codec
	commitStore   storetypes.KVStore
	ibcStore      storetypes.KVStore
	sendingPacket *channeltypes.Packet
}

func newKVStore(cdc codec.Codec, commitStore storetypes.KVStore, ibcStore storetypes.KVStore) store.KVStore {
	return coreDoubleCommitStore{
		cdc:         cdc,
		commitStore: commitStore,
		ibcStore:    ibcStore,
	}
}

func (s coreDoubleCommitStore) Get(key []byte) ([]byte, error) {
	return s.ibcStore.Get(key), nil
}

func (s coreDoubleCommitStore) Has(key []byte) (bool, error) {
	return s.ibcStore.Has(key), nil
}

func (s coreDoubleCommitStore) Set(key, value []byte) error {
	s.ibcStore.Set(key, value)
	// double commit depending on the path, ordered by hotest to coldest path
	keyStr := string(key)
	// packet commitment
	if sequence, err := parsePacketCommitmentPath(string(key)); err == nil {
	}
	if sequence, err := parsePacketReceiptPath(string(key)); err == nil {

	}
	if sequence, err := parsePacketAckPath(string(key)); err == nil {

	}
	// channel commitment
	if strings.HasPrefix(keyStr, host.KeyChannelEndPrefix) {
		_, channelId, err := host.ParseChannelPath(string(key))
		if err != nil {
			return err
		}
		id, err := channeltypes.ParseChannelSequence(channelId)
		if err != nil {
			return err
		}
		if id > math.MaxUint32 {
			return fmt.Errorf(
				"can't parse channel, id > MaxUint32: %d",
				id,
			)
		}
		var channel channeltypes.Channel
		s.cdc.MustUnmarshal(value, &channel)
		commitmentValue, err := commitChannel(channel)
		if err != nil {
			return err
		}
		commitmentKey, err := channelCommitmentKey(uint32(id))
		if err != nil {
			return err
		}
		s.commitStore.Set(commitmentKey, commitmentValue)
	}
	// connection commitment
	if strings.HasPrefix(keyStr, host.KeyConnectionPrefix) {
		connectionId, err := host.ParseConnectionPath(string(key))
		if err != nil {
			return err
		}
		id, err := connectiontypes.ParseConnectionSequence(connectionId)
		if err != nil {
			return err
		}
		if id > math.MaxUint32 {
			return fmt.Errorf(
				"can't parse connection, id > MaxUint32: %d",
				id,
			)
		}
		var connection connectiontypes.ConnectionEnd
		s.cdc.MustUnmarshal(value, &connection)
		commitmentValue, err := commitConnection(connection)
		if err != nil {
			return err
		}
		commitmentKey, err := connectionCommitmentKey(uint32(id))
		if err != nil {
			return err
		}
		s.commitStore.Set(commitmentKey, commitmentValue)
	}
	return nil
}

func (s coreDoubleCommitStore) Delete(key []byte) error {
	s.ibcStore.Delete(key)
	return nil
}

func (s coreDoubleCommitStore) Iterator(start, end []byte) (store.Iterator, error) {
	return s.ibcStore.Iterator(start, end), nil
}

func (s coreDoubleCommitStore) ReverseIterator(start, end []byte) (store.Iterator, error) {
	return s.ibcStore.ReverseIterator(start, end), nil
}

// "commitments/ports/{identifier}/channels/{identifier}/sequences/{sequence}"
func parsePacketCommitmentPath(path string) (uint64, error) {
	split := strings.Split(path, "/")
	if len(split) < 7 {
		return 0, fmt.Errorf("cannot parse packet commitment path")
	}
	if split[0] != host.KeyPacketCommitmentPrefix ||
		split[1] != host.KeyPortPrefix ||
		split[3] != host.KeyChannelPrefix ||
		split[5] != host.KeySequencePrefix {
		return 0, fmt.Errorf("cannot parse packet commitment path")
	}
	sequence, err := strconv.ParseUint(split[6], 10, 64)
	if err != nil {
		return 0, fmt.Errorf("cannot parse packet commitment path")
	}
	return sequence, nil
}

// "acks/ports/{identifier}/channels/{identifier}/sequences/{sequence}"
func parsePacketAckPath(path string) (uint64, error) {
	split := strings.Split(path, "/")
	if len(split) < 7 {
		return 0, fmt.Errorf("cannot parse packet ack path")
	}
	if split[0] != host.KeyPacketAckPrefix ||
		split[1] != host.KeyPortPrefix ||
		split[3] != host.KeyChannelPrefix ||
		split[5] != host.KeySequencePrefix {
		return 0, fmt.Errorf("cannot parse packet ack path")
	}
	sequence, err := strconv.ParseUint(split[6], 10, 64)
	if err != nil {
		return 0, fmt.Errorf("cannot parse packet ack path")
	}
	return sequence, nil
}

// "receipts/ports/{identifier}/channels/{identifier}/sequences/{sequence}"
func parsePacketReceiptPath(path string) (uint64, error) {
	split := strings.Split(path, "/")
	if len(split) < 7 {
		return 0, fmt.Errorf("cannot parse packet receipt path")
	}
	if split[0] != host.KeyPacketReceiptPrefix ||
		split[1] != host.KeyPortPrefix ||
		split[3] != host.KeyChannelPrefix ||
		split[5] != host.KeySequencePrefix {
		return 0, fmt.Errorf("cannot parse packet receipt path")
	}
	sequence, err := strconv.ParseUint(split[6], 10, 64)
	if err != nil {
		return 0, fmt.Errorf("cannot parse packet receipt path")
	}
	return sequence, nil
}

func keccak(bz []byte) [32]byte {
	hash := sha3.NewLegacyKeccak256()
	hash.Write(bz)
	var buf [32]byte
	copy(buf[:], hash.Sum(nil))
	return buf
}

func commitChannel(channel channeltypes.Channel) ([]byte, error) {
	if len(channel.ConnectionHops) != 1 {
		return nil, fmt.Errorf(
			"can't commit channel, expected 1 connection hop, got %d",
			len(channel.ConnectionHops),
		)
	}
	connectionId, err := connectiontypes.ParseConnectionSequence(channel.ConnectionHops[0])
	if err != nil {
		return nil, err
	}
	if connectionId > math.MaxUint32 {
		return nil, fmt.Errorf(
			"can't commit channel, connectionId > MaxUint32: %d",
			connectionId,
		)
	}
	counterpartyChannelId, err := channeltypes.ParseChannelSequence(channel.Counterparty.ChannelId)
	if err != nil {
		return nil, err
	}
	if counterpartyChannelId > math.MaxUint32 {
		return nil, fmt.Errorf(
			"can't commit channel, counterpartyChannelId > MaxUint32: %d",
			counterpartyChannelId,
		)
	}
	arguments := abi.Arguments{
		{Name: "channel", Type: EthChannel},
	}
	bytes, err := arguments.Pack(
		ethChannel{
			state:                 uint8(channel.State),
			ordering:              uint8(channel.Ordering),
			connectionId:          uint32(connectionId),
			counterpartyChannelId: uint32(counterpartyChannelId),
			version:               keccak([]byte(channel.Version)),
		},
	)
	if err != nil {
		return nil, err
	}
	hash := keccak(bytes)
	return hash[:], nil
}

func commitConnection(connection connectiontypes.ConnectionEnd) ([]byte, error) {
	_, clientId, err := clienttypes.ParseClientIdentifier(connection.ClientId)
	if err != nil {
		return nil, err
	}
	if clientId > math.MaxUint32 {
		return nil, fmt.Errorf(
			"can't commit connection, clientId > MaxUint32: %d",
			clientId,
		)
	}
	_, counterpartyClientId, err := clienttypes.ParseClientIdentifier(connection.Counterparty.ClientId)
	if err != nil {
		return nil, err
	}
	if counterpartyClientId > math.MaxUint32 {
		return nil, fmt.Errorf(
			"can't commit connection, counterpartyClientId > MaxUint32: %d",
			counterpartyClientId,
		)
	}
	counterpartyConnectionId, err := connectiontypes.ParseConnectionSequence(connection.Counterparty.ConnectionId)
	if counterpartyConnectionId > math.MaxUint32 {
		return nil, fmt.Errorf(
			"can't commit connection, counterpartyConnectionId > MaxUint32: %d",
			counterpartyConnectionId,
		)
	}
	arguments := abi.Arguments{
		{Name: "connection", Type: EthConnection},
	}
	bytes, err := arguments.Pack(
		ethConnection{
			state:                    uint8(connection.State),
			clientId:                 uint32(clientId),
			counterpartyClientId:     uint32(counterpartyClientId),
			counterpartyConnectionId: uint32(counterpartyConnectionId),
		},
	)
	if err != nil {
		return nil, err
	}
	hash := keccak(bytes)
	return hash[:], nil
}

func connectionCommitmentKey(connectionId uint32) ([]byte, error) {
	arguments := abi.Arguments{
		{Name: "prefix", Type: EthUint8},
		{Name: "connectionId", Type: EthUint32},
	}
	bytes, err := arguments.Pack(
		uint8(EthConnectionCommitmentPrefix),
		connectionId,
	)
	if err != nil {
		return nil, err
	}
	hash := keccak(bytes)
	return hash[:], nil
}

func channelCommitmentKey(channelId uint32) ([]byte, error) {
	arguments := abi.Arguments{
		{Name: "prefix", Type: EthUint8},
		{Name: "channelId", Type: EthUint32},
	}
	bytes, err := arguments.Pack(
		uint8(EthChannelCommitmentPrefix),
		channelId,
	)
	if err != nil {
		return nil, err
	}
	hash := keccak(bytes)
	return hash[:], nil
}

func batchPacketsCommitmentKey(channelId uint32, batchHash [32]byte) ([]byte, error) {
	arguments := abi.Arguments{
		{Name: "prefix", Type: EthUint8},
		{Name: "channelId", Type: EthUint32},
		{Name: "batchHash", Type: EthBytes32},
	}
	bytes, err := arguments.Pack(
		uint8(EthPacketsCommitmentPrefix),
		channelId,
		batchHash,
	)
	if err != nil {
		return nil, err
	}
	hash := keccak(bytes)
	return hash[:], nil
}

func batchPacketReceiptsCommitmentKey(channelId uint32, batchHash [32]byte) ([]byte, error) {
	arguments := abi.Arguments{
		{Name: "prefix", Type: EthUint8},
		{Name: "channelId", Type: EthUint32},
		{Name: "batchHash", Type: EthBytes32},
	}
	bytes, err := arguments.Pack(
		uint8(EthPacketAcksCommitmentPrefix),
		channelId,
		batchHash,
	)
	if err != nil {
		return nil, err
	}
	hash := keccak(bytes)
	return hash[:], nil
}
