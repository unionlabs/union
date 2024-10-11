package types

import (
	"fmt"
	"strconv"
	"strings"

	errorsmod "cosmossdk.io/errors"

	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
)

const (
	// ModuleName defines the 29-fee name
	ModuleName = "feeibc"

	// StoreKey is the store key string for IBC fee module
	StoreKey = ModuleName

	// RouterKey is the message route for IBC fee module
	RouterKey = ModuleName

	// QuerierRoute is the querier route for IBC fee module
	QuerierRoute = ModuleName

	Version = "ics29-1"

	// FeeEnabledPrefix is the key prefix for storing fee enabled flag
	FeeEnabledKeyPrefix = "feeEnabled"

	// PayeeKeyPrefix is the key prefix for the fee payee address stored in state
	PayeeKeyPrefix = "payee"

	// CounterpartyPayeeKeyPrefix is the key prefix for the counterparty payee address mapping
	CounterpartyPayeeKeyPrefix = "counterpartyPayee"

	// FeesInEscrowPrefix is the key prefix for fee in escrow mapping
	FeesInEscrowPrefix = "feesInEscrow"

	// ForwardRelayerPrefix is the key prefix for forward relayer addresses stored in state for async acknowledgements
	ForwardRelayerPrefix = "forwardRelayer"
)

// KeyLocked returns the key used to lock and unlock the fee module. This key is used
// in the presence of a severe bug.
func KeyLocked() []byte {
	return []byte("locked")
}

// KeyFeeEnabled returns the key that stores a flag to determine if fee logic should
// be enabled for the given port and channel identifiers.
func KeyFeeEnabled(portID, channelID string) []byte {
	return []byte(fmt.Sprintf("%s/%s/%s", FeeEnabledKeyPrefix, portID, channelID))
}

// ParseKeyFeeEnabled parses the key used to indicate if the fee logic should be
// enabled for the given port and channel identifiers.
func ParseKeyFeeEnabled(key string) (portID, channelID string, err error) {
	keySplit := strings.Split(key, "/")
	if len(keySplit) != 3 {
		return "", "", errorsmod.Wrapf(
			ibcerrors.ErrLogic, "key provided is incorrect: the key split has incorrect length, expected %d, got %d", 3, len(keySplit),
		)
	}

	if keySplit[0] != FeeEnabledKeyPrefix {
		return "", "", errorsmod.Wrapf(ibcerrors.ErrLogic, "key prefix is incorrect: expected %s, got %s", FeeEnabledKeyPrefix, keySplit[0])
	}

	portID = keySplit[1]
	channelID = keySplit[2]

	return portID, channelID, nil
}

// KeyPayee returns the key for relayer address -> payee address mapping
func KeyPayee(relayerAddr, channelID string) []byte {
	return []byte(fmt.Sprintf("%s/%s/%s", PayeeKeyPrefix, relayerAddr, channelID))
}

// ParseKeyPayeeAddress returns the registered relayer addresss and channelID used to the store the fee payee address
func ParseKeyPayeeAddress(key string) (relayerAddr, channelID string, err error) {
	keySplit := strings.Split(key, "/")
	if len(keySplit) != 3 {
		return "", "", errorsmod.Wrapf(
			ibcerrors.ErrLogic, "key provided is incorrect: the key split has incorrect length, expected %d, got %d", 3, len(keySplit),
		)
	}

	return keySplit[1], keySplit[2], nil
}

// KeyCounterpartyPayee returns the key for relayer address -> counterparty payee address mapping
func KeyCounterpartyPayee(address, channelID string) []byte {
	return []byte(fmt.Sprintf("%s/%s/%s", CounterpartyPayeeKeyPrefix, address, channelID))
}

// ParseKeyCounterpartyPayee returns the registered relayer address and channelID used to store the counterparty payee address
func ParseKeyCounterpartyPayee(key string) (address string, channelID string, error error) {
	keySplit := strings.Split(key, "/")
	if len(keySplit) != 3 {
		return "", "", errorsmod.Wrapf(
			ibcerrors.ErrLogic, "key provided is incorrect: the key split has incorrect length, expected %d, got %d", 3, len(keySplit),
		)
	}

	return keySplit[1], keySplit[2], nil
}

// KeyRelayerAddressForAsyncAck returns the key for packetID -> forwardAddress mapping
func KeyRelayerAddressForAsyncAck(packetID channeltypes.PacketId) []byte {
	return []byte(fmt.Sprintf("%s/%s/%s/%d", ForwardRelayerPrefix, packetID.PortId, packetID.ChannelId, packetID.Sequence))
}

// ParseKeyRelayerAddressForAsyncAck parses the key used to store the forward relayer address and returns the packetID
func ParseKeyRelayerAddressForAsyncAck(key string) (channeltypes.PacketId, error) {
	keySplit := strings.Split(key, "/")
	if len(keySplit) != 4 {
		return channeltypes.PacketId{}, errorsmod.Wrapf(
			ibcerrors.ErrLogic, "key provided is incorrect: the key split has incorrect length, expected %d, got %d", 4, len(keySplit),
		)
	}

	seq, err := strconv.ParseUint(keySplit[3], 10, 64)
	if err != nil {
		return channeltypes.PacketId{}, err
	}

	packetID := channeltypes.NewPacketID(keySplit[1], keySplit[2], seq)
	return packetID, nil
}

// KeyFeesInEscrow returns the key for escrowed fees
func KeyFeesInEscrow(packetID channeltypes.PacketId) []byte {
	return []byte(fmt.Sprintf("%s/%d", KeyFeesInEscrowChannelPrefix(packetID.PortId, packetID.ChannelId), packetID.Sequence))
}

// ParseKeyFeesInEscrow parses the key used to store fees in escrow and returns the packet id
func ParseKeyFeesInEscrow(key string) (channeltypes.PacketId, error) {
	keySplit := strings.Split(key, "/")
	if len(keySplit) != 4 {
		return channeltypes.PacketId{}, errorsmod.Wrapf(
			ibcerrors.ErrLogic, "key provided is incorrect: the key split has incorrect length, expected %d, got %d", 4, len(keySplit),
		)
	}

	seq, err := strconv.ParseUint(keySplit[3], 10, 64)
	if err != nil {
		return channeltypes.PacketId{}, err
	}

	packetID := channeltypes.NewPacketID(keySplit[1], keySplit[2], seq)
	return packetID, nil
}

// KeyFeesInEscrowChannelPrefix returns the key prefix for escrowed fees on the given channel
func KeyFeesInEscrowChannelPrefix(portID, channelID string) []byte {
	return []byte(fmt.Sprintf("%s/%s/%s", FeesInEscrowPrefix, portID, channelID))
}
