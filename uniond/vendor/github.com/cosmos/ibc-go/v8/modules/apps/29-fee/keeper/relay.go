package keeper

import (
	"fmt"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"

	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	ibcexported "github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// SendPacket wraps the ICS4Wrapper SendPacket function
func (k Keeper) SendPacket(
	ctx sdk.Context,
	chanCap *capabilitytypes.Capability,
	sourcePort string,
	sourceChannel string,
	timeoutHeight clienttypes.Height,
	timeoutTimestamp uint64,
	data []byte,
) (uint64, error) {
	return k.ics4Wrapper.SendPacket(ctx, chanCap, sourcePort, sourceChannel, timeoutHeight, timeoutTimestamp, data)
}

// WriteAcknowledgement wraps IBC ChannelKeeper's WriteAcknowledgement function
// ICS29 WriteAcknowledgement is used for asynchronous acknowledgements
func (k Keeper) WriteAcknowledgement(ctx sdk.Context, chanCap *capabilitytypes.Capability, packet ibcexported.PacketI, acknowledgement ibcexported.Acknowledgement) error {
	if !k.IsFeeEnabled(ctx, packet.GetDestPort(), packet.GetDestChannel()) {
		// ics4Wrapper may be core IBC or higher-level middleware
		return k.ics4Wrapper.WriteAcknowledgement(ctx, chanCap, packet, acknowledgement)
	}

	packetID := channeltypes.NewPacketID(packet.GetDestPort(), packet.GetDestChannel(), packet.GetSequence())

	// retrieve the forward relayer that was stored in `onRecvPacket`
	relayer, found := k.GetRelayerAddressForAsyncAck(ctx, packetID)
	if !found {
		return errorsmod.Wrapf(types.ErrRelayerNotFoundForAsyncAck, "no relayer address stored for async acknowledgement for packet with portID: %s, channelID: %s, sequence: %d", packetID.PortId, packetID.ChannelId, packetID.Sequence)
	}

	// it is possible that a relayer has not registered a counterparty address.
	// if there is no registered counterparty address then write acknowledgement with empty relayer address and refund recv_fee.
	forwardRelayer, _ := k.GetCounterpartyPayeeAddress(ctx, relayer, packet.GetDestChannel())

	ack := types.NewIncentivizedAcknowledgement(forwardRelayer, acknowledgement.Acknowledgement(), acknowledgement.Success())

	k.DeleteForwardRelayerAddress(ctx, packetID)

	// ics4Wrapper may be core IBC or higher-level middleware
	return k.ics4Wrapper.WriteAcknowledgement(ctx, chanCap, packet, ack)
}

// GetAppVersion returns the underlying application version.
func (k Keeper) GetAppVersion(ctx sdk.Context, portID, channelID string) (string, bool) {
	version, found := k.ics4Wrapper.GetAppVersion(ctx, portID, channelID)
	if !found {
		return "", false
	}

	if !k.IsFeeEnabled(ctx, portID, channelID) {
		return version, true
	}

	metadata, err := types.MetadataFromVersion(version)
	if err != nil {
		panic(fmt.Errorf("unable to unmarshal metadata for fee enabled channel: %w", err))
	}

	return metadata.AppVersion, true
}
