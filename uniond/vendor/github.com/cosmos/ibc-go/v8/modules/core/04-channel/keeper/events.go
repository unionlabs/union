package keeper

import (
	"encoding/hex"
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// emitChannelOpenInitEvent emits a channel open init event
func emitChannelOpenInitEvent(ctx sdk.Context, portID string, channelID string, channel types.Channel) {
	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeChannelOpenInit,
			sdk.NewAttribute(types.AttributeKeyPortID, portID),
			sdk.NewAttribute(types.AttributeKeyChannelID, channelID),
			sdk.NewAttribute(types.AttributeCounterpartyPortID, channel.Counterparty.PortId),
			sdk.NewAttribute(types.AttributeKeyConnectionID, channel.ConnectionHops[0]),
			sdk.NewAttribute(types.AttributeVersion, channel.Version),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// emitChannelOpenTryEvent emits a channel open try event
func emitChannelOpenTryEvent(ctx sdk.Context, portID string, channelID string, channel types.Channel) {
	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeChannelOpenTry,
			sdk.NewAttribute(types.AttributeKeyPortID, portID),
			sdk.NewAttribute(types.AttributeKeyChannelID, channelID),
			sdk.NewAttribute(types.AttributeCounterpartyPortID, channel.Counterparty.PortId),
			sdk.NewAttribute(types.AttributeCounterpartyChannelID, channel.Counterparty.ChannelId),
			sdk.NewAttribute(types.AttributeKeyConnectionID, channel.ConnectionHops[0]),
			sdk.NewAttribute(types.AttributeVersion, channel.Version),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// emitChannelOpenAckEvent emits a channel open acknowledge event
func emitChannelOpenAckEvent(ctx sdk.Context, portID string, channelID string, channel types.Channel) {
	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeChannelOpenAck,
			sdk.NewAttribute(types.AttributeKeyPortID, portID),
			sdk.NewAttribute(types.AttributeKeyChannelID, channelID),
			sdk.NewAttribute(types.AttributeCounterpartyPortID, channel.Counterparty.PortId),
			sdk.NewAttribute(types.AttributeCounterpartyChannelID, channel.Counterparty.ChannelId),
			sdk.NewAttribute(types.AttributeKeyConnectionID, channel.ConnectionHops[0]),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// emitChannelOpenConfirmEvent emits a channel open confirm event
func emitChannelOpenConfirmEvent(ctx sdk.Context, portID string, channelID string, channel types.Channel) {
	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeChannelOpenConfirm,
			sdk.NewAttribute(types.AttributeKeyPortID, portID),
			sdk.NewAttribute(types.AttributeKeyChannelID, channelID),
			sdk.NewAttribute(types.AttributeCounterpartyPortID, channel.Counterparty.PortId),
			sdk.NewAttribute(types.AttributeCounterpartyChannelID, channel.Counterparty.ChannelId),
			sdk.NewAttribute(types.AttributeKeyConnectionID, channel.ConnectionHops[0]),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// emitChannelCloseInitEvent emits a channel close init event
func emitChannelCloseInitEvent(ctx sdk.Context, portID string, channelID string, channel types.Channel) {
	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeChannelCloseInit,
			sdk.NewAttribute(types.AttributeKeyPortID, portID),
			sdk.NewAttribute(types.AttributeKeyChannelID, channelID),
			sdk.NewAttribute(types.AttributeCounterpartyPortID, channel.Counterparty.PortId),
			sdk.NewAttribute(types.AttributeCounterpartyChannelID, channel.Counterparty.ChannelId),
			sdk.NewAttribute(types.AttributeKeyConnectionID, channel.ConnectionHops[0]),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// emitChannelCloseConfirmEvent emits a channel close confirm event
func emitChannelCloseConfirmEvent(ctx sdk.Context, portID string, channelID string, channel types.Channel) {
	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeChannelCloseConfirm,
			sdk.NewAttribute(types.AttributeKeyPortID, portID),
			sdk.NewAttribute(types.AttributeKeyChannelID, channelID),
			sdk.NewAttribute(types.AttributeCounterpartyPortID, channel.Counterparty.PortId),
			sdk.NewAttribute(types.AttributeCounterpartyChannelID, channel.Counterparty.ChannelId),
			sdk.NewAttribute(types.AttributeKeyConnectionID, channel.ConnectionHops[0]),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// emitSendPacketEvent emits an event with packet data along with other packet information for relayer
// to pick up and relay to other chain
func emitSendPacketEvent(ctx sdk.Context, packet exported.PacketI, channel types.Channel, timeoutHeight exported.Height) {
	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeSendPacket,
			sdk.NewAttribute(types.AttributeKeyData, string(packet.GetData())), //nolint:staticcheck // DEPRECATED
			sdk.NewAttribute(types.AttributeKeyDataHex, hex.EncodeToString(packet.GetData())),
			sdk.NewAttribute(types.AttributeKeyTimeoutHeight, timeoutHeight.String()),
			sdk.NewAttribute(types.AttributeKeyTimeoutTimestamp, fmt.Sprintf("%d", packet.GetTimeoutTimestamp())),
			sdk.NewAttribute(types.AttributeKeySequence, fmt.Sprintf("%d", packet.GetSequence())),
			sdk.NewAttribute(types.AttributeKeySrcPort, packet.GetSourcePort()),
			sdk.NewAttribute(types.AttributeKeySrcChannel, packet.GetSourceChannel()),
			sdk.NewAttribute(types.AttributeKeyDstPort, packet.GetDestPort()),
			sdk.NewAttribute(types.AttributeKeyDstChannel, packet.GetDestChannel()),
			sdk.NewAttribute(types.AttributeKeyChannelOrdering, channel.Ordering.String()),
			// we only support 1-hop packets now, and that is the most important hop for a relayer
			// (is it going to a chain I am connected to)
			sdk.NewAttribute(types.AttributeKeyConnection, channel.ConnectionHops[0]), // DEPRECATED
			sdk.NewAttribute(types.AttributeKeyConnectionID, channel.ConnectionHops[0]),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// emitRecvPacketEvent emits a receive packet event. It will be emitted both the first time a packet
// is received for a certain sequence and for all duplicate receives.
func emitRecvPacketEvent(ctx sdk.Context, packet exported.PacketI, channel types.Channel) {
	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeRecvPacket,
			sdk.NewAttribute(types.AttributeKeyData, string(packet.GetData())), //nolint:staticcheck // DEPRECATED
			sdk.NewAttribute(types.AttributeKeyDataHex, hex.EncodeToString(packet.GetData())),
			sdk.NewAttribute(types.AttributeKeyTimeoutHeight, packet.GetTimeoutHeight().String()),
			sdk.NewAttribute(types.AttributeKeyTimeoutTimestamp, fmt.Sprintf("%d", packet.GetTimeoutTimestamp())),
			sdk.NewAttribute(types.AttributeKeySequence, fmt.Sprintf("%d", packet.GetSequence())),
			sdk.NewAttribute(types.AttributeKeySrcPort, packet.GetSourcePort()),
			sdk.NewAttribute(types.AttributeKeySrcChannel, packet.GetSourceChannel()),
			sdk.NewAttribute(types.AttributeKeyDstPort, packet.GetDestPort()),
			sdk.NewAttribute(types.AttributeKeyDstChannel, packet.GetDestChannel()),
			sdk.NewAttribute(types.AttributeKeyChannelOrdering, channel.Ordering.String()),
			// we only support 1-hop packets now, and that is the most important hop for a relayer
			// (is it going to a chain I am connected to)
			sdk.NewAttribute(types.AttributeKeyConnection, channel.ConnectionHops[0]), // DEPRECATED
			sdk.NewAttribute(types.AttributeKeyConnectionID, channel.ConnectionHops[0]),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// emitWriteAcknowledgementEvent emits an event that the relayer can query for
func emitWriteAcknowledgementEvent(ctx sdk.Context, packet exported.PacketI, channel types.Channel, acknowledgement []byte) {
	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeWriteAck,
			sdk.NewAttribute(types.AttributeKeyData, string(packet.GetData())), //nolint:staticcheck // DEPRECATED
			sdk.NewAttribute(types.AttributeKeyDataHex, hex.EncodeToString(packet.GetData())),
			sdk.NewAttribute(types.AttributeKeyTimeoutHeight, packet.GetTimeoutHeight().String()),
			sdk.NewAttribute(types.AttributeKeyTimeoutTimestamp, fmt.Sprintf("%d", packet.GetTimeoutTimestamp())),
			sdk.NewAttribute(types.AttributeKeySequence, fmt.Sprintf("%d", packet.GetSequence())),
			sdk.NewAttribute(types.AttributeKeySrcPort, packet.GetSourcePort()),
			sdk.NewAttribute(types.AttributeKeySrcChannel, packet.GetSourceChannel()),
			sdk.NewAttribute(types.AttributeKeyDstPort, packet.GetDestPort()),
			sdk.NewAttribute(types.AttributeKeyDstChannel, packet.GetDestChannel()),
			sdk.NewAttribute(types.AttributeKeyAck, string(acknowledgement)), //nolint:staticcheck // DEPRECATED
			sdk.NewAttribute(types.AttributeKeyAckHex, hex.EncodeToString(acknowledgement)),
			// we only support 1-hop packets now, and that is the most important hop for a relayer
			// (is it going to a chain I am connected to)
			sdk.NewAttribute(types.AttributeKeyConnection, channel.ConnectionHops[0]), // DEPRECATED
			sdk.NewAttribute(types.AttributeKeyConnectionID, channel.ConnectionHops[0]),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// emitAcknowledgePacketEvent emits an acknowledge packet event. It will be emitted both the first time
// a packet is acknowledged for a certain sequence and for all duplicate acknowledgements.
func emitAcknowledgePacketEvent(ctx sdk.Context, packet exported.PacketI, channel types.Channel) {
	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeAcknowledgePacket,
			sdk.NewAttribute(types.AttributeKeyTimeoutHeight, packet.GetTimeoutHeight().String()),
			sdk.NewAttribute(types.AttributeKeyTimeoutTimestamp, fmt.Sprintf("%d", packet.GetTimeoutTimestamp())),
			sdk.NewAttribute(types.AttributeKeySequence, fmt.Sprintf("%d", packet.GetSequence())),
			sdk.NewAttribute(types.AttributeKeySrcPort, packet.GetSourcePort()),
			sdk.NewAttribute(types.AttributeKeySrcChannel, packet.GetSourceChannel()),
			sdk.NewAttribute(types.AttributeKeyDstPort, packet.GetDestPort()),
			sdk.NewAttribute(types.AttributeKeyDstChannel, packet.GetDestChannel()),
			sdk.NewAttribute(types.AttributeKeyChannelOrdering, channel.Ordering.String()),
			// we only support 1-hop packets now, and that is the most important hop for a relayer
			// (is it going to a chain I am connected to)
			sdk.NewAttribute(types.AttributeKeyConnection, channel.ConnectionHops[0]), // DEPRECATED
			sdk.NewAttribute(types.AttributeKeyConnectionID, channel.ConnectionHops[0]),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// emitTimeoutPacketEvent emits a timeout packet event. It will be emitted both the first time a packet
// is timed out for a certain sequence and for all duplicate timeouts.
func emitTimeoutPacketEvent(ctx sdk.Context, packet exported.PacketI, channel types.Channel) {
	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeTimeoutPacket,
			sdk.NewAttribute(types.AttributeKeyTimeoutHeight, packet.GetTimeoutHeight().String()),
			sdk.NewAttribute(types.AttributeKeyTimeoutTimestamp, fmt.Sprintf("%d", packet.GetTimeoutTimestamp())),
			sdk.NewAttribute(types.AttributeKeySequence, fmt.Sprintf("%d", packet.GetSequence())),
			sdk.NewAttribute(types.AttributeKeySrcPort, packet.GetSourcePort()),
			sdk.NewAttribute(types.AttributeKeySrcChannel, packet.GetSourceChannel()),
			sdk.NewAttribute(types.AttributeKeyDstPort, packet.GetDestPort()),
			sdk.NewAttribute(types.AttributeKeyDstChannel, packet.GetDestChannel()),
			sdk.NewAttribute(types.AttributeKeyConnectionID, channel.ConnectionHops[0]),
			sdk.NewAttribute(types.AttributeKeyChannelOrdering, channel.Ordering.String()),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// emitChannelClosedEvent emits a channel closed event.
func emitChannelClosedEvent(ctx sdk.Context, packet exported.PacketI, channel types.Channel) {
	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeChannelClosed,
			sdk.NewAttribute(types.AttributeKeyPortID, packet.GetSourcePort()),
			sdk.NewAttribute(types.AttributeKeyChannelID, packet.GetSourceChannel()),
			sdk.NewAttribute(types.AttributeCounterpartyPortID, channel.Counterparty.PortId),
			sdk.NewAttribute(types.AttributeCounterpartyChannelID, channel.Counterparty.ChannelId),
			sdk.NewAttribute(types.AttributeKeyConnectionID, channel.ConnectionHops[0]),
			sdk.NewAttribute(types.AttributeKeyChannelOrdering, channel.Ordering.String()),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// EmitChannelUpgradeInitEvent emits a channel upgrade init event
func EmitChannelUpgradeInitEvent(ctx sdk.Context, portID string, channelID string, currentChannel types.Channel, upgrade types.Upgrade) {
	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeChannelUpgradeInit,
			sdk.NewAttribute(types.AttributeKeyPortID, portID),
			sdk.NewAttribute(types.AttributeKeyChannelID, channelID),
			sdk.NewAttribute(types.AttributeCounterpartyPortID, currentChannel.Counterparty.PortId),
			sdk.NewAttribute(types.AttributeCounterpartyChannelID, currentChannel.Counterparty.ChannelId),
			sdk.NewAttribute(types.AttributeKeyConnectionHops, upgrade.Fields.ConnectionHops[0]),
			sdk.NewAttribute(types.AttributeVersion, upgrade.Fields.Version),
			sdk.NewAttribute(types.AttributeKeyOrdering, upgrade.Fields.Ordering.String()),
			sdk.NewAttribute(types.AttributeKeyUpgradeSequence, fmt.Sprintf("%d", currentChannel.UpgradeSequence)),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// EmitChannelUpgradeTryEvent emits a channel upgrade try event
func EmitChannelUpgradeTryEvent(ctx sdk.Context, portID string, channelID string, currentChannel types.Channel, upgrade types.Upgrade) {
	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeChannelUpgradeTry,
			sdk.NewAttribute(types.AttributeKeyPortID, portID),
			sdk.NewAttribute(types.AttributeKeyChannelID, channelID),
			sdk.NewAttribute(types.AttributeCounterpartyPortID, currentChannel.Counterparty.PortId),
			sdk.NewAttribute(types.AttributeCounterpartyChannelID, currentChannel.Counterparty.ChannelId),
			sdk.NewAttribute(types.AttributeKeyConnectionHops, upgrade.Fields.ConnectionHops[0]),
			sdk.NewAttribute(types.AttributeVersion, upgrade.Fields.Version),
			sdk.NewAttribute(types.AttributeKeyOrdering, upgrade.Fields.Ordering.String()),
			sdk.NewAttribute(types.AttributeKeyUpgradeSequence, fmt.Sprintf("%d", currentChannel.UpgradeSequence)),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// EmitChannelUpgradeAckEvent emits a channel upgrade ack event
func EmitChannelUpgradeAckEvent(ctx sdk.Context, portID string, channelID string, currentChannel types.Channel, upgrade types.Upgrade) {
	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeChannelUpgradeAck,
			sdk.NewAttribute(types.AttributeKeyPortID, portID),
			sdk.NewAttribute(types.AttributeKeyChannelID, channelID),
			sdk.NewAttribute(types.AttributeCounterpartyPortID, currentChannel.Counterparty.PortId),
			sdk.NewAttribute(types.AttributeCounterpartyChannelID, currentChannel.Counterparty.ChannelId),
			sdk.NewAttribute(types.AttributeKeyConnectionHops, upgrade.Fields.ConnectionHops[0]),
			sdk.NewAttribute(types.AttributeVersion, upgrade.Fields.Version),
			sdk.NewAttribute(types.AttributeKeyOrdering, upgrade.Fields.Ordering.String()),
			sdk.NewAttribute(types.AttributeKeyUpgradeSequence, fmt.Sprintf("%d", currentChannel.UpgradeSequence)),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// EmitChannelUpgradeConfirmEvent emits a channel upgrade confirm event
func EmitChannelUpgradeConfirmEvent(ctx sdk.Context, portID, channelID string, currentChannel types.Channel) {
	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeChannelUpgradeConfirm,
			sdk.NewAttribute(types.AttributeKeyPortID, portID),
			sdk.NewAttribute(types.AttributeKeyChannelID, channelID),
			sdk.NewAttribute(types.AttributeKeyChannelState, currentChannel.State.String()),
			sdk.NewAttribute(types.AttributeCounterpartyPortID, currentChannel.Counterparty.PortId),
			sdk.NewAttribute(types.AttributeCounterpartyChannelID, currentChannel.Counterparty.ChannelId),
			sdk.NewAttribute(types.AttributeKeyUpgradeSequence, fmt.Sprintf("%d", currentChannel.UpgradeSequence)),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// EmitChannelUpgradeOpenEvent emits a channel upgrade open event
func EmitChannelUpgradeOpenEvent(ctx sdk.Context, portID string, channelID string, currentChannel types.Channel) {
	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeChannelUpgradeOpen,
			sdk.NewAttribute(types.AttributeKeyPortID, portID),
			sdk.NewAttribute(types.AttributeKeyChannelID, channelID),
			sdk.NewAttribute(types.AttributeKeyChannelState, currentChannel.State.String()),
			sdk.NewAttribute(types.AttributeCounterpartyPortID, currentChannel.Counterparty.PortId),
			sdk.NewAttribute(types.AttributeCounterpartyChannelID, currentChannel.Counterparty.ChannelId),
			sdk.NewAttribute(types.AttributeKeyConnectionHops, currentChannel.ConnectionHops[0]),
			sdk.NewAttribute(types.AttributeVersion, currentChannel.Version),
			sdk.NewAttribute(types.AttributeKeyOrdering, currentChannel.Ordering.String()),
			sdk.NewAttribute(types.AttributeKeyUpgradeSequence, fmt.Sprintf("%d", currentChannel.UpgradeSequence)),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// EmitChannelUpgradeTimeoutEvent emits an upgrade timeout event.
func EmitChannelUpgradeTimeoutEvent(ctx sdk.Context, portID string, channelID string, currentChannel types.Channel, upgrade types.Upgrade) {
	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeChannelUpgradeTimeout,
			sdk.NewAttribute(types.AttributeKeyPortID, portID),
			sdk.NewAttribute(types.AttributeKeyChannelID, channelID),
			sdk.NewAttribute(types.AttributeCounterpartyPortID, currentChannel.Counterparty.PortId),
			sdk.NewAttribute(types.AttributeCounterpartyChannelID, currentChannel.Counterparty.ChannelId),
			sdk.NewAttribute(types.AttributeKeyConnectionHops, upgrade.Fields.ConnectionHops[0]),
			sdk.NewAttribute(types.AttributeVersion, upgrade.Fields.Version),
			sdk.NewAttribute(types.AttributeKeyOrdering, upgrade.Fields.Ordering.String()),
			sdk.NewAttribute(types.AttributeKeyUpgradeTimeoutHeight, upgrade.Timeout.Height.String()),
			sdk.NewAttribute(types.AttributeKeyUpgradeTimeoutTimestamp, fmt.Sprintf("%d", upgrade.Timeout.Timestamp)),
			sdk.NewAttribute(types.AttributeKeyUpgradeSequence, fmt.Sprintf("%d", currentChannel.UpgradeSequence)),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// EmitErrorReceiptEvent emits an error receipt event
func EmitErrorReceiptEvent(ctx sdk.Context, portID string, channelID string, currentChannel types.Channel, err error) {
	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeChannelUpgradeError,
			sdk.NewAttribute(types.AttributeKeyPortID, portID),
			sdk.NewAttribute(types.AttributeKeyChannelID, channelID),
			sdk.NewAttribute(types.AttributeCounterpartyPortID, currentChannel.Counterparty.PortId),
			sdk.NewAttribute(types.AttributeCounterpartyChannelID, currentChannel.Counterparty.ChannelId),
			sdk.NewAttribute(types.AttributeKeyUpgradeSequence, fmt.Sprintf("%d", currentChannel.UpgradeSequence)),
			sdk.NewAttribute(types.AttributeKeyErrorReceipt, err.Error()),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// EmitChannelUpgradeCancelEvent emits an upgraded cancelled event.
func EmitChannelUpgradeCancelEvent(ctx sdk.Context, portID string, channelID string, currentChannel types.Channel, upgrade types.Upgrade) {
	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeChannelUpgradeCancel,
			sdk.NewAttribute(types.AttributeKeyPortID, portID),
			sdk.NewAttribute(types.AttributeKeyChannelID, channelID),
			sdk.NewAttribute(types.AttributeCounterpartyPortID, currentChannel.Counterparty.PortId),
			sdk.NewAttribute(types.AttributeCounterpartyChannelID, currentChannel.Counterparty.ChannelId),
			sdk.NewAttribute(types.AttributeKeyConnectionHops, upgrade.Fields.ConnectionHops[0]),
			sdk.NewAttribute(types.AttributeVersion, upgrade.Fields.Version),
			sdk.NewAttribute(types.AttributeKeyOrdering, upgrade.Fields.Ordering.String()),
			sdk.NewAttribute(types.AttributeKeyUpgradeSequence, fmt.Sprintf("%d", currentChannel.UpgradeSequence)),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// emitChannelFlushCompleteEvents emits an flushing event.
func emitChannelFlushCompleteEvent(ctx sdk.Context, portID string, channelID string, currentChannel types.Channel) {
	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeChannelFlushComplete,
			sdk.NewAttribute(types.AttributeKeyPortID, portID),
			sdk.NewAttribute(types.AttributeKeyChannelID, channelID),
			sdk.NewAttribute(types.AttributeCounterpartyPortID, currentChannel.Counterparty.PortId),
			sdk.NewAttribute(types.AttributeCounterpartyChannelID, currentChannel.Counterparty.ChannelId),
			sdk.NewAttribute(types.AttributeKeyChannelState, currentChannel.State.String()),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}
