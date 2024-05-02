package keeper

import (
	"fmt"
	"reflect"
	"slices"

	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/telemetry"
	sdk "github.com/cosmos/cosmos-sdk/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	connectiontypes "github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	"github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// ChanUpgradeInit is called by a module to initiate a channel upgrade handshake with
// a module on another chain.
func (k Keeper) ChanUpgradeInit(
	ctx sdk.Context,
	portID string,
	channelID string,
	upgradeFields types.UpgradeFields,
) (types.Upgrade, error) {
	channel, found := k.GetChannel(ctx, portID, channelID)
	if !found {
		return types.Upgrade{}, errorsmod.Wrapf(types.ErrChannelNotFound, "port ID (%s) channel ID (%s)", portID, channelID)
	}

	if channel.State != types.OPEN {
		return types.Upgrade{}, errorsmod.Wrapf(types.ErrInvalidChannelState, "expected %s, got %s", types.OPEN, channel.State)
	}

	if err := k.validateSelfUpgradeFields(ctx, upgradeFields, channel); err != nil {
		return types.Upgrade{}, err
	}

	// NOTE: the Upgrade returned here is intentionally not fully populated. The Timeout remains unset
	// until the counterparty calls ChanUpgradeTry.
	return types.Upgrade{Fields: upgradeFields}, nil
}

// WriteUpgradeInitChannel writes a channel which has successfully passed the UpgradeInit handshake step.
// An event is emitted for the handshake step.
func (k Keeper) WriteUpgradeInitChannel(ctx sdk.Context, portID, channelID string, upgrade types.Upgrade, upgradeVersion string) (types.Channel, types.Upgrade) {
	defer telemetry.IncrCounter(1, "ibc", "channel", "upgrade-init")

	channel, found := k.GetChannel(ctx, portID, channelID)
	if !found {
		panic(fmt.Errorf("could not find existing channel when updating channel state in successful ChanUpgradeInit step, channelID: %s, portID: %s", channelID, portID))
	}

	if k.hasUpgrade(ctx, portID, channelID) {
		// invalidating previous upgrade
		k.deleteUpgradeInfo(ctx, portID, channelID)
		k.WriteErrorReceipt(ctx, portID, channelID, types.NewUpgradeError(channel.UpgradeSequence, types.ErrInvalidUpgrade))
	}

	channel.UpgradeSequence++

	upgrade.Fields.Version = upgradeVersion

	k.SetChannel(ctx, portID, channelID, channel)
	k.SetUpgrade(ctx, portID, channelID, upgrade)

	k.Logger(ctx).Info("channel state updated", "port-id", portID, "channel-id", channelID, "state", channel.State, "upgrade-sequence", fmt.Sprintf("%d", channel.UpgradeSequence))

	return channel, upgrade
}

// ChanUpgradeTry is called by a module to accept the first step of a channel upgrade handshake initiated by
// a module on another chain. If this function is successful, the proposed upgrade will be returned. If the upgrade fails, the upgrade sequence will still be incremented but an error will be returned.
func (k Keeper) ChanUpgradeTry(
	ctx sdk.Context,
	portID,
	channelID string,
	proposedConnectionHops []string,
	counterpartyUpgradeFields types.UpgradeFields,
	counterpartyUpgradeSequence uint64,
	channelProof,
	upgradeProof []byte,
	proofHeight clienttypes.Height,
) (types.Channel, types.Upgrade, error) {
	channel, found := k.GetChannel(ctx, portID, channelID)
	if !found {
		return types.Channel{}, types.Upgrade{}, errorsmod.Wrapf(types.ErrChannelNotFound, "port ID (%s) channel ID (%s)", portID, channelID)
	}

	if channel.State != types.OPEN {
		return types.Channel{}, types.Upgrade{}, errorsmod.Wrapf(types.ErrInvalidChannelState, "expected %s, got %s", types.OPEN, channel.State)
	}

	connection, found := k.connectionKeeper.GetConnection(ctx, channel.ConnectionHops[0])
	if !found {
		return types.Channel{}, types.Upgrade{}, errorsmod.Wrap(connectiontypes.ErrConnectionNotFound, channel.ConnectionHops[0])
	}

	if connection.GetState() != int32(connectiontypes.OPEN) {
		return types.Channel{}, types.Upgrade{}, errorsmod.Wrapf(
			connectiontypes.ErrInvalidConnectionState, "connection state is not OPEN (got %s)", connectiontypes.State(connection.GetState()).String(),
		)
	}

	// construct expected counterparty channel from information in state
	// only the counterpartyUpgradeSequence is provided by the relayer
	counterpartyConnectionHops := []string{connection.GetCounterparty().GetConnectionID()}
	counterpartyChannel := types.Channel{
		State:           types.OPEN,
		Ordering:        channel.Ordering,
		Counterparty:    types.NewCounterparty(portID, channelID),
		ConnectionHops:  counterpartyConnectionHops,
		Version:         channel.Version,
		UpgradeSequence: counterpartyUpgradeSequence, // provided by the relayer
	}

	// verify the counterparty channel state containing the upgrade sequence
	if err := k.connectionKeeper.VerifyChannelState(
		ctx,
		connection,
		proofHeight, channelProof,
		channel.Counterparty.PortId,
		channel.Counterparty.ChannelId,
		counterpartyChannel,
	); err != nil {
		return types.Channel{}, types.Upgrade{}, errorsmod.Wrap(err, "failed to verify counterparty channel state")
	}

	var (
		err                     error
		upgrade                 types.Upgrade
		isCrossingHello         bool
		expectedUpgradeSequence uint64
	)

	upgrade, isCrossingHello = k.GetUpgrade(ctx, portID, channelID)
	if isCrossingHello {
		expectedUpgradeSequence = channel.UpgradeSequence
	} else {
		// at the end of the TRY step, the current upgrade sequence will be incremented in the non-crossing
		// hello case due to calling chanUpgradeInit, we should use this expected upgrade sequence for
		// sequence mismatch comparison
		expectedUpgradeSequence = channel.UpgradeSequence + 1
	}
	if counterpartyUpgradeSequence < expectedUpgradeSequence {
		// In this case, the counterparty upgrade is outdated. We want to force the counterparty
		// to abort their upgrade and come back to sync with our own upgrade sequence.

		// In the crossing hello case, we already have an upgrade but it is at a higher sequence than the counterparty.
		// Thus, our upgrade should take priority. We force the counterparty to abort their upgrade by invalidating all counterparty
		// upgrade attempts below our own sequence by setting errorReceipt to upgradeSequence - 1.
		// Since our channel upgrade sequence was incremented in the previous step, the counterparty will be forced to abort their upgrade
		// and we will be able to proceed with our own upgrade.
		// The upgrade handshake may then proceed on the counterparty with our sequence
		// In the non-crossing hello (i.e. upgrade does not exist on our side),
		// the sequence on both sides should move to a fresh sequence on the next upgrade attempt.
		// Thus, we write an error receipt with our own upgrade sequence which will cause the counterparty
		// to cancel their upgrade and move to the same sequence. When a new upgrade attempt is started from either
		// side, it will be a fresh sequence for both sides (i.e. channel.upgradeSequence + 1).
		// Note that expectedUpgradeSequence - 1 == channel.UpgradeSequence in the non-crossing hello case.

		// NOTE: Two possible outcomes may occur in this scenario.
		// The ChanUpgradeCancel datagram may reach the counterparty first, which will cause the counterparty to cancel. The counterparty
		// may then receive a TRY with our channel upgrade sequence and correctly increment their sequence to become synced with our upgrade attempt.
		// The ChanUpgradeTry message may arrive first, in this case, **IF** the upgrade fields are mutually compatible; the counterparty will simply
		// fast forward their sequence to our own and continue the upgrade. The following ChanUpgradeCancel message will be rejected as it is below the current sequence.

		return channel, upgrade, types.NewUpgradeError(expectedUpgradeSequence-1, errorsmod.Wrapf(
			types.ErrInvalidUpgradeSequence, "counterparty upgrade sequence < current upgrade sequence (%d < %d)", counterpartyUpgradeSequence, channel.UpgradeSequence,
		))
	}

	// verifies the proof that a particular proposed upgrade has been stored in the upgrade path of the counterparty
	if err := k.connectionKeeper.VerifyChannelUpgrade(
		ctx,
		connection,
		proofHeight, upgradeProof,
		channel.Counterparty.PortId,
		channel.Counterparty.ChannelId,
		types.NewUpgrade(counterpartyUpgradeFields, types.Timeout{}, 0),
	); err != nil {
		return types.Channel{}, types.Upgrade{}, errorsmod.Wrap(err, "failed to verify counterparty upgrade")
	}

	// construct counterpartyChannel from existing information and provided counterpartyUpgradeSequence
	// create upgrade fields from counterparty proposed upgrade and own verified connection hops
	proposedUpgradeFields := types.UpgradeFields{
		Ordering:       counterpartyUpgradeFields.Ordering,
		ConnectionHops: proposedConnectionHops,
		Version:        counterpartyUpgradeFields.Version,
	}

	// NOTE: if an upgrade exists (crossing hellos) then use existing upgrade fields
	// otherwise, run the upgrade init sub-protocol
	if isCrossingHello {
		proposedUpgradeFields = upgrade.Fields
	} else {
		// NOTE: OnChanUpgradeInit will not be executed by the application
		upgrade, err = k.ChanUpgradeInit(ctx, portID, channelID, proposedUpgradeFields)
		if err != nil {
			return types.Channel{}, types.Upgrade{}, errorsmod.Wrap(err, "failed to initialize upgrade")
		}

		channel, upgrade = k.WriteUpgradeInitChannel(ctx, portID, channelID, upgrade, upgrade.Fields.Version)
	}

	if err := k.checkForUpgradeCompatibility(ctx, proposedUpgradeFields, counterpartyUpgradeFields); err != nil {
		return types.Channel{}, types.Upgrade{}, errorsmod.Wrap(err, "failed upgrade compatibility check")
	}

	// if the counterparty sequence is greater than the current sequence, we fast-forward to the counterparty sequence.
	if counterpartyUpgradeSequence > channel.UpgradeSequence {
		channel.UpgradeSequence = counterpartyUpgradeSequence
		k.SetChannel(ctx, portID, channelID, channel)
	}

	if err := k.startFlushing(ctx, portID, channelID, &upgrade); err != nil {
		return types.Channel{}, types.Upgrade{}, err
	}

	return channel, upgrade, nil
}

// WriteUpgradeTryChannel writes the channel end and upgrade to state after successfully passing the UpgradeTry handshake step.
// An event is emitted for the handshake step.
func (k Keeper) WriteUpgradeTryChannel(ctx sdk.Context, portID, channelID string, upgrade types.Upgrade, upgradeVersion string) (types.Channel, types.Upgrade) {
	defer telemetry.IncrCounter(1, "ibc", "channel", "upgrade-try")

	channel, found := k.GetChannel(ctx, portID, channelID)
	if !found {
		panic(fmt.Errorf("could not find existing channel when updating channel state in successful ChanUpgradeTry step, channelID: %s, portID: %s", channelID, portID))
	}

	upgrade.Fields.Version = upgradeVersion
	k.SetUpgrade(ctx, portID, channelID, upgrade)

	k.Logger(ctx).Info("channel state updated", "port-id", portID, "channel-id", channelID, "previous-state", types.OPEN, "new-state", channel.State)

	return channel, upgrade
}

// ChanUpgradeAck is called by a module to accept the ACKUPGRADE handshake step of the channel upgrade protocol.
// This method should only be called by the IBC core msg server.
// This method will verify that the counterparty has called the ChanUpgradeTry handler.
// and that its own upgrade is compatible with the selected counterparty version.
// NOTE: the channel may be in either the OPEN or FLUSHING state.
// The channel may be in OPEN if we are in the happy path.
//
//	A -> Init (OPEN), B -> Try (FLUSHING), A -> Ack (begins in OPEN)
//
// The channel may be in FLUSHING if we are in a crossing hellos situation.
//
//	A -> Init (OPEN), B -> Init (OPEN) -> A -> Try (FLUSHING), B -> Try (FLUSHING), A -> Ack (begins in FLUSHING)
func (k Keeper) ChanUpgradeAck(
	ctx sdk.Context,
	portID,
	channelID string,
	counterpartyUpgrade types.Upgrade,
	channelProof,
	upgradeProof []byte,
	proofHeight clienttypes.Height,
) error {
	channel, found := k.GetChannel(ctx, portID, channelID)
	if !found {
		return errorsmod.Wrapf(types.ErrChannelNotFound, "port ID (%s) channel ID (%s)", portID, channelID)
	}

	if !slices.Contains([]types.State{types.OPEN, types.FLUSHING}, channel.State) {
		return errorsmod.Wrapf(types.ErrInvalidChannelState, "expected one of [%s, %s], got %s", types.OPEN, types.FLUSHING, channel.State)
	}

	connection, found := k.connectionKeeper.GetConnection(ctx, channel.ConnectionHops[0])
	if !found {
		return errorsmod.Wrap(connectiontypes.ErrConnectionNotFound, channel.ConnectionHops[0])
	}

	if connection.GetState() != int32(connectiontypes.OPEN) {
		return errorsmod.Wrapf(connectiontypes.ErrInvalidConnectionState, "connection state is not OPEN (got %s)", connectiontypes.State(connection.GetState()).String())
	}

	counterpartyHops := []string{connection.GetCounterparty().GetConnectionID()}
	counterpartyChannel := types.Channel{
		State:           types.FLUSHING,
		Ordering:        channel.Ordering,
		ConnectionHops:  counterpartyHops,
		Counterparty:    types.NewCounterparty(portID, channelID),
		Version:         channel.Version,
		UpgradeSequence: channel.UpgradeSequence,
	}

	// verify the counterparty channel state containing the upgrade sequence
	if err := k.connectionKeeper.VerifyChannelState(
		ctx,
		connection,
		proofHeight, channelProof,
		channel.Counterparty.PortId,
		channel.Counterparty.ChannelId,
		counterpartyChannel,
	); err != nil {
		return errorsmod.Wrap(err, "failed to verify counterparty channel state")
	}

	// verifies the proof that a particular proposed upgrade has been stored in the upgrade path of the counterparty
	if err := k.connectionKeeper.VerifyChannelUpgrade(
		ctx,
		connection,
		proofHeight, upgradeProof,
		channel.Counterparty.PortId,
		channel.Counterparty.ChannelId,
		counterpartyUpgrade,
	); err != nil {
		return errorsmod.Wrap(err, "failed to verify counterparty upgrade")
	}

	// if we have cancelled our upgrade after performing UpgradeInit
	// or UpgradeTry, the lack of a stored upgrade will prevent us from
	// continuing the upgrade handshake
	upgrade, found := k.GetUpgrade(ctx, portID, channelID)
	if !found {
		return errorsmod.Wrapf(types.ErrUpgradeNotFound, "failed to retrieve channel upgrade: port ID (%s) channel ID (%s)", portID, channelID)
	}

	// optimistically accept version that TRY chain proposes and pass this to callback for confirmation
	// in the crossing hello case, we do not modify version that our TRY call returned and instead enforce
	// that both TRY calls returned the same version. It is possible that this will fail in the OnChanUpgradeAck
	// callback if the version is invalid.
	if channel.State == types.OPEN {
		upgrade.Fields.Version = counterpartyUpgrade.Fields.Version
	}

	// if upgrades are not compatible by ACK step, then we restore the channel
	if err := k.checkForUpgradeCompatibility(ctx, upgrade.Fields, counterpartyUpgrade.Fields); err != nil {
		return types.NewUpgradeError(channel.UpgradeSequence, err)
	}

	if channel.State == types.OPEN {
		if err := k.startFlushing(ctx, portID, channelID, &upgrade); err != nil {
			return err
		}
	}

	timeout := counterpartyUpgrade.Timeout
	selfHeight, selfTimestamp := clienttypes.GetSelfHeight(ctx), uint64(ctx.BlockTime().UnixNano())

	if timeout.Elapsed(selfHeight, selfTimestamp) {
		return types.NewUpgradeError(channel.UpgradeSequence, errorsmod.Wrap(timeout.ErrTimeoutElapsed(selfHeight, selfTimestamp), "counterparty upgrade timeout elapsed"))
	}

	return nil
}

// WriteUpgradeAckChannel writes a channel which has successfully passed the UpgradeAck handshake step as well as
// setting the upgrade for that channel.
// An event is emitted for the handshake step.
func (k Keeper) WriteUpgradeAckChannel(ctx sdk.Context, portID, channelID string, counterpartyUpgrade types.Upgrade) (types.Channel, types.Upgrade) {
	defer telemetry.IncrCounter(1, "ibc", "channel", "upgrade-ack")

	channel, found := k.GetChannel(ctx, portID, channelID)
	if !found {
		panic(fmt.Errorf("could not find existing channel when updating channel state in successful ChanUpgradeAck step, channelID: %s, portID: %s", channelID, portID))
	}

	if !k.HasInflightPackets(ctx, portID, channelID) {
		previousState := channel.State
		channel.State = types.FLUSHCOMPLETE
		k.SetChannel(ctx, portID, channelID, channel)
		k.Logger(ctx).Info("channel state updated", "port-id", portID, "channel-id", channelID, "previous-state", previousState, "new-state", channel.State)
	}

	upgrade, found := k.GetUpgrade(ctx, portID, channelID)
	if !found {
		panic(fmt.Errorf("could not find existing upgrade when updating channel state in successful ChanUpgradeAck step, channelID: %s, portID: %s", channelID, portID))
	}

	upgrade.Fields.Version = counterpartyUpgrade.Fields.Version

	k.SetUpgrade(ctx, portID, channelID, upgrade)
	k.SetCounterpartyUpgrade(ctx, portID, channelID, counterpartyUpgrade)

	return channel, upgrade
}

// ChanUpgradeConfirm is called on the chain which is on FLUSHING after chanUpgradeAck is called on the counterparty.
// This will inform the TRY chain of the timeout set on ACK by the counterparty. If the timeout has already exceeded, we will write an error receipt and restore.
func (k Keeper) ChanUpgradeConfirm(
	ctx sdk.Context,
	portID,
	channelID string,
	counterpartyChannelState types.State,
	counterpartyUpgrade types.Upgrade,
	channelProof,
	upgradeProof []byte,
	proofHeight clienttypes.Height,
) error {
	channel, found := k.GetChannel(ctx, portID, channelID)
	if !found {
		return errorsmod.Wrapf(types.ErrChannelNotFound, "port ID (%s) channel ID (%s)", portID, channelID)
	}

	if channel.State != types.FLUSHING {
		return errorsmod.Wrapf(types.ErrInvalidChannelState, "expected %s, got %s", types.FLUSHING, channel.State)
	}

	if !slices.Contains([]types.State{types.FLUSHING, types.FLUSHCOMPLETE}, counterpartyChannelState) {
		return errorsmod.Wrapf(types.ErrInvalidCounterparty, "expected one of [%s, %s], got %s", types.FLUSHING, types.FLUSHCOMPLETE, counterpartyChannelState)
	}

	connection, found := k.connectionKeeper.GetConnection(ctx, channel.ConnectionHops[0])
	if !found {
		return errorsmod.Wrap(connectiontypes.ErrConnectionNotFound, channel.ConnectionHops[0])
	}

	if connection.GetState() != int32(connectiontypes.OPEN) {
		return errorsmod.Wrapf(connectiontypes.ErrInvalidConnectionState, "connection state is not OPEN (got %s)", connectiontypes.State(connection.GetState()).String())
	}

	counterpartyHops := []string{connection.GetCounterparty().GetConnectionID()}
	counterpartyChannel := types.Channel{
		State:           counterpartyChannelState,
		Ordering:        channel.Ordering,
		ConnectionHops:  counterpartyHops,
		Counterparty:    types.NewCounterparty(portID, channelID),
		Version:         channel.Version,
		UpgradeSequence: channel.UpgradeSequence,
	}

	if err := k.connectionKeeper.VerifyChannelState(
		ctx,
		connection,
		proofHeight, channelProof,
		channel.Counterparty.PortId,
		channel.Counterparty.ChannelId,
		counterpartyChannel,
	); err != nil {
		return errorsmod.Wrap(err, "failed to verify counterparty channel state")
	}

	if err := k.connectionKeeper.VerifyChannelUpgrade(
		ctx,
		connection,
		proofHeight, upgradeProof,
		channel.Counterparty.PortId,
		channel.Counterparty.ChannelId,
		counterpartyUpgrade,
	); err != nil {
		return errorsmod.Wrap(err, "failed to verify counterparty upgrade")
	}

	timeout := counterpartyUpgrade.Timeout
	selfHeight, selfTimestamp := clienttypes.GetSelfHeight(ctx), uint64(ctx.BlockTime().UnixNano())

	if timeout.Elapsed(selfHeight, selfTimestamp) {
		return types.NewUpgradeError(channel.UpgradeSequence, errorsmod.Wrap(timeout.ErrTimeoutElapsed(selfHeight, selfTimestamp), "counterparty upgrade timeout elapsed"))
	}

	return nil
}

// WriteUpgradeConfirmChannel writes a channel which has successfully passed the ChanUpgradeConfirm handshake step.
// If the channel has no in-flight packets, its state is updated to indicate that flushing has completed. Otherwise, the counterparty upgrade is set
// and the channel state is left unchanged.
// An event is emitted for the handshake step.
func (k Keeper) WriteUpgradeConfirmChannel(ctx sdk.Context, portID, channelID string, counterpartyUpgrade types.Upgrade) types.Channel {
	defer telemetry.IncrCounter(1, "ibc", "channel", "upgrade-confirm")

	channel, found := k.GetChannel(ctx, portID, channelID)
	if !found {
		panic(fmt.Errorf("could not find existing channel when updating channel state in successful ChanUpgradeConfirm step, channelID: %s, portID: %s", channelID, portID))
	}

	if !k.HasInflightPackets(ctx, portID, channelID) {
		previousState := channel.State
		channel.State = types.FLUSHCOMPLETE
		k.SetChannel(ctx, portID, channelID, channel)
		k.Logger(ctx).Info("channel state updated", "port-id", portID, "channel-id", channelID, "previous-state", previousState, "new-state", channel.State)
	}

	k.SetCounterpartyUpgrade(ctx, portID, channelID, counterpartyUpgrade)
	return channel
}

// ChanUpgradeOpen is called by a module to complete the channel upgrade handshake and move the channel back to an OPEN state.
// This method should only be called after both channels have flushed any in-flight packets.
// This method should only be called directly by the core IBC message server.
func (k Keeper) ChanUpgradeOpen(
	ctx sdk.Context,
	portID,
	channelID string,
	counterpartyChannelState types.State,
	counterpartyUpgradeSequence uint64,
	channelProof []byte,
	proofHeight clienttypes.Height,
) error {
	channel, found := k.GetChannel(ctx, portID, channelID)
	if !found {
		return errorsmod.Wrapf(types.ErrChannelNotFound, "port ID (%s) channel ID (%s)", portID, channelID)
	}

	if channel.State != types.FLUSHCOMPLETE {
		return errorsmod.Wrapf(types.ErrInvalidChannelState, "expected %s, got %s", types.FLUSHCOMPLETE, channel.State)
	}

	connection, found := k.connectionKeeper.GetConnection(ctx, channel.ConnectionHops[0])
	if !found {
		return errorsmod.Wrap(connectiontypes.ErrConnectionNotFound, channel.ConnectionHops[0])
	}

	if connection.GetState() != int32(connectiontypes.OPEN) {
		return errorsmod.Wrapf(connectiontypes.ErrInvalidConnectionState, "connection state is not OPEN (got %s)", connectiontypes.State(connection.GetState()).String())
	}

	var counterpartyChannel types.Channel
	switch counterpartyChannelState {
	case types.OPEN:
		upgrade, found := k.GetUpgrade(ctx, portID, channelID)
		if !found {
			return errorsmod.Wrapf(types.ErrUpgradeNotFound, "failed to retrieve channel upgrade: port ID (%s) channel ID (%s)", portID, channelID)
		}
		// If counterparty has reached OPEN, we must use the upgraded connection to verify the counterparty channel
		upgradeConnection, found := k.connectionKeeper.GetConnection(ctx, upgrade.Fields.ConnectionHops[0])
		if !found {
			return errorsmod.Wrap(connectiontypes.ErrConnectionNotFound, upgrade.Fields.ConnectionHops[0])
		}

		if upgradeConnection.GetState() != int32(connectiontypes.OPEN) {
			return errorsmod.Wrapf(connectiontypes.ErrInvalidConnectionState, "connection state is not OPEN (got %s)", connectiontypes.State(upgradeConnection.GetState()).String())
		}

		// The counterparty upgrade sequence must be greater than or equal to
		// the channel upgrade sequence. It should normally be equivalent, but
		// in the unlikely case a new upgrade is initiated after it reopens,
		// then the upgrade sequence will be greater than our upgrade sequence.
		if counterpartyUpgradeSequence < channel.UpgradeSequence {
			return errorsmod.Wrapf(types.ErrInvalidUpgradeSequence, "counterparty channel upgrade sequence (%d) must be greater than or equal to current upgrade sequence (%d)", counterpartyUpgradeSequence, channel.UpgradeSequence)
		}

		counterpartyChannel = types.Channel{
			State:           types.OPEN,
			Ordering:        upgrade.Fields.Ordering,
			ConnectionHops:  []string{upgradeConnection.GetCounterparty().GetConnectionID()},
			Counterparty:    types.NewCounterparty(portID, channelID),
			Version:         upgrade.Fields.Version,
			UpgradeSequence: counterpartyUpgradeSequence,
		}

	case types.FLUSHCOMPLETE:
		counterpartyChannel = types.Channel{
			State:           types.FLUSHCOMPLETE,
			Ordering:        channel.Ordering,
			ConnectionHops:  []string{connection.GetCounterparty().GetConnectionID()},
			Counterparty:    types.NewCounterparty(portID, channelID),
			Version:         channel.Version,
			UpgradeSequence: channel.UpgradeSequence,
		}

	default:
		return errorsmod.Wrapf(types.ErrInvalidCounterparty, "counterparty channel state must be one of [%s, %s], got %s", types.OPEN, types.FLUSHCOMPLETE, counterpartyChannelState)
	}

	if err := k.connectionKeeper.VerifyChannelState(
		ctx,
		connection,
		proofHeight, channelProof,
		channel.Counterparty.PortId,
		channel.Counterparty.ChannelId,
		counterpartyChannel,
	); err != nil {
		return errorsmod.Wrap(err, "failed to verify counterparty channel")
	}

	return nil
}

// WriteUpgradeOpenChannel writes the agreed upon upgrade fields to the channel, and sets the channel state back to OPEN. This can be called in one of two cases:
// - In the UpgradeConfirm step of the handshake if both sides have already flushed all in-flight packets.
// - In the UpgradeOpen step of the handshake.
func (k Keeper) WriteUpgradeOpenChannel(ctx sdk.Context, portID, channelID string) types.Channel {
	channel, found := k.GetChannel(ctx, portID, channelID)
	if !found {
		panic(fmt.Errorf("could not find existing channel when updating channel state, channelID: %s, portID: %s", channelID, portID))
	}

	upgrade, found := k.GetUpgrade(ctx, portID, channelID)
	if !found {
		panic(fmt.Errorf("could not find upgrade when updating channel state, channelID: %s, portID: %s", channelID, portID))
	}

	counterpartyUpgrade, found := k.GetCounterpartyUpgrade(ctx, portID, channelID)
	if !found {
		panic(fmt.Errorf("could not find counterparty upgrade when updating channel state, channelID: %s, portID: %s", channelID, portID))
	}

	// next seq recv and ack is used for ordered channels to verify the packet has been received/acked in the correct order
	// this is no longer necessary if the channel is UNORDERED and should be reset to 1
	if channel.Ordering == types.ORDERED && upgrade.Fields.Ordering == types.UNORDERED {
		k.SetNextSequenceRecv(ctx, portID, channelID, 1)
		k.SetNextSequenceAck(ctx, portID, channelID, 1)
	}

	// next seq recv and ack should updated when moving from UNORDERED to ORDERED using the counterparty NextSequenceSend as set just after blocking new packet sends.
	// we can be sure that the next packet we are set to receive will be the first packet the counterparty sends after reopening.
	// we can be sure that our next acknowledgement will be our first packet sent after upgrade, as the counterparty processed all sent packets after flushing completes.
	if channel.Ordering == types.UNORDERED && upgrade.Fields.Ordering == types.ORDERED {
		k.SetNextSequenceRecv(ctx, portID, channelID, counterpartyUpgrade.NextSequenceSend)
		k.SetNextSequenceAck(ctx, portID, channelID, upgrade.NextSequenceSend)
	}

	// Set the counterparty next sequence send as the recv start sequence.
	// This will be the upper bound for pruning and it will allow for replay
	// protection of historical packets.
	k.setRecvStartSequence(ctx, portID, channelID, counterpartyUpgrade.NextSequenceSend)

	// First upgrade for this channel will set the pruning sequence to 1, the starting sequence for pruning.
	// Subsequent upgrades will not modify the pruning sequence thereby allowing pruning to continue from the last
	// pruned sequence.
	if !k.HasPruningSequenceStart(ctx, portID, channelID) {
		k.SetPruningSequenceStart(ctx, portID, channelID, 1)
	}

	// Switch channel fields to upgrade fields and set channel state to OPEN
	previousState := channel.State
	channel.Ordering = upgrade.Fields.Ordering
	channel.Version = upgrade.Fields.Version
	channel.ConnectionHops = upgrade.Fields.ConnectionHops
	channel.State = types.OPEN

	k.SetChannel(ctx, portID, channelID, channel)

	// delete state associated with upgrade which is no longer required.
	k.deleteUpgradeInfo(ctx, portID, channelID)

	k.Logger(ctx).Info("channel state updated", "port-id", portID, "channel-id", channelID, "previous-state", previousState.String(), "new-state", types.OPEN.String())
	return channel
}

// ChanUpgradeCancel is called by the msg server to prove that an error receipt was written on the counterparty
// which constitutes a valid situation where the upgrade should be cancelled. An error is returned if sufficient evidence
// for cancelling the upgrade has not been provided.
func (k Keeper) ChanUpgradeCancel(ctx sdk.Context, portID, channelID string, errorReceipt types.ErrorReceipt, errorReceiptProof []byte, proofHeight clienttypes.Height) error {
	channel, found := k.GetChannel(ctx, portID, channelID)
	if !found {
		return errorsmod.Wrapf(types.ErrChannelNotFound, "port ID (%s) channel ID (%s)", portID, channelID)
	}

	_, found = k.GetUpgrade(ctx, portID, channelID)
	if !found {
		return errorsmod.Wrapf(types.ErrUpgradeNotFound, "port ID (%s) channel ID (%s)", portID, channelID)
	}

	// an error receipt proof must be provided.
	if len(errorReceiptProof) == 0 {
		return errorsmod.Wrap(commitmenttypes.ErrInvalidProof, "cannot submit an empty error receipt proof unless the sender is authorized to cancel upgrades AND channel is not in FLUSHCOMPLETE")
	}

	// REPLAY PROTECTION: The error receipt MUST have a sequence greater than or equal to the current upgrade sequence,
	// except when the channel state is FLUSHCOMPLETE, in which case the sequences MUST match. This is required
	// to guarantee that when a counterparty successfully completes an upgrade and moves to OPEN, this channel
	// cannot cancel its upgrade.  Without this strict sequence check, it would be possible for the counterparty
	// to complete its upgrade, move to OPEN, initiate a new upgrade (and thus increment the upgrade sequence) and
	// then cancel the new upgrade, all in the same block. This results in a valid error receipt being written at channel.UpgradeSequence + 1.
	// The desired behaviour in this circumstance is for this channel to complete its current upgrade despite proof
	// of an error receipt at a greater upgrade sequence
	if channel.State == types.FLUSHCOMPLETE && errorReceipt.Sequence != channel.UpgradeSequence {
		return errorsmod.Wrapf(types.ErrInvalidUpgradeSequence, "error receipt sequence (%d) must be equal to current upgrade sequence (%d) when the channel is in FLUSHCOMPLETE", errorReceipt.Sequence, channel.UpgradeSequence)
	}
	if errorReceipt.Sequence < channel.UpgradeSequence {
		return errorsmod.Wrapf(types.ErrInvalidUpgradeSequence, "error receipt sequence (%d) must be greater than or equal to current upgrade sequence (%d)", errorReceipt.Sequence, channel.UpgradeSequence)
	}

	// get underlying connection for proof verification
	connection, found := k.connectionKeeper.GetConnection(ctx, channel.ConnectionHops[0])
	if !found {
		return errorsmod.Wrap(connectiontypes.ErrConnectionNotFound, channel.ConnectionHops[0])
	}

	if connection.GetState() != int32(connectiontypes.OPEN) {
		return errorsmod.Wrapf(
			connectiontypes.ErrInvalidConnectionState,
			"connection state is not OPEN (got %s)", connectiontypes.State(connection.GetState()).String(),
		)
	}

	if err := k.connectionKeeper.VerifyChannelUpgradeError(
		ctx,
		connection,
		proofHeight,
		errorReceiptProof,
		channel.Counterparty.PortId,
		channel.Counterparty.ChannelId,
		errorReceipt,
	); err != nil {
		return errorsmod.Wrap(err, "failed to verify counterparty error receipt")
	}

	return nil
}

// WriteUpgradeCancelChannel writes a channel which has canceled the upgrade process.Auxiliary upgrade state is
// also deleted.
func (k Keeper) WriteUpgradeCancelChannel(ctx sdk.Context, portID, channelID string, sequence uint64) {
	defer telemetry.IncrCounter(1, "ibc", "channel", "upgrade-cancel")

	channel, found := k.GetChannel(ctx, portID, channelID)
	if !found {
		panic(fmt.Errorf("could not find existing channel when updating channel state, channelID: %s, portID: %s", channelID, portID))
	}

	previousState := channel.State

	channel = k.restoreChannel(ctx, portID, channelID, sequence, channel)
	k.WriteErrorReceipt(ctx, portID, channelID, types.NewUpgradeError(sequence, types.ErrInvalidUpgrade))

	k.Logger(ctx).Info("channel state updated", "port-id", portID, "channel-id", channelID, "previous-state", previousState, "new-state", types.OPEN.String())
}

// ChanUpgradeTimeout times out an outstanding upgrade.
// This should be used by the initialising chain when the counterparty chain has not responded to an upgrade proposal within the specified timeout period.
func (k Keeper) ChanUpgradeTimeout(
	ctx sdk.Context,
	portID, channelID string,
	counterpartyChannel types.Channel,
	counterpartyChannelProof []byte,
	proofHeight exported.Height,
) error {
	channel, found := k.GetChannel(ctx, portID, channelID)
	if !found {
		return errorsmod.Wrapf(types.ErrChannelNotFound, "port ID (%s) channel ID (%s)", portID, channelID)
	}

	if !slices.Contains([]types.State{types.FLUSHING, types.FLUSHCOMPLETE}, channel.State) {
		return errorsmod.Wrapf(types.ErrInvalidChannelState, "expected one of [%s, %s], got %s", types.FLUSHING, types.FLUSHCOMPLETE, channel.State)
	}

	upgrade, found := k.GetUpgrade(ctx, portID, channelID)
	if !found {
		return errorsmod.Wrapf(types.ErrUpgradeNotFound, "port ID (%s) channel ID (%s)", portID, channelID)
	}

	connection, found := k.connectionKeeper.GetConnection(ctx, channel.ConnectionHops[0])
	if !found {
		return errorsmod.Wrap(
			connectiontypes.ErrConnectionNotFound,
			channel.ConnectionHops[0],
		)
	}

	if connection.GetState() != int32(connectiontypes.OPEN) {
		return errorsmod.Wrapf(
			connectiontypes.ErrInvalidConnectionState,
			"connection state is not OPEN (got %s)", connectiontypes.State(connection.GetState()).String(),
		)
	}

	proofTimestamp, err := k.connectionKeeper.GetTimestampAtHeight(ctx, connection, proofHeight)
	if err != nil {
		return err
	}

	// proof must be from a height after timeout has elapsed. Either timeoutHeight or timeoutTimestamp must be defined.
	// if timeoutHeight is defined and proof is from before timeout height, abort transaction
	if !upgrade.Timeout.Elapsed(proofHeight.(clienttypes.Height), proofTimestamp) {
		return errorsmod.Wrap(upgrade.Timeout.ErrTimeoutNotReached(proofHeight.(clienttypes.Height), proofTimestamp), "upgrade timeout not reached")
	}

	// counterparty channel must be proved to still be in OPEN state or FLUSHING state.
	if !slices.Contains([]types.State{types.OPEN, types.FLUSHING}, counterpartyChannel.State) {
		return errorsmod.Wrapf(types.ErrInvalidCounterparty, "expected one of [%s, %s], got %s", types.OPEN, types.FLUSHING, counterpartyChannel.State)
	}

	if counterpartyChannel.State == types.OPEN {
		upgradeConnection, found := k.connectionKeeper.GetConnection(ctx, upgrade.Fields.ConnectionHops[0])
		if !found {
			return errorsmod.Wrap(
				connectiontypes.ErrConnectionNotFound,
				upgrade.Fields.ConnectionHops[0],
			)
		}
		counterpartyHops := []string{upgradeConnection.GetCounterparty().GetConnectionID()}

		upgradeAlreadyComplete := upgrade.Fields.Version == counterpartyChannel.Version && upgrade.Fields.Ordering == counterpartyChannel.Ordering && upgrade.Fields.ConnectionHops[0] == counterpartyHops[0]
		if upgradeAlreadyComplete {
			// counterparty has already successfully upgraded so we cannot timeout
			return errorsmod.Wrap(types.ErrUpgradeTimeoutFailed, "counterparty channel is already upgraded")
		}
	}

	if counterpartyChannel.UpgradeSequence < channel.UpgradeSequence {
		return errorsmod.Wrapf(types.ErrInvalidUpgradeSequence, "counterparty channel upgrade sequence (%d) must be greater than or equal to current upgrade sequence (%d)", counterpartyChannel.UpgradeSequence, channel.UpgradeSequence)
	}

	// NOTE: The counterpartyChannel upgrade fields are not checked in the case
	// the counterpartyChannel is in FLUSHING. This is not required because
	// we prove that the upgrade timeout has elapsed on the counterparty,
	// thus no historical proofs can be submitted. It is not possible for the
	// counterparty to have upgraded if they were in FLUSHING and the upgrade
	// timeout elapsed. Do not make use of the relayer provided fields without
	// verifying them.

	// verify the counterparty channel state
	if err := k.connectionKeeper.VerifyChannelState(
		ctx,
		connection,
		proofHeight, counterpartyChannelProof,
		channel.Counterparty.PortId,
		channel.Counterparty.ChannelId,
		counterpartyChannel,
	); err != nil {
		return errorsmod.Wrap(err, "failed to verify counterparty channel state")
	}

	return nil
}

// WriteUpgradeTimeoutChannel restores the channel state of an initialising chain in the event that the counterparty chain has passed the timeout set in ChanUpgradeInit to the state before the upgrade was proposed.
// Auxiliary upgrade state is also deleted.
// An event is emitted for the handshake step.
func (k Keeper) WriteUpgradeTimeoutChannel(
	ctx sdk.Context,
	portID, channelID string,
) (types.Channel, types.Upgrade) {
	defer telemetry.IncrCounter(1, "ibc", "channel", "upgrade-timeout")

	channel, found := k.GetChannel(ctx, portID, channelID)
	if !found {
		panic(fmt.Errorf("could not find existing channel when updating channel state in successful ChanUpgradeTimeout step, channelID: %s, portID: %s", channelID, portID))
	}

	upgrade, found := k.GetUpgrade(ctx, portID, channelID)
	if !found {
		panic(fmt.Errorf("could not find existing upgrade when cancelling channel upgrade, channelID: %s, portID: %s", channelID, portID))
	}

	channel = k.restoreChannel(ctx, portID, channelID, channel.UpgradeSequence, channel)
	k.WriteErrorReceipt(ctx, portID, channelID, types.NewUpgradeError(channel.UpgradeSequence, types.ErrUpgradeTimeout))

	k.Logger(ctx).Info("channel state restored", "port-id", portID, "channel-id", channelID)

	return channel, upgrade
}

// startFlushing will set the upgrade last packet send and continue blocking the upgrade from continuing until all
// in-flight packets have been flushed.
func (k Keeper) startFlushing(ctx sdk.Context, portID, channelID string, upgrade *types.Upgrade) error {
	channel, found := k.GetChannel(ctx, portID, channelID)
	if !found {
		return errorsmod.Wrapf(types.ErrChannelNotFound, "port ID (%s) channel ID (%s)", portID, channelID)
	}

	connection, found := k.connectionKeeper.GetConnection(ctx, channel.ConnectionHops[0])
	if !found {
		return errorsmod.Wrap(connectiontypes.ErrConnectionNotFound, channel.ConnectionHops[0])
	}

	if connection.GetState() != int32(connectiontypes.OPEN) {
		return errorsmod.Wrapf(connectiontypes.ErrInvalidConnectionState, "connection state is not OPEN (got %s)", connectiontypes.State(connection.GetState()).String())
	}

	channel.State = types.FLUSHING
	k.SetChannel(ctx, portID, channelID, channel)

	nextSequenceSend, found := k.GetNextSequenceSend(ctx, portID, channelID)
	if !found {
		return errorsmod.Wrapf(types.ErrSequenceSendNotFound, "port ID (%s) channel ID (%s)", portID, channelID)
	}

	upgrade.NextSequenceSend = nextSequenceSend
	upgrade.Timeout = k.getAbsoluteUpgradeTimeout(ctx)
	k.SetUpgrade(ctx, portID, channelID, *upgrade)

	return nil
}

// getAbsoluteUpgradeTimeout returns the absolute timeout for the given upgrade.
func (k Keeper) getAbsoluteUpgradeTimeout(ctx sdk.Context) types.Timeout {
	upgradeTimeout := k.GetParams(ctx).UpgradeTimeout
	return types.NewTimeout(clienttypes.ZeroHeight(), uint64(ctx.BlockTime().UnixNano())+upgradeTimeout.Timestamp)
}

// checkForUpgradeCompatibility checks performs stateful validation of self upgrade fields relative to counterparty upgrade.
func (k Keeper) checkForUpgradeCompatibility(ctx sdk.Context, upgradeFields, counterpartyUpgradeFields types.UpgradeFields) error {
	// assert that both sides propose the same channel ordering
	if upgradeFields.Ordering != counterpartyUpgradeFields.Ordering {
		return errorsmod.Wrapf(types.ErrIncompatibleCounterpartyUpgrade, "expected upgrade ordering (%s) to match counterparty upgrade ordering (%s)", upgradeFields.Ordering, counterpartyUpgradeFields.Ordering)
	}

	if upgradeFields.Version != counterpartyUpgradeFields.Version {
		return errorsmod.Wrapf(types.ErrIncompatibleCounterpartyUpgrade, "expected upgrade version (%s) to match counterparty upgrade version (%s)", upgradeFields.Version, counterpartyUpgradeFields.Version)
	}

	connection, found := k.connectionKeeper.GetConnection(ctx, upgradeFields.ConnectionHops[0])
	if !found {
		// NOTE: this error is expected to be unreachable as the proposed upgrade connectionID should have been
		// validated in the upgrade INIT and TRY handlers
		return errorsmod.Wrap(connectiontypes.ErrConnectionNotFound, upgradeFields.ConnectionHops[0])
	}

	if connection.GetState() != int32(connectiontypes.OPEN) {
		// NOTE: this error is expected to be unreachable as the proposed upgrade connectionID should have been
		// validated in the upgrade INIT and TRY handlers
		return errorsmod.Wrapf(connectiontypes.ErrInvalidConnectionState, "expected proposed connection to be OPEN (got %s)", connectiontypes.State(connection.GetState()).String())
	}

	// connectionHops can change in a channelUpgrade, however both sides must still be each other's counterparty.
	if counterpartyUpgradeFields.ConnectionHops[0] != connection.GetCounterparty().GetConnectionID() {
		return errorsmod.Wrapf(
			types.ErrIncompatibleCounterpartyUpgrade, "counterparty upgrade connection end is not a counterparty of self proposed connection end (%s != %s)", counterpartyUpgradeFields.ConnectionHops[0], connection.GetCounterparty().GetConnectionID())
	}

	return nil
}

// validateSelfUpgradeFields validates the proposed upgrade fields against the existing channel.
// It returns an error if the following constraints are not met:
// - there exists at least one valid proposed change to the existing channel fields
// - the proposed connection hops do not exist
// - the proposed version is non-empty (checked in UpgradeFields.ValidateBasic())
// - the proposed connection hops are not open
func (k Keeper) validateSelfUpgradeFields(ctx sdk.Context, proposedUpgrade types.UpgradeFields, currentChannel types.Channel) error {
	currentFields := extractUpgradeFields(currentChannel)

	if reflect.DeepEqual(proposedUpgrade, currentFields) {
		return errorsmod.Wrapf(types.ErrInvalidUpgrade, "existing channel end is identical to proposed upgrade channel end: got %s", proposedUpgrade)
	}

	connectionID := proposedUpgrade.ConnectionHops[0]
	connection, found := k.connectionKeeper.GetConnection(ctx, connectionID)
	if !found {
		return errorsmod.Wrapf(connectiontypes.ErrConnectionNotFound, "failed to retrieve connection: %s", connectionID)
	}

	if connection.GetState() != int32(connectiontypes.OPEN) {
		return errorsmod.Wrapf(
			connectiontypes.ErrInvalidConnectionState,
			"connection state is not OPEN (got %s)", connectiontypes.State(connection.GetState()).String(),
		)
	}

	getVersions := connection.GetVersions()
	if len(getVersions) != 1 {
		return errorsmod.Wrapf(
			connectiontypes.ErrInvalidVersion,
			"single version must be negotiated on connection before opening channel, got: %v",
			getVersions,
		)
	}

	if !connectiontypes.VerifySupportedFeature(getVersions[0], proposedUpgrade.Ordering.String()) {
		return errorsmod.Wrapf(
			connectiontypes.ErrInvalidVersion,
			"connection version %s does not support channel ordering: %s",
			getVersions[0], proposedUpgrade.Ordering.String(),
		)
	}

	return nil
}

// extractUpgradeFields returns the upgrade fields from the provided channel.
func extractUpgradeFields(channel types.Channel) types.UpgradeFields {
	return types.UpgradeFields{
		Ordering:       channel.Ordering,
		ConnectionHops: channel.ConnectionHops,
		Version:        channel.Version,
	}
}

// MustAbortUpgrade will restore the channel state to its pre-upgrade state so that upgrade is aborted.
// Any unnecessary state is deleted and an error receipt is written.
// This function is expected to always succeed, a panic will occur if an error occurs.
func (k Keeper) MustAbortUpgrade(ctx sdk.Context, portID, channelID string, err error) {
	if err := k.abortUpgrade(ctx, portID, channelID, err); err != nil {
		panic(err)
	}
}

// abortUpgrade will restore the channel state to its pre-upgrade state so that upgrade is aborted.
// All upgrade information associated with the upgrade attempt is deleted and an upgrade error
// receipt is written for that upgrade attempt. This prevents the upgrade handshake from continuing
// on our side and provides proof for the counterparty to safely abort the upgrade.
func (k Keeper) abortUpgrade(ctx sdk.Context, portID, channelID string, err error) error {
	if err == nil {
		return errorsmod.Wrap(types.ErrInvalidUpgradeError, "cannot abort upgrade handshake with nil error")
	}

	channel, found := k.GetChannel(ctx, portID, channelID)
	if !found {
		return errorsmod.Wrapf(types.ErrChannelNotFound, "port ID (%s) channel ID (%s)", portID, channelID)
	}

	// in the case of application callbacks, the error may not be an upgrade error.
	// in this case we need to construct one in order to write the error receipt.
	upgradeError, ok := err.(*types.UpgradeError)
	if !ok {
		upgradeError = types.NewUpgradeError(channel.UpgradeSequence, err)
	}

	// the channel upgrade sequence has already been updated in ChannelUpgradeTry, so we can pass
	// its updated value.
	k.restoreChannel(ctx, portID, channelID, channel.UpgradeSequence, channel)
	k.WriteErrorReceipt(ctx, portID, channelID, upgradeError)

	return nil
}

// restoreChannel will restore the channel state to its pre-upgrade state so that upgrade is aborted.
// When an upgrade attempt is aborted, the upgrade information must be deleted. This prevents us
// from continuing an upgrade handshake after we cancel an upgrade attempt.
func (k Keeper) restoreChannel(ctx sdk.Context, portID, channelID string, upgradeSequence uint64, channel types.Channel) types.Channel {
	channel.State = types.OPEN
	channel.UpgradeSequence = upgradeSequence

	k.SetChannel(ctx, portID, channelID, channel)

	// delete state associated with upgrade which is no longer required.
	k.deleteUpgradeInfo(ctx, portID, channelID)

	return channel
}

// WriteErrorReceipt will write an error receipt from the provided UpgradeError.
func (k Keeper) WriteErrorReceipt(ctx sdk.Context, portID, channelID string, upgradeError *types.UpgradeError) {
	channel, found := k.GetChannel(ctx, portID, channelID)
	if !found {
		panic(errorsmod.Wrapf(types.ErrChannelNotFound, "port ID (%s) channel ID (%s)", portID, channelID))
	}

	errorReceiptToWrite := upgradeError.GetErrorReceipt()

	existingErrorReceipt, found := k.GetUpgradeErrorReceipt(ctx, portID, channelID)
	if found && existingErrorReceipt.Sequence >= errorReceiptToWrite.Sequence {
		panic(errorsmod.Wrapf(types.ErrInvalidUpgradeSequence, "error receipt sequence (%d) must be greater than existing error receipt sequence (%d)", errorReceiptToWrite.Sequence, existingErrorReceipt.Sequence))
	}

	// Ensure that no upgrade attempt exists for the same sequence we are
	// writing an error receipt for. This could lead to divergent behaviour
	// on the counterparty.
	if channel.UpgradeSequence <= errorReceiptToWrite.Sequence {
		upgradeFound := k.hasUpgrade(ctx, portID, channelID)
		counterpartyUpgradeFound := k.hasCounterpartyUpgrade(ctx, portID, channelID)
		if upgradeFound || counterpartyUpgradeFound {
			panic(errorsmod.Wrapf(types.ErrInvalidUpgradeSequence, "attempting to write error receipt at sequence (%d) while upgrade information exists at the same sequence", errorReceiptToWrite.Sequence))
		}
	}

	k.setUpgradeErrorReceipt(ctx, portID, channelID, errorReceiptToWrite)
	EmitErrorReceiptEvent(ctx, portID, channelID, channel, upgradeError)
}
