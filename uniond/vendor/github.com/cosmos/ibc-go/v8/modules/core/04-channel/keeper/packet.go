package keeper

import (
	"bytes"
	"slices"
	"strconv"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"

	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	"github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// SendPacket is called by a module in order to send an IBC packet on a channel.
// The packet sequence generated for the packet to be sent is returned. An error
// is returned if one occurs.
func (k Keeper) SendPacket(
	ctx sdk.Context,
	channelCap *capabilitytypes.Capability,
	sourcePort string,
	sourceChannel string,
	timeoutHeight clienttypes.Height,
	timeoutTimestamp uint64,
	data []byte,
) (uint64, error) {
	channel, found := k.GetChannel(ctx, sourcePort, sourceChannel)
	if !found {
		return 0, errorsmod.Wrap(types.ErrChannelNotFound, sourceChannel)
	}

	if channel.State != types.OPEN {
		return 0, errorsmod.Wrapf(types.ErrInvalidChannelState, "channel is not OPEN (got %s)", channel.State)
	}

	if !k.scopedKeeper.AuthenticateCapability(ctx, channelCap, host.ChannelCapabilityPath(sourcePort, sourceChannel)) {
		return 0, errorsmod.Wrapf(types.ErrChannelCapabilityNotFound, "caller does not own capability for channel, port ID (%s) channel ID (%s)", sourcePort, sourceChannel)
	}

	sequence, found := k.GetNextSequenceSend(ctx, sourcePort, sourceChannel)
	if !found {
		return 0, errorsmod.Wrapf(
			types.ErrSequenceSendNotFound,
			"source port: %s, source channel: %s", sourcePort, sourceChannel,
		)
	}

	// construct packet from given fields and channel state
	packet := types.NewPacket(data, sequence, sourcePort, sourceChannel,
		channel.Counterparty.PortId, channel.Counterparty.ChannelId, timeoutHeight, timeoutTimestamp)

	if err := packet.ValidateBasic(); err != nil {
		return 0, errorsmod.Wrap(err, "constructed packet failed basic validation")
	}

	connectionEnd, found := k.connectionKeeper.GetConnection(ctx, channel.ConnectionHops[0])
	if !found {
		return 0, errorsmod.Wrap(connectiontypes.ErrConnectionNotFound, channel.ConnectionHops[0])
	}

	clientState, found := k.clientKeeper.GetClientState(ctx, connectionEnd.GetClientID())
	if !found {
		return 0, clienttypes.ErrClientNotFound
	}

	// prevent accidental sends with clients that cannot be updated
	if status := k.clientKeeper.GetClientStatus(ctx, clientState, connectionEnd.GetClientID()); status != exported.Active {
		return 0, errorsmod.Wrapf(clienttypes.ErrClientNotActive, "cannot send packet using client (%s) with status %s", connectionEnd.GetClientID(), status)
	}

	latestHeight := clientState.GetLatestHeight()
	latestTimestamp, err := k.connectionKeeper.GetTimestampAtHeight(ctx, connectionEnd, latestHeight)
	if err != nil {
		return 0, err
	}

	// check if packet is timed out on the receiving chain
	timeout := types.NewTimeout(packet.GetTimeoutHeight().(clienttypes.Height), packet.GetTimeoutTimestamp())
	if timeout.Elapsed(latestHeight.(clienttypes.Height), latestTimestamp) {
		return 0, errorsmod.Wrap(timeout.ErrTimeoutElapsed(latestHeight.(clienttypes.Height), latestTimestamp), "invalid packet timeout")
	}

	commitment := types.CommitPacket(k.cdc, packet)

	k.SetNextSequenceSend(ctx, sourcePort, sourceChannel, sequence+1)
	k.SetPacketCommitment(ctx, sourcePort, sourceChannel, packet.GetSequence(), commitment)

	emitSendPacketEvent(ctx, packet, channel, timeoutHeight)

	k.Logger(ctx).Info(
		"packet sent",
		"sequence", strconv.FormatUint(packet.GetSequence(), 10),
		"src_port", sourcePort,
		"src_channel", sourceChannel,
		"dst_port", packet.GetDestPort(),
		"dst_channel", packet.GetDestChannel(),
	)

	return packet.GetSequence(), nil
}

// RecvPacket is called by a module in order to receive & process an IBC packet
// sent on the corresponding channel end on the counterparty chain.
func (k Keeper) RecvPacket(
	ctx sdk.Context,
	chanCap *capabilitytypes.Capability,
	packet exported.PacketI,
	proof []byte,
	proofHeight exported.Height,
) error {
	channel, found := k.GetChannel(ctx, packet.GetDestPort(), packet.GetDestChannel())
	if !found {
		return errorsmod.Wrap(types.ErrChannelNotFound, packet.GetDestChannel())
	}

	if !slices.Contains([]types.State{types.OPEN, types.FLUSHING, types.FLUSHCOMPLETE}, channel.State) {
		return errorsmod.Wrapf(types.ErrInvalidChannelState, "expected channel state to be one of [%s, %s, %s], but got %s", types.OPEN, types.FLUSHING, types.FLUSHCOMPLETE, channel.State)
	}

	// If counterpartyUpgrade is stored we need to ensure that the
	// packet sequence is < counterparty next sequence send. If the
	// counterparty is implemented correctly, this may only occur
	// when we are in FLUSHCOMPLETE and the counterparty has already
	// completed the channel upgrade.
	counterpartyUpgrade, found := k.GetCounterpartyUpgrade(ctx, packet.GetDestPort(), packet.GetDestChannel())
	if found {
		counterpartyNextSequenceSend := counterpartyUpgrade.NextSequenceSend
		if packet.GetSequence() >= counterpartyNextSequenceSend {
			return errorsmod.Wrapf(types.ErrInvalidPacket, "cannot flush packet at sequence greater than or equal to counterparty next sequence send (%d) ≥ (%d).", packet.GetSequence(), counterpartyNextSequenceSend)
		}
	}

	// Authenticate capability to ensure caller has authority to receive packet on this channel
	capName := host.ChannelCapabilityPath(packet.GetDestPort(), packet.GetDestChannel())
	if !k.scopedKeeper.AuthenticateCapability(ctx, chanCap, capName) {
		return errorsmod.Wrapf(
			types.ErrInvalidChannelCapability,
			"channel capability failed authentication for capability name %s", capName,
		)
	}

	// packet must come from the channel's counterparty
	if packet.GetSourcePort() != channel.Counterparty.PortId {
		return errorsmod.Wrapf(
			types.ErrInvalidPacket,
			"packet source port doesn't match the counterparty's port (%s ≠ %s)", packet.GetSourcePort(), channel.Counterparty.PortId,
		)
	}

	if packet.GetSourceChannel() != channel.Counterparty.ChannelId {
		return errorsmod.Wrapf(
			types.ErrInvalidPacket,
			"packet source channel doesn't match the counterparty's channel (%s ≠ %s)", packet.GetSourceChannel(), channel.Counterparty.ChannelId,
		)
	}

	// Connection must be OPEN to receive a packet. It is possible for connection to not yet be open if packet was
	// sent optimistically before connection and channel handshake completed. However, to receive a packet,
	// connection and channel must both be open
	connectionEnd, found := k.connectionKeeper.GetConnection(ctx, channel.ConnectionHops[0])
	if !found {
		return errorsmod.Wrap(connectiontypes.ErrConnectionNotFound, channel.ConnectionHops[0])
	}

	if connectionEnd.GetState() != int32(connectiontypes.OPEN) {
		return errorsmod.Wrapf(
			connectiontypes.ErrInvalidConnectionState,
			"connection state is not OPEN (got %s)", connectiontypes.State(connectionEnd.GetState()).String(),
		)
	}

	// check if packet timed out by comparing it with the latest height of the chain
	selfHeight, selfTimestamp := clienttypes.GetSelfHeight(ctx), uint64(ctx.BlockTime().UnixNano())
	timeout := types.NewTimeout(packet.GetTimeoutHeight().(clienttypes.Height), packet.GetTimeoutTimestamp())
	if timeout.Elapsed(selfHeight, selfTimestamp) {
		return errorsmod.Wrap(timeout.ErrTimeoutElapsed(selfHeight, selfTimestamp), "packet timeout elapsed")
	}

	commitment := types.CommitPacket(k.cdc, packet)

	// verify that the counterparty did commit to sending this packet
	if err := k.connectionKeeper.VerifyPacketCommitment(
		ctx, connectionEnd, proofHeight, proof,
		packet.GetSourcePort(), packet.GetSourceChannel(), packet.GetSequence(),
		commitment,
	); err != nil {
		return errorsmod.Wrap(err, "couldn't verify counterparty packet commitment")
	}

	// REPLAY PROTECTION: The recvStartSequence will prevent historical proofs from allowing replay
	// attacks on packets processed in previous lifecycles of a channel. After a successful channel
	// upgrade all packets under the recvStartSequence will have been processed and thus should be
	// rejected.
	recvStartSequence, _ := k.GetRecvStartSequence(ctx, packet.GetDestPort(), packet.GetDestChannel())
	if packet.GetSequence() < recvStartSequence {
		return errorsmod.Wrap(types.ErrPacketReceived, "packet already processed in previous channel upgrade")
	}

	switch channel.Ordering {
	case types.UNORDERED:
		// REPLAY PROTECTION: Packet receipts will indicate that a packet has already been received
		// on unordered channels. Packet receipts must not be pruned, unless it has been marked stale
		// by the increase of the recvStartSequence.
		_, found := k.GetPacketReceipt(ctx, packet.GetDestPort(), packet.GetDestChannel(), packet.GetSequence())
		if found {
			emitRecvPacketEvent(ctx, packet, channel)
			// This error indicates that the packet has already been relayed. Core IBC will
			// treat this error as a no-op in order to prevent an entire relay transaction
			// from failing and consuming unnecessary fees.
			return types.ErrNoOpMsg
		}

		// All verification complete, update state
		// For unordered channels we must set the receipt so it can be verified on the other side.
		// This receipt does not contain any data, since the packet has not yet been processed,
		// it's just a single store key set to a single byte to indicate that the packet has been received
		k.SetPacketReceipt(ctx, packet.GetDestPort(), packet.GetDestChannel(), packet.GetSequence())

	case types.ORDERED:
		// check if the packet is being received in order
		nextSequenceRecv, found := k.GetNextSequenceRecv(ctx, packet.GetDestPort(), packet.GetDestChannel())
		if !found {
			return errorsmod.Wrapf(
				types.ErrSequenceReceiveNotFound,
				"destination port: %s, destination channel: %s", packet.GetDestPort(), packet.GetDestChannel(),
			)
		}

		if packet.GetSequence() < nextSequenceRecv {
			emitRecvPacketEvent(ctx, packet, channel)
			// This error indicates that the packet has already been relayed. Core IBC will
			// treat this error as a no-op in order to prevent an entire relay transaction
			// from failing and consuming unnecessary fees.
			return types.ErrNoOpMsg
		}

		// REPLAY PROTECTION: Ordered channels require packets to be received in a strict order.
		// Any out of order or previously received packets are rejected.
		if packet.GetSequence() != nextSequenceRecv {
			return errorsmod.Wrapf(
				types.ErrPacketSequenceOutOfOrder,
				"packet sequence ≠ next receive sequence (%d ≠ %d)", packet.GetSequence(), nextSequenceRecv,
			)
		}

		// All verification complete, update state
		// In ordered case, we must increment nextSequenceRecv
		nextSequenceRecv++

		// incrementing nextSequenceRecv and storing under this chain's channelEnd identifiers
		// Since this is the receiving chain, our channelEnd is packet's destination port and channel
		k.SetNextSequenceRecv(ctx, packet.GetDestPort(), packet.GetDestChannel(), nextSequenceRecv)
	}

	// log that a packet has been received & executed
	k.Logger(ctx).Info(
		"packet received",
		"sequence", strconv.FormatUint(packet.GetSequence(), 10),
		"src_port", packet.GetSourcePort(),
		"src_channel", packet.GetSourceChannel(),
		"dst_port", packet.GetDestPort(),
		"dst_channel", packet.GetDestChannel(),
	)

	// emit an event that the relayer can query for
	emitRecvPacketEvent(ctx, packet, channel)

	return nil
}

// WriteAcknowledgement writes the packet execution acknowledgement to the state,
// which will be verified by the counterparty chain using AcknowledgePacket.
//
// CONTRACT:
//
// 1) For synchronous execution, this function is be called in the IBC handler .
// For async handling, it needs to be called directly by the module which originally
// processed the packet.
//
// 2) Assumes that packet receipt has been written (unordered), or nextSeqRecv was incremented (ordered)
// previously by RecvPacket.
func (k Keeper) WriteAcknowledgement(
	ctx sdk.Context,
	chanCap *capabilitytypes.Capability,
	packet exported.PacketI,
	acknowledgement exported.Acknowledgement,
) error {
	channel, found := k.GetChannel(ctx, packet.GetDestPort(), packet.GetDestChannel())
	if !found {
		return errorsmod.Wrap(types.ErrChannelNotFound, packet.GetDestChannel())
	}

	if !slices.Contains([]types.State{types.OPEN, types.FLUSHING, types.FLUSHCOMPLETE}, channel.State) {
		return errorsmod.Wrapf(types.ErrInvalidChannelState, "expected one of [%s, %s, %s], got %s", types.OPEN, types.FLUSHING, types.FLUSHCOMPLETE, channel.State)
	}

	// Authenticate capability to ensure caller has authority to receive packet on this channel
	capName := host.ChannelCapabilityPath(packet.GetDestPort(), packet.GetDestChannel())
	if !k.scopedKeeper.AuthenticateCapability(ctx, chanCap, capName) {
		return errorsmod.Wrapf(
			types.ErrInvalidChannelCapability,
			"channel capability failed authentication for capability name %s", capName,
		)
	}

	// NOTE: IBC app modules might have written the acknowledgement synchronously on
	// the OnRecvPacket callback so we need to check if the acknowledgement is already
	// set on the store and return an error if so.
	if k.HasPacketAcknowledgement(ctx, packet.GetDestPort(), packet.GetDestChannel(), packet.GetSequence()) {
		return types.ErrAcknowledgementExists
	}

	if acknowledgement == nil {
		return errorsmod.Wrap(types.ErrInvalidAcknowledgement, "acknowledgement cannot be nil")
	}

	bz := acknowledgement.Acknowledgement()
	if len(bz) == 0 {
		return errorsmod.Wrap(types.ErrInvalidAcknowledgement, "acknowledgement cannot be empty")
	}

	// set the acknowledgement so that it can be verified on the other side
	k.SetPacketAcknowledgement(
		ctx, packet.GetDestPort(), packet.GetDestChannel(), packet.GetSequence(),
		types.CommitAcknowledgement(bz),
	)

	// log that a packet acknowledgement has been written
	k.Logger(ctx).Info(
		"acknowledgement written",
		"sequence", strconv.FormatUint(packet.GetSequence(), 10),
		"src_port", packet.GetSourcePort(),
		"src_channel", packet.GetSourceChannel(),
		"dst_port", packet.GetDestPort(),
		"dst_channel", packet.GetDestChannel(),
	)

	emitWriteAcknowledgementEvent(ctx, packet, channel, bz)

	return nil
}

// AcknowledgePacket is called by a module to process the acknowledgement of a
// packet previously sent by the calling module on a channel to a counterparty
// module on the counterparty chain. Its intended usage is within the ante
// handler. AcknowledgePacket will clean up the packet commitment,
// which is no longer necessary since the packet has been received and acted upon.
// It will also increment NextSequenceAck in case of ORDERED channels.
func (k Keeper) AcknowledgePacket(
	ctx sdk.Context,
	chanCap *capabilitytypes.Capability,
	packet exported.PacketI,
	acknowledgement []byte,
	proof []byte,
	proofHeight exported.Height,
) error {
	channel, found := k.GetChannel(ctx, packet.GetSourcePort(), packet.GetSourceChannel())
	if !found {
		return errorsmod.Wrapf(
			types.ErrChannelNotFound,
			"port ID (%s) channel ID (%s)", packet.GetSourcePort(), packet.GetSourceChannel(),
		)
	}

	if !slices.Contains([]types.State{types.OPEN, types.FLUSHING}, channel.State) {
		return errorsmod.Wrapf(types.ErrInvalidChannelState, "packets cannot be acknowledged on channel with state (%s)", channel.State)
	}

	// Authenticate capability to ensure caller has authority to receive packet on this channel
	capName := host.ChannelCapabilityPath(packet.GetSourcePort(), packet.GetSourceChannel())
	if !k.scopedKeeper.AuthenticateCapability(ctx, chanCap, capName) {
		return errorsmod.Wrapf(
			types.ErrInvalidChannelCapability,
			"channel capability failed authentication for capability name %s", capName,
		)
	}

	// packet must have been sent to the channel's counterparty
	if packet.GetDestPort() != channel.Counterparty.PortId {
		return errorsmod.Wrapf(
			types.ErrInvalidPacket,
			"packet destination port doesn't match the counterparty's port (%s ≠ %s)", packet.GetDestPort(), channel.Counterparty.PortId,
		)
	}

	if packet.GetDestChannel() != channel.Counterparty.ChannelId {
		return errorsmod.Wrapf(
			types.ErrInvalidPacket,
			"packet destination channel doesn't match the counterparty's channel (%s ≠ %s)", packet.GetDestChannel(), channel.Counterparty.ChannelId,
		)
	}

	connectionEnd, found := k.connectionKeeper.GetConnection(ctx, channel.ConnectionHops[0])
	if !found {
		return errorsmod.Wrap(connectiontypes.ErrConnectionNotFound, channel.ConnectionHops[0])
	}

	if connectionEnd.GetState() != int32(connectiontypes.OPEN) {
		return errorsmod.Wrapf(
			connectiontypes.ErrInvalidConnectionState,
			"connection state is not OPEN (got %s)", connectiontypes.State(connectionEnd.GetState()).String(),
		)
	}

	commitment := k.GetPacketCommitment(ctx, packet.GetSourcePort(), packet.GetSourceChannel(), packet.GetSequence())

	if len(commitment) == 0 {
		emitAcknowledgePacketEvent(ctx, packet, channel)
		// This error indicates that the acknowledgement has already been relayed
		// or there is a misconfigured relayer attempting to prove an acknowledgement
		// for a packet never sent. Core IBC will treat this error as a no-op in order to
		// prevent an entire relay transaction from failing and consuming unnecessary fees.
		return types.ErrNoOpMsg
	}

	packetCommitment := types.CommitPacket(k.cdc, packet)

	// verify we sent the packet and haven't cleared it out yet
	if !bytes.Equal(commitment, packetCommitment) {
		return errorsmod.Wrapf(types.ErrInvalidPacket, "commitment bytes are not equal: got (%v), expected (%v)", packetCommitment, commitment)
	}

	if err := k.connectionKeeper.VerifyPacketAcknowledgement(
		ctx, connectionEnd, proofHeight, proof, packet.GetDestPort(), packet.GetDestChannel(),
		packet.GetSequence(), acknowledgement,
	); err != nil {
		return err
	}

	// assert packets acknowledged in order
	if channel.Ordering == types.ORDERED {
		nextSequenceAck, found := k.GetNextSequenceAck(ctx, packet.GetSourcePort(), packet.GetSourceChannel())
		if !found {
			return errorsmod.Wrapf(
				types.ErrSequenceAckNotFound,
				"source port: %s, source channel: %s", packet.GetSourcePort(), packet.GetSourceChannel(),
			)
		}

		if packet.GetSequence() != nextSequenceAck {
			return errorsmod.Wrapf(
				types.ErrPacketSequenceOutOfOrder,
				"packet sequence ≠ next ack sequence (%d ≠ %d)", packet.GetSequence(), nextSequenceAck,
			)
		}

		// All verification complete, in the case of ORDERED channels we must increment nextSequenceAck
		nextSequenceAck++

		// incrementing NextSequenceAck and storing under this chain's channelEnd identifiers
		// Since this is the original sending chain, our channelEnd is packet's source port and channel
		k.SetNextSequenceAck(ctx, packet.GetSourcePort(), packet.GetSourceChannel(), nextSequenceAck)

	}

	// Delete packet commitment, since the packet has been acknowledged, the commitement is no longer necessary
	k.deletePacketCommitment(ctx, packet.GetSourcePort(), packet.GetSourceChannel(), packet.GetSequence())

	// log that a packet has been acknowledged
	k.Logger(ctx).Info(
		"packet acknowledged",
		"sequence", strconv.FormatUint(packet.GetSequence(), 10),
		"src_port", packet.GetSourcePort(),
		"src_channel", packet.GetSourceChannel(),
		"dst_port", packet.GetDestPort(),
		"dst_channel", packet.GetDestChannel(),
	)

	// emit an event marking that we have processed the acknowledgement
	emitAcknowledgePacketEvent(ctx, packet, channel)

	// if an upgrade is in progress, handling packet flushing and update channel state appropriately
	if channel.State == types.FLUSHING {
		// counterparty upgrade is written in the OnChanUpgradeAck step.
		counterpartyUpgrade, found := k.GetCounterpartyUpgrade(ctx, packet.GetSourcePort(), packet.GetSourceChannel())
		if found {
			timeout := counterpartyUpgrade.Timeout
			selfHeight, selfTimestamp := clienttypes.GetSelfHeight(ctx), uint64(ctx.BlockTime().UnixNano())

			if timeout.Elapsed(selfHeight, selfTimestamp) {
				// packet flushing timeout has expired, abort the upgrade and return nil,
				// committing an error receipt to state, restoring the channel and successfully acknowledging the packet.
				k.MustAbortUpgrade(ctx, packet.GetSourcePort(), packet.GetSourceChannel(), timeout.ErrTimeoutElapsed(selfHeight, selfTimestamp))
				return nil
			}

			// set the channel state to flush complete if all packets have been acknowledged/flushed.
			if !k.HasInflightPackets(ctx, packet.GetSourcePort(), packet.GetSourceChannel()) {
				channel.State = types.FLUSHCOMPLETE
				k.SetChannel(ctx, packet.GetSourcePort(), packet.GetSourceChannel(), channel)
				emitChannelFlushCompleteEvent(ctx, packet.GetSourcePort(), packet.GetSourceChannel(), channel)
			}
		}
	}

	return nil
}
