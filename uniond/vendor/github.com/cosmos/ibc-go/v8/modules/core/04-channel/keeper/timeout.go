package keeper

import (
	"bytes"
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

// TimeoutPacket is called by a module which originally attempted to send a
// packet to a counterparty module, where the timeout height has passed on the
// counterparty chain without the packet being committed, to prove that the
// packet can no longer be executed and to allow the calling module to safely
// perform appropriate state transitions. Its intended usage is within the
// ante handler.
func (k Keeper) TimeoutPacket(
	ctx sdk.Context,
	packet exported.PacketI,
	proof []byte,
	proofHeight exported.Height,
	nextSequenceRecv uint64,
) error {
	channel, found := k.GetChannel(ctx, packet.GetSourcePort(), packet.GetSourceChannel())
	if !found {
		return errorsmod.Wrapf(
			types.ErrChannelNotFound,
			"port ID (%s) channel ID (%s)", packet.GetSourcePort(), packet.GetSourceChannel(),
		)
	}

	// NOTE: TimeoutPacket is called by the AnteHandler which acts upon the packet.Route(),
	// so the capability authentication can be omitted here

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
		return errorsmod.Wrap(
			connectiontypes.ErrConnectionNotFound,
			channel.ConnectionHops[0],
		)
	}

	// check that timeout height or timeout timestamp has passed on the other end
	proofTimestamp, err := k.connectionKeeper.GetTimestampAtHeight(ctx, connectionEnd, proofHeight)
	if err != nil {
		return err
	}

	timeout := types.NewTimeout(packet.GetTimeoutHeight().(clienttypes.Height), packet.GetTimeoutTimestamp())
	if !timeout.Elapsed(proofHeight.(clienttypes.Height), proofTimestamp) {
		return errorsmod.Wrap(timeout.ErrTimeoutNotReached(proofHeight.(clienttypes.Height), proofTimestamp), "packet timeout not reached")
	}

	commitment := k.GetPacketCommitment(ctx, packet.GetSourcePort(), packet.GetSourceChannel(), packet.GetSequence())

	if len(commitment) == 0 {
		emitTimeoutPacketEvent(ctx, packet, channel)
		// This error indicates that the timeout has already been relayed
		// or there is a misconfigured relayer attempting to prove a timeout
		// for a packet never sent. Core IBC will treat this error as a no-op in order to
		// prevent an entire relay transaction from failing and consuming unnecessary fees.
		return types.ErrNoOpMsg
	}

	packetCommitment := types.CommitPacket(k.cdc, packet)

	// verify we sent the packet and haven't cleared it out yet
	if !bytes.Equal(commitment, packetCommitment) {
		return errorsmod.Wrapf(types.ErrInvalidPacket, "packet commitment bytes are not equal: got (%v), expected (%v)", commitment, packetCommitment)
	}

	switch channel.Ordering {
	case types.ORDERED:
		// check that packet has not been received
		if nextSequenceRecv > packet.GetSequence() {
			return errorsmod.Wrapf(
				types.ErrPacketReceived,
				"packet already received, next sequence receive > packet sequence (%d > %d)", nextSequenceRecv, packet.GetSequence(),
			)
		}

		// check that the recv sequence is as claimed
		err = k.connectionKeeper.VerifyNextSequenceRecv(
			ctx, connectionEnd, proofHeight, proof,
			packet.GetDestPort(), packet.GetDestChannel(), nextSequenceRecv,
		)
	case types.UNORDERED:
		err = k.connectionKeeper.VerifyPacketReceiptAbsence(
			ctx, connectionEnd, proofHeight, proof,
			packet.GetDestPort(), packet.GetDestChannel(), packet.GetSequence(),
		)
	default:
		panic(errorsmod.Wrapf(types.ErrInvalidChannelOrdering, channel.Ordering.String()))
	}

	if err != nil {
		return err
	}

	// NOTE: the remaining code is located in the TimeoutExecuted function
	return nil
}

// TimeoutExecuted deletes the commitment send from this chain after it verifies timeout.
// If the timed-out packet came from an ORDERED channel then this channel will be closed.
// If the channel is in the FLUSHING state and there is a counterparty upgrade, then the
// upgrade will be aborted if the upgrade has timed out. Otherwise, if there are no more inflight packets,
// then the channel will be set to the FLUSHCOMPLETE state.
//
// CONTRACT: this function must be called in the IBC handler
func (k Keeper) TimeoutExecuted(
	ctx sdk.Context,
	chanCap *capabilitytypes.Capability,
	packet exported.PacketI,
) error {
	channel, found := k.GetChannel(ctx, packet.GetSourcePort(), packet.GetSourceChannel())
	if !found {
		return errorsmod.Wrapf(types.ErrChannelNotFound, "port ID (%s) channel ID (%s)", packet.GetSourcePort(), packet.GetSourceChannel())
	}

	capName := host.ChannelCapabilityPath(packet.GetSourcePort(), packet.GetSourceChannel())
	if !k.scopedKeeper.AuthenticateCapability(ctx, chanCap, capName) {
		return errorsmod.Wrapf(
			types.ErrChannelCapabilityNotFound,
			"caller does not own capability for channel with capability name %s", capName,
		)
	}

	k.deletePacketCommitment(ctx, packet.GetSourcePort(), packet.GetSourceChannel(), packet.GetSequence())

	// if an upgrade is in progress, handling packet flushing and update channel state appropriately
	if channel.State == types.FLUSHING && channel.Ordering == types.UNORDERED {
		counterpartyUpgrade, found := k.GetCounterpartyUpgrade(ctx, packet.GetSourcePort(), packet.GetSourceChannel())
		// once we have received the counterparty timeout in the channel UpgradeAck or UpgradeConfirm handshake steps
		// then we can move to flushing complete if the timeout has not passed and there are no in-flight packets
		if found {
			timeout := counterpartyUpgrade.Timeout
			selfHeight, selfTimestamp := clienttypes.GetSelfHeight(ctx), uint64(ctx.BlockTime().UnixNano())

			if timeout.Elapsed(selfHeight, selfTimestamp) {
				// packet flushing timeout has expired, abort the upgrade and return nil,
				// committing an error receipt to state, restoring the channel and successfully timing out the packet.
				k.MustAbortUpgrade(ctx, packet.GetSourcePort(), packet.GetSourceChannel(), timeout.ErrTimeoutElapsed(selfHeight, selfTimestamp))
			} else if !k.HasInflightPackets(ctx, packet.GetSourcePort(), packet.GetSourceChannel()) {
				// set the channel state to flush complete if all packets have been flushed.
				channel.State = types.FLUSHCOMPLETE
				k.SetChannel(ctx, packet.GetSourcePort(), packet.GetSourceChannel(), channel)
				emitChannelFlushCompleteEvent(ctx, packet.GetSourcePort(), packet.GetSourceChannel(), channel)
			}
		}
	}

	if channel.Ordering == types.ORDERED {
		// NOTE: if the channel is ORDERED and a packet is timed out in FLUSHING state then
		// the upgrade is aborted and the channel is set to CLOSED.
		if channel.State == types.FLUSHING {
			// an error receipt is written to state and the channel is restored to OPEN
			k.MustAbortUpgrade(ctx, packet.GetSourcePort(), packet.GetSourceChannel(), errorsmod.Wrap(types.ErrTimeoutElapsed, "packet timeout elapsed on ORDERED channel"))
		}

		channel.State = types.CLOSED
		k.SetChannel(ctx, packet.GetSourcePort(), packet.GetSourceChannel(), channel)
		emitChannelClosedEvent(ctx, packet, channel)
	}

	k.Logger(ctx).Info(
		"packet timed-out",
		"sequence", strconv.FormatUint(packet.GetSequence(), 10),
		"src_port", packet.GetSourcePort(),
		"src_channel", packet.GetSourceChannel(),
		"dst_port", packet.GetDestPort(),
		"dst_channel", packet.GetDestChannel(),
	)

	// emit an event marking that we have processed the timeout
	emitTimeoutPacketEvent(ctx, packet, channel)

	return nil
}

// TimeoutOnClose is called by a module in order to prove that the channel to
// which an unreceived packet was addressed has been closed, so the packet will
// never be received (even if the timeoutHeight has not yet been reached).
func (k Keeper) TimeoutOnClose(
	ctx sdk.Context,
	chanCap *capabilitytypes.Capability,
	packet exported.PacketI,
	proof,
	closedProof []byte,
	proofHeight exported.Height,
	nextSequenceRecv uint64,
) error {
	return k.TimeoutOnCloseWithCounterpartyUpgradeSequence(ctx, chanCap, packet, proof, closedProof, proofHeight, nextSequenceRecv, 0)
}

// TimeoutOnCloseWithCounterpartyUpgradeSequence is called by a module in order
// to prove that the channel to which an unreceived packet was addressed has
// been closed, so the packet will never be received (even if the timeoutHeight
// has not yet been reached). The difference with TimeoutOnClose is that it
// accepts an extra argument counterpartyUpgradeSequence that was needed for
// channel upgradability.
//
// This function will be removed in ibc-go v9.0.0 and the API of TimeoutOnClose will be updated.
func (k Keeper) TimeoutOnCloseWithCounterpartyUpgradeSequence(
	ctx sdk.Context,
	chanCap *capabilitytypes.Capability,
	packet exported.PacketI,
	proof,
	closedProof []byte,
	proofHeight exported.Height,
	nextSequenceRecv uint64,
	counterpartyUpgradeSequence uint64,
) error {
	channel, found := k.GetChannel(ctx, packet.GetSourcePort(), packet.GetSourceChannel())
	if !found {
		return errorsmod.Wrapf(types.ErrChannelNotFound, "port ID (%s) channel ID (%s)", packet.GetSourcePort(), packet.GetSourceChannel())
	}

	capName := host.ChannelCapabilityPath(packet.GetSourcePort(), packet.GetSourceChannel())
	if !k.scopedKeeper.AuthenticateCapability(ctx, chanCap, capName) {
		return errorsmod.Wrapf(
			types.ErrInvalidChannelCapability,
			"channel capability failed authentication with capability name %s", capName,
		)
	}

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

	commitment := k.GetPacketCommitment(ctx, packet.GetSourcePort(), packet.GetSourceChannel(), packet.GetSequence())

	if len(commitment) == 0 {
		emitTimeoutPacketEvent(ctx, packet, channel)
		// This error indicates that the timeout has already been relayed
		// or there is a misconfigured relayer attempting to prove a timeout
		// for a packet never sent. Core IBC will treat this error as a no-op in order to
		// prevent an entire relay transaction from failing and consuming unnecessary fees.
		return types.ErrNoOpMsg
	}

	packetCommitment := types.CommitPacket(k.cdc, packet)

	// verify we sent the packet and haven't cleared it out yet
	if !bytes.Equal(commitment, packetCommitment) {
		return errorsmod.Wrapf(types.ErrInvalidPacket, "packet commitment bytes are not equal: got (%v), expected (%v)", commitment, packetCommitment)
	}

	counterpartyHops := []string{connectionEnd.GetCounterparty().GetConnectionID()}

	counterparty := types.NewCounterparty(packet.GetSourcePort(), packet.GetSourceChannel())
	expectedChannel := types.Channel{
		State:           types.CLOSED,
		Ordering:        channel.Ordering,
		Counterparty:    counterparty,
		ConnectionHops:  counterpartyHops,
		Version:         channel.Version,
		UpgradeSequence: counterpartyUpgradeSequence,
	}

	// check that the opposing channel end has closed
	if err := k.connectionKeeper.VerifyChannelState(
		ctx, connectionEnd, proofHeight, closedProof,
		channel.Counterparty.PortId, channel.Counterparty.ChannelId,
		expectedChannel,
	); err != nil {
		return err
	}

	var err error
	switch channel.Ordering {
	case types.ORDERED:
		// check that packet has not been received
		if nextSequenceRecv > packet.GetSequence() {
			return errorsmod.Wrapf(types.ErrInvalidPacket, "packet already received, next sequence receive > packet sequence (%d > %d", nextSequenceRecv, packet.GetSequence())
		}

		// check that the recv sequence is as claimed
		err = k.connectionKeeper.VerifyNextSequenceRecv(
			ctx, connectionEnd, proofHeight, proof,
			packet.GetDestPort(), packet.GetDestChannel(), nextSequenceRecv,
		)
	case types.UNORDERED:
		err = k.connectionKeeper.VerifyPacketReceiptAbsence(
			ctx, connectionEnd, proofHeight, proof,
			packet.GetDestPort(), packet.GetDestChannel(), packet.GetSequence(),
		)
	default:
		panic(errorsmod.Wrapf(types.ErrInvalidChannelOrdering, channel.Ordering.String()))
	}

	if err != nil {
		return err
	}

	// NOTE: the remaining code is located in the TimeoutExecuted function
	return nil
}
