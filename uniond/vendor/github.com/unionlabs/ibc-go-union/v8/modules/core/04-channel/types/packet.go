package types

import (
	"crypto/sha256"

	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// CommitPacket returns the packet commitment bytes. The commitment consists of:
// sha256_hash(timeout_timestamp + timeout_height.RevisionNumber + timeout_height.RevisionHeight + sha256_hash(data))
// from a given packet. This results in a fixed length preimage.
// NOTE: sdk.Uint64ToBigEndian sets the uint64 to a slice of length 8.
func CommitPacket(cdc codec.BinaryCodec, packet exported.PacketI) []byte {
	timeoutHeight := packet.GetTimeoutHeight()

	buf := sdk.Uint64ToBigEndian(packet.GetTimeoutTimestamp())

	revisionNumber := sdk.Uint64ToBigEndian(timeoutHeight.GetRevisionNumber())
	buf = append(buf, revisionNumber...)

	revisionHeight := sdk.Uint64ToBigEndian(timeoutHeight.GetRevisionHeight())
	buf = append(buf, revisionHeight...)

	dataHash := sha256.Sum256(packet.GetData())
	buf = append(buf, dataHash[:]...)

	hash := sha256.Sum256(buf)
	return hash[:]
}

// CommitAcknowledgement returns the hash of commitment bytes
func CommitAcknowledgement(data []byte) []byte {
	hash := sha256.Sum256(data)
	return hash[:]
}

var _ exported.PacketI = (*Packet)(nil)

// NewPacket creates a new Packet instance. It panics if the provided
// packet data interface is not registered.
func NewPacket(
	data []byte,
	sequence uint64, sourcePort, sourceChannel,
	destinationPort, destinationChannel string,
	timeoutHeight clienttypes.Height, timeoutTimestamp uint64,
) Packet {
	return Packet{
		Data:               data,
		Sequence:           sequence,
		SourcePort:         sourcePort,
		SourceChannel:      sourceChannel,
		DestinationPort:    destinationPort,
		DestinationChannel: destinationChannel,
		TimeoutHeight:      timeoutHeight,
		TimeoutTimestamp:   timeoutTimestamp,
	}
}

// GetSequence implements PacketI interface
func (p Packet) GetSequence() uint64 { return p.Sequence }

// GetSourcePort implements PacketI interface
func (p Packet) GetSourcePort() string { return p.SourcePort }

// GetSourceChannel implements PacketI interface
func (p Packet) GetSourceChannel() string { return p.SourceChannel }

// GetDestPort implements PacketI interface
func (p Packet) GetDestPort() string { return p.DestinationPort }

// GetDestChannel implements PacketI interface
func (p Packet) GetDestChannel() string { return p.DestinationChannel }

// GetData implements PacketI interface
func (p Packet) GetData() []byte { return p.Data }

// GetTimeoutHeight implements PacketI interface
func (p Packet) GetTimeoutHeight() exported.Height { return p.TimeoutHeight }

// GetTimeoutTimestamp implements PacketI interface
func (p Packet) GetTimeoutTimestamp() uint64 { return p.TimeoutTimestamp }

// ValidateBasic implements PacketI interface
func (p Packet) ValidateBasic() error {
	if err := host.PortIdentifierValidator(p.SourcePort); err != nil {
		return errorsmod.Wrap(err, "invalid source port ID")
	}
	if err := host.PortIdentifierValidator(p.DestinationPort); err != nil {
		return errorsmod.Wrap(err, "invalid destination port ID")
	}
	if err := host.ChannelIdentifierValidator(p.SourceChannel); err != nil {
		return errorsmod.Wrap(err, "invalid source channel ID")
	}
	if err := host.ChannelIdentifierValidator(p.DestinationChannel); err != nil {
		return errorsmod.Wrap(err, "invalid destination channel ID")
	}
	if p.Sequence == 0 {
		return errorsmod.Wrap(ErrInvalidPacket, "packet sequence cannot be 0")
	}
	if p.TimeoutHeight.IsZero() && p.TimeoutTimestamp == 0 {
		return errorsmod.Wrap(ErrInvalidPacket, "packet timeout height and packet timeout timestamp cannot both be 0")
	}
	if len(p.Data) == 0 {
		return errorsmod.Wrap(ErrInvalidPacket, "packet data bytes cannot be empty")
	}
	return nil
}

// Validates a PacketId
func (p PacketId) Validate() error {
	if err := host.PortIdentifierValidator(p.PortId); err != nil {
		return errorsmod.Wrap(err, "invalid source port ID")
	}

	if err := host.ChannelIdentifierValidator(p.ChannelId); err != nil {
		return errorsmod.Wrap(err, "invalid source channel ID")
	}

	if p.Sequence == 0 {
		return errorsmod.Wrap(ErrInvalidPacket, "packet sequence cannot be 0")
	}

	return nil
}

// NewPacketID returns a new instance of PacketId
func NewPacketID(portID, channelID string, seq uint64) PacketId {
	return PacketId{PortId: portID, ChannelId: channelID, Sequence: seq}
}
