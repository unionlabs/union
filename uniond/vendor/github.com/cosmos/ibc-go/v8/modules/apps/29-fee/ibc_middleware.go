package fee

import (
	"strings"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"

	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/keeper"
	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	porttypes "github.com/cosmos/ibc-go/v8/modules/core/05-port/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

var (
	_ porttypes.Middleware            = (*IBCMiddleware)(nil)
	_ porttypes.PacketDataUnmarshaler = (*IBCMiddleware)(nil)
	_ porttypes.UpgradableModule      = (*IBCMiddleware)(nil)
)

// IBCMiddleware implements the ICS26 callbacks for the fee middleware given the
// fee keeper and the underlying application.
type IBCMiddleware struct {
	app    porttypes.IBCModule
	keeper keeper.Keeper
}

// NewIBCMiddleware creates a new IBCMiddlware given the keeper and underlying application
func NewIBCMiddleware(app porttypes.IBCModule, k keeper.Keeper) IBCMiddleware {
	return IBCMiddleware{
		app:    app,
		keeper: k,
	}
}

// OnChanOpenInit implements the IBCMiddleware interface
func (im IBCMiddleware) OnChanOpenInit(
	ctx sdk.Context,
	order channeltypes.Order,
	connectionHops []string,
	portID string,
	channelID string,
	chanCap *capabilitytypes.Capability,
	counterparty channeltypes.Counterparty,
	version string,
) (string, error) {
	var versionMetadata types.Metadata

	if strings.TrimSpace(version) == "" {
		// default version
		versionMetadata = types.Metadata{
			FeeVersion: types.Version,
			AppVersion: "",
		}
	} else {
		metadata, err := types.MetadataFromVersion(version)
		if err != nil {
			// Since it is valid for fee version to not be specified, the above middleware version may be for a middleware
			// lower down in the stack. Thus, if it is not a fee version we pass the entire version string onto the underlying
			// application.
			return im.app.OnChanOpenInit(ctx, order, connectionHops, portID, channelID,
				chanCap, counterparty, version)
		}
		versionMetadata = metadata
	}

	if versionMetadata.FeeVersion != types.Version {
		return "", errorsmod.Wrapf(types.ErrInvalidVersion, "expected %s, got %s", types.Version, versionMetadata.FeeVersion)
	}

	appVersion, err := im.app.OnChanOpenInit(ctx, order, connectionHops, portID, channelID, chanCap, counterparty, versionMetadata.AppVersion)
	if err != nil {
		return "", err
	}

	versionMetadata.AppVersion = appVersion
	versionBytes, err := types.ModuleCdc.MarshalJSON(&versionMetadata)
	if err != nil {
		return "", err
	}

	im.keeper.SetFeeEnabled(ctx, portID, channelID)

	// call underlying app's OnChanOpenInit callback with the appVersion
	return string(versionBytes), nil
}

// OnChanOpenTry implements the IBCMiddleware interface
// If the channel is not fee enabled the underlying application version will be returned
// If the channel is fee enabled we merge the underlying application version with the ics29 version
func (im IBCMiddleware) OnChanOpenTry(
	ctx sdk.Context,
	order channeltypes.Order,
	connectionHops []string,
	portID,
	channelID string,
	chanCap *capabilitytypes.Capability,
	counterparty channeltypes.Counterparty,
	counterpartyVersion string,
) (string, error) {
	versionMetadata, err := types.MetadataFromVersion(counterpartyVersion)
	if err != nil {
		// Since it is valid for fee version to not be specified, the above middleware version may be for a middleware
		// lower down in the stack. Thus, if it is not a fee version we pass the entire version string onto the underlying
		// application.
		return im.app.OnChanOpenTry(ctx, order, connectionHops, portID, channelID, chanCap, counterparty, counterpartyVersion)
	}

	if versionMetadata.FeeVersion != types.Version {
		return "", errorsmod.Wrapf(types.ErrInvalidVersion, "expected %s, got %s", types.Version, versionMetadata.FeeVersion)
	}

	im.keeper.SetFeeEnabled(ctx, portID, channelID)

	// call underlying app's OnChanOpenTry callback with the app versions
	appVersion, err := im.app.OnChanOpenTry(ctx, order, connectionHops, portID, channelID, chanCap, counterparty, versionMetadata.AppVersion)
	if err != nil {
		return "", err
	}

	versionMetadata.AppVersion = appVersion
	versionBytes, err := types.ModuleCdc.MarshalJSON(&versionMetadata)
	if err != nil {
		return "", err
	}

	return string(versionBytes), nil
}

// OnChanOpenAck implements the IBCMiddleware interface
func (im IBCMiddleware) OnChanOpenAck(
	ctx sdk.Context,
	portID,
	channelID string,
	counterpartyChannelID string,
	counterpartyVersion string,
) error {
	// If handshake was initialized with fee enabled it must complete with fee enabled.
	// If handshake was initialized with fee disabled it must complete with fee disabled.
	if im.keeper.IsFeeEnabled(ctx, portID, channelID) {
		versionMetadata, err := types.MetadataFromVersion(counterpartyVersion)
		if err != nil {
			return errorsmod.Wrapf(err, "failed to unmarshal ICS29 counterparty version metadata: %s", counterpartyVersion)
		}

		if versionMetadata.FeeVersion != types.Version {
			return errorsmod.Wrapf(types.ErrInvalidVersion, "expected counterparty fee version: %s, got: %s", types.Version, versionMetadata.FeeVersion)
		}

		// call underlying app's OnChanOpenAck callback with the counterparty app version.
		return im.app.OnChanOpenAck(ctx, portID, channelID, counterpartyChannelID, versionMetadata.AppVersion)
	}

	// call underlying app's OnChanOpenAck callback with the counterparty app version.
	return im.app.OnChanOpenAck(ctx, portID, channelID, counterpartyChannelID, counterpartyVersion)
}

// OnChanOpenConfirm implements the IBCMiddleware interface
func (im IBCMiddleware) OnChanOpenConfirm(
	ctx sdk.Context,
	portID,
	channelID string,
) error {
	// call underlying app's OnChanOpenConfirm callback.
	return im.app.OnChanOpenConfirm(ctx, portID, channelID)
}

// OnChanCloseInit implements the IBCMiddleware interface
func (im IBCMiddleware) OnChanCloseInit(
	ctx sdk.Context,
	portID,
	channelID string,
) error {
	if err := im.app.OnChanCloseInit(ctx, portID, channelID); err != nil {
		return err
	}

	if !im.keeper.IsFeeEnabled(ctx, portID, channelID) {
		return nil
	}

	if im.keeper.IsLocked(ctx) {
		return types.ErrFeeModuleLocked
	}

	return im.keeper.RefundFeesOnChannelClosure(ctx, portID, channelID)
}

// OnChanCloseConfirm implements the IBCMiddleware interface
func (im IBCMiddleware) OnChanCloseConfirm(
	ctx sdk.Context,
	portID,
	channelID string,
) error {
	if err := im.app.OnChanCloseConfirm(ctx, portID, channelID); err != nil {
		return err
	}

	if !im.keeper.IsFeeEnabled(ctx, portID, channelID) {
		return nil
	}

	if im.keeper.IsLocked(ctx) {
		return types.ErrFeeModuleLocked
	}

	return im.keeper.RefundFeesOnChannelClosure(ctx, portID, channelID)
}

// OnRecvPacket implements the IBCMiddleware interface.
// If fees are not enabled, this callback will default to the ibc-core packet callback
func (im IBCMiddleware) OnRecvPacket(
	ctx sdk.Context,
	packet channeltypes.Packet,
	relayer sdk.AccAddress,
) exported.Acknowledgement {
	if !im.keeper.IsFeeEnabled(ctx, packet.DestinationPort, packet.DestinationChannel) {
		return im.app.OnRecvPacket(ctx, packet, relayer)
	}

	ack := im.app.OnRecvPacket(ctx, packet, relayer)

	// in case of async aknowledgement (ack == nil) store the relayer address for use later during async WriteAcknowledgement
	if ack == nil {
		im.keeper.SetRelayerAddressForAsyncAck(ctx, channeltypes.NewPacketID(packet.GetDestPort(), packet.GetDestChannel(), packet.GetSequence()), relayer.String())
		return nil
	}

	// if forwardRelayer is not found we refund recv_fee
	forwardRelayer, _ := im.keeper.GetCounterpartyPayeeAddress(ctx, relayer.String(), packet.GetDestChannel())

	return types.NewIncentivizedAcknowledgement(forwardRelayer, ack.Acknowledgement(), ack.Success())
}

// OnAcknowledgementPacket implements the IBCMiddleware interface
// If fees are not enabled, this callback will default to the ibc-core packet callback
func (im IBCMiddleware) OnAcknowledgementPacket(
	ctx sdk.Context,
	packet channeltypes.Packet,
	acknowledgement []byte,
	relayer sdk.AccAddress,
) error {
	if !im.keeper.IsFeeEnabled(ctx, packet.SourcePort, packet.SourceChannel) {
		return im.app.OnAcknowledgementPacket(ctx, packet, acknowledgement, relayer)
	}

	var ack types.IncentivizedAcknowledgement
	if err := types.ModuleCdc.UnmarshalJSON(acknowledgement, &ack); err != nil {
		return errorsmod.Wrapf(err, "cannot unmarshal ICS-29 incentivized packet acknowledgement: %v", ack)
	}

	if im.keeper.IsLocked(ctx) {
		// if the fee keeper is locked then fee logic should be skipped
		// this may occur in the presence of a severe bug which leads to invalid state
		// the fee keeper will be unlocked after manual intervention
		// the acknowledgement has been unmarshalled into an ics29 acknowledgement
		// since the counterparty is still sending incentivized acknowledgements
		// for fee enabled channels
		//
		// Please see ADR 004 for more information.
		return im.app.OnAcknowledgementPacket(ctx, packet, ack.AppAcknowledgement, relayer)
	}

	packetID := channeltypes.NewPacketID(packet.SourcePort, packet.SourceChannel, packet.Sequence)
	feesInEscrow, found := im.keeper.GetFeesInEscrow(ctx, packetID)
	if !found {
		// call underlying callback
		return im.app.OnAcknowledgementPacket(ctx, packet, ack.AppAcknowledgement, relayer)
	}

	payee, found := im.keeper.GetPayeeAddress(ctx, relayer.String(), packet.SourceChannel)
	if !found {
		payee = relayer.String()
	}

	payeeAddr, err := sdk.AccAddressFromBech32(payee)
	if err != nil {
		return errorsmod.Wrapf(err, "failed to create sdk.Address from payee: %s", payee)
	}

	im.keeper.DistributePacketFeesOnAcknowledgement(ctx, ack.ForwardRelayerAddress, payeeAddr, feesInEscrow.PacketFees, packetID)

	// call underlying callback
	return im.app.OnAcknowledgementPacket(ctx, packet, ack.AppAcknowledgement, relayer)
}

// OnTimeoutPacket implements the IBCMiddleware interface
// If fees are not enabled, this callback will default to the ibc-core packet callback
func (im IBCMiddleware) OnTimeoutPacket(
	ctx sdk.Context,
	packet channeltypes.Packet,
	relayer sdk.AccAddress,
) error {
	// if the fee keeper is locked then fee logic should be skipped
	// this may occur in the presence of a severe bug which leads to invalid state
	// the fee keeper will be unlocked after manual intervention
	//
	// Please see ADR 004 for more information.
	if !im.keeper.IsFeeEnabled(ctx, packet.SourcePort, packet.SourceChannel) || im.keeper.IsLocked(ctx) {
		return im.app.OnTimeoutPacket(ctx, packet, relayer)
	}

	packetID := channeltypes.NewPacketID(packet.SourcePort, packet.SourceChannel, packet.Sequence)
	feesInEscrow, found := im.keeper.GetFeesInEscrow(ctx, packetID)
	if !found {
		// call underlying callback
		return im.app.OnTimeoutPacket(ctx, packet, relayer)
	}

	payee, found := im.keeper.GetPayeeAddress(ctx, relayer.String(), packet.SourceChannel)
	if !found {
		payee = relayer.String()
	}

	payeeAddr, err := sdk.AccAddressFromBech32(payee)
	if err != nil {
		return errorsmod.Wrapf(err, "failed to create sdk.Address from payee: %s", payee)
	}

	im.keeper.DistributePacketFeesOnTimeout(ctx, payeeAddr, feesInEscrow.PacketFees, packetID)

	// call underlying callback
	return im.app.OnTimeoutPacket(ctx, packet, relayer)
}

// OnChanUpgradeInit implements the IBCModule interface
func (im IBCMiddleware) OnChanUpgradeInit(
	ctx sdk.Context,
	portID string,
	channelID string,
	proposedOrder channeltypes.Order,
	proposedConnectionHops []string,
	proposedVersion string,
) (string, error) {
	cbs, ok := im.app.(porttypes.UpgradableModule)
	if !ok {
		return "", errorsmod.Wrap(porttypes.ErrInvalidRoute, "upgrade route not found to module in application callstack")
	}

	versionMetadata, err := types.MetadataFromVersion(proposedVersion)
	if err != nil {
		// since it is valid for fee version to not be specified, the upgrade version may be for a middleware
		// or application further down in the stack. Thus, passthrough to next middleware or application in callstack.
		return cbs.OnChanUpgradeInit(ctx, portID, channelID, proposedOrder, proposedConnectionHops, proposedVersion)
	}

	if versionMetadata.FeeVersion != types.Version {
		return "", errorsmod.Wrapf(types.ErrInvalidVersion, "expected %s, got %s", types.Version, versionMetadata.FeeVersion)
	}

	appVersion, err := cbs.OnChanUpgradeInit(ctx, portID, channelID, proposedOrder, proposedConnectionHops, versionMetadata.AppVersion)
	if err != nil {
		return "", err
	}

	versionMetadata.AppVersion = appVersion
	versionBz, err := types.ModuleCdc.MarshalJSON(&versionMetadata)
	if err != nil {
		return "", err
	}

	return string(versionBz), nil
}

// OnChanUpgradeTry implement s the IBCModule interface
func (im IBCMiddleware) OnChanUpgradeTry(ctx sdk.Context, portID, channelID string, proposedOrder channeltypes.Order, proposedConnectionHops []string, counterpartyVersion string) (string, error) {
	cbs, ok := im.app.(porttypes.UpgradableModule)
	if !ok {
		return "", errorsmod.Wrap(porttypes.ErrInvalidRoute, "upgrade route not found to module in application callstack")
	}

	versionMetadata, err := types.MetadataFromVersion(counterpartyVersion)
	if err != nil {
		// since it is valid for fee version to not be specified, the counterparty upgrade version may be for a middleware
		// or application further down in the stack. Thus, passthrough to next middleware or application in callstack.
		return cbs.OnChanUpgradeTry(ctx, portID, channelID, proposedOrder, proposedConnectionHops, counterpartyVersion)
	}

	if versionMetadata.FeeVersion != types.Version {
		return "", errorsmod.Wrapf(types.ErrInvalidVersion, "expected %s, got %s", types.Version, versionMetadata.FeeVersion)
	}

	appVersion, err := cbs.OnChanUpgradeTry(ctx, portID, channelID, proposedOrder, proposedConnectionHops, versionMetadata.AppVersion)
	if err != nil {
		return "", err
	}

	versionMetadata.AppVersion = appVersion
	versionBz, err := types.ModuleCdc.MarshalJSON(&versionMetadata)
	if err != nil {
		return "", err
	}

	return string(versionBz), nil
}

// OnChanUpgradeAck implements the IBCModule interface
func (im IBCMiddleware) OnChanUpgradeAck(ctx sdk.Context, portID, channelID, counterpartyVersion string) error {
	cbs, ok := im.app.(porttypes.UpgradableModule)
	if !ok {
		return errorsmod.Wrap(porttypes.ErrInvalidRoute, "upgrade route not found to module in application callstack")
	}

	versionMetadata, err := types.MetadataFromVersion(counterpartyVersion)
	if err != nil {
		// since it is valid for fee version to not be specified, the counterparty upgrade version may be for a middleware
		// or application further down in the stack. Thus, passthrough to next middleware or application in callstack.
		return cbs.OnChanUpgradeAck(ctx, portID, channelID, counterpartyVersion)
	}

	if versionMetadata.FeeVersion != types.Version {
		return errorsmod.Wrapf(types.ErrInvalidVersion, "expected counterparty fee version: %s, got: %s", types.Version, versionMetadata.FeeVersion)
	}

	// call underlying app's OnChanUpgradeAck callback with the counterparty app version.
	return cbs.OnChanUpgradeAck(ctx, portID, channelID, versionMetadata.AppVersion)
}

// OnChanUpgradeOpen implements the IBCModule interface
func (im IBCMiddleware) OnChanUpgradeOpen(ctx sdk.Context, portID, channelID string, proposedOrder channeltypes.Order, proposedConnectionHops []string, proposedVersion string) {
	cbs, ok := im.app.(porttypes.UpgradableModule)
	if !ok {
		panic(errorsmod.Wrap(porttypes.ErrInvalidRoute, "upgrade route not found to module in application callstack"))
	}

	versionMetadata, err := types.MetadataFromVersion(proposedVersion)
	if err != nil {
		// set fee disabled and passthrough to the next middleware or application in callstack.
		im.keeper.DeleteFeeEnabled(ctx, portID, channelID)
		cbs.OnChanUpgradeOpen(ctx, portID, channelID, proposedOrder, proposedConnectionHops, proposedVersion)
		return
	}

	// set fee enabled and passthrough to the next middleware of application in callstack.
	im.keeper.SetFeeEnabled(ctx, portID, channelID)
	cbs.OnChanUpgradeOpen(ctx, portID, channelID, proposedOrder, proposedConnectionHops, versionMetadata.AppVersion)
}

// SendPacket implements the ICS4 Wrapper interface
func (im IBCMiddleware) SendPacket(
	ctx sdk.Context,
	chanCap *capabilitytypes.Capability,
	sourcePort string,
	sourceChannel string,
	timeoutHeight clienttypes.Height,
	timeoutTimestamp uint64,
	data []byte,
) (uint64, error) {
	return im.keeper.SendPacket(ctx, chanCap, sourcePort, sourceChannel, timeoutHeight, timeoutTimestamp, data)
}

// WriteAcknowledgement implements the ICS4 Wrapper interface
func (im IBCMiddleware) WriteAcknowledgement(
	ctx sdk.Context,
	chanCap *capabilitytypes.Capability,
	packet exported.PacketI,
	ack exported.Acknowledgement,
) error {
	return im.keeper.WriteAcknowledgement(ctx, chanCap, packet, ack)
}

// GetAppVersion returns the application version of the underlying application
func (im IBCMiddleware) GetAppVersion(ctx sdk.Context, portID, channelID string) (string, bool) {
	return im.keeper.GetAppVersion(ctx, portID, channelID)
}

// UnmarshalPacketData attempts to use the underlying app to unmarshal the packet data.
// If the underlying app does not support the PacketDataUnmarshaler interface, an error is returned.
// This function implements the optional PacketDataUnmarshaler interface required for ADR 008 support.
func (im IBCMiddleware) UnmarshalPacketData(bz []byte) (interface{}, error) {
	unmarshaler, ok := im.app.(porttypes.PacketDataUnmarshaler)
	if !ok {
		return nil, errorsmod.Wrapf(types.ErrUnsupportedAction, "underlying app does not implement %T", (*porttypes.PacketDataUnmarshaler)(nil))
	}

	return unmarshaler.UnmarshalPacketData(bz)
}
