package wasm

import (
	"math"

	errorsmod "cosmossdk.io/errors"
	wasmvmtypes "github.com/CosmWasm/wasmvm/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	capabilitytypes "github.com/cosmos/cosmos-sdk/x/capability/types"
	channeltypes "github.com/cosmos/ibc-go/v7/modules/core/04-channel/types"
	porttypes "github.com/cosmos/ibc-go/v7/modules/core/05-port/types"
	host "github.com/cosmos/ibc-go/v7/modules/core/24-host"
	ibcexported "github.com/cosmos/ibc-go/v7/modules/core/exported"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

var _ porttypes.IBCModule = IBCHandler{}

// internal interface that is implemented by ibc middleware
type appVersionGetter interface {
	// GetAppVersion returns the application level version with all middleware data stripped out
	GetAppVersion(ctx sdk.Context, portID, channelID string) (string, bool)
}

type IBCHandler struct {
	keeper           types.IBCContractKeeper
	channelKeeper    types.ChannelKeeper
	appVersionGetter appVersionGetter
}

func NewIBCHandler(k types.IBCContractKeeper, ck types.ChannelKeeper, vg appVersionGetter) IBCHandler {
	return IBCHandler{keeper: k, channelKeeper: ck, appVersionGetter: vg}
}

// OnChanOpenInit implements the IBCModule interface
func (i IBCHandler) OnChanOpenInit(
	ctx sdk.Context,
	order channeltypes.Order,
	connectionHops []string,
	portID string,
	channelID string,
	chanCap *capabilitytypes.Capability,
	counterParty channeltypes.Counterparty,
	version string,
) (string, error) {
	// ensure port, version, capability
	if err := ValidateChannelParams(channelID); err != nil {
		return "", err
	}
	contractAddr, err := ContractFromPortID(portID)
	if err != nil {
		return "", errorsmod.Wrapf(err, "contract port id")
	}

	msg := wasmvmtypes.IBCChannelOpenMsg{
		OpenInit: &wasmvmtypes.IBCOpenInit{
			Channel: wasmvmtypes.IBCChannel{
				Endpoint:             wasmvmtypes.IBCEndpoint{PortID: portID, ChannelID: channelID},
				CounterpartyEndpoint: wasmvmtypes.IBCEndpoint{PortID: counterParty.PortId, ChannelID: counterParty.ChannelId},
				Order:                order.String(),
				// DESIGN V3: this may be "" ??
				Version:      version,
				ConnectionID: connectionHops[0], // At the moment this list must be of length 1. In the future multi-hop channels may be supported.
			},
		},
	}

	// Allow contracts to return a version (or default to proposed version if unset)
	acceptedVersion, err := i.keeper.OnOpenChannel(ctx, contractAddr, msg)
	if err != nil {
		return "", err
	}
	if acceptedVersion == "" { // accept incoming version when nothing returned by contract
		acceptedVersion = version
	}

	// Claim channel capability passed back by IBC module
	if err := i.keeper.ClaimCapability(ctx, chanCap, host.ChannelCapabilityPath(portID, channelID)); err != nil {
		return "", errorsmod.Wrap(err, "claim capability")
	}
	return acceptedVersion, nil
}

// OnChanOpenTry implements the IBCModule interface
func (i IBCHandler) OnChanOpenTry(
	ctx sdk.Context,
	order channeltypes.Order,
	connectionHops []string,
	portID, channelID string,
	chanCap *capabilitytypes.Capability,
	counterParty channeltypes.Counterparty,
	counterpartyVersion string,
) (string, error) {
	// ensure port, version, capability
	if err := ValidateChannelParams(channelID); err != nil {
		return "", err
	}

	contractAddr, err := ContractFromPortID(portID)
	if err != nil {
		return "", errorsmod.Wrapf(err, "contract port id")
	}

	msg := wasmvmtypes.IBCChannelOpenMsg{
		OpenTry: &wasmvmtypes.IBCOpenTry{
			Channel: wasmvmtypes.IBCChannel{
				Endpoint:             wasmvmtypes.IBCEndpoint{PortID: portID, ChannelID: channelID},
				CounterpartyEndpoint: wasmvmtypes.IBCEndpoint{PortID: counterParty.PortId, ChannelID: counterParty.ChannelId},
				Order:                order.String(),
				Version:              counterpartyVersion,
				ConnectionID:         connectionHops[0], // At the moment this list must be of length 1. In the future multi-hop channels may be supported.
			},
			CounterpartyVersion: counterpartyVersion,
		},
	}

	// Allow contracts to return a version (or default to counterpartyVersion if unset)
	version, err := i.keeper.OnOpenChannel(ctx, contractAddr, msg)
	if err != nil {
		return "", err
	}
	if version == "" {
		version = counterpartyVersion
	}

	// Module may have already claimed capability in OnChanOpenInit in the case of crossing hellos
	// (ie chainA and chainB both call ChanOpenInit before one of them calls ChanOpenTry)
	// If module can already authenticate the capability then module already owns it, so we don't need to claim
	// Otherwise, module does not have channel capability, and we must claim it from IBC
	if !i.keeper.AuthenticateCapability(ctx, chanCap, host.ChannelCapabilityPath(portID, channelID)) {
		// Only claim channel capability passed back by IBC module if we do not already own it
		if err := i.keeper.ClaimCapability(ctx, chanCap, host.ChannelCapabilityPath(portID, channelID)); err != nil {
			return "", errorsmod.Wrap(err, "claim capability")
		}
	}

	return version, nil
}

// OnChanOpenAck implements the IBCModule interface
func (i IBCHandler) OnChanOpenAck(
	ctx sdk.Context,
	portID, channelID string,
	counterpartyChannelID string,
	counterpartyVersion string,
) error {
	contractAddr, err := ContractFromPortID(portID)
	if err != nil {
		return errorsmod.Wrapf(err, "contract port id")
	}
	channelInfo, ok := i.channelKeeper.GetChannel(ctx, portID, channelID)
	if !ok {
		return errorsmod.Wrapf(channeltypes.ErrChannelNotFound, "port ID (%s) channel ID (%s)", portID, channelID)
	}
	channelInfo.Counterparty.ChannelId = counterpartyChannelID

	appVersion, ok := i.appVersionGetter.GetAppVersion(ctx, portID, channelID)
	if !ok {
		return errorsmod.Wrapf(channeltypes.ErrInvalidChannelVersion, "port ID (%s) channel ID (%s)", portID, channelID)
	}

	msg := wasmvmtypes.IBCChannelConnectMsg{
		OpenAck: &wasmvmtypes.IBCOpenAck{
			Channel:             toWasmVMChannel(portID, channelID, channelInfo, appVersion),
			CounterpartyVersion: counterpartyVersion,
		},
	}
	return i.keeper.OnConnectChannel(ctx, contractAddr, msg)
}

// OnChanOpenConfirm implements the IBCModule interface
func (i IBCHandler) OnChanOpenConfirm(ctx sdk.Context, portID, channelID string) error {
	contractAddr, err := ContractFromPortID(portID)
	if err != nil {
		return errorsmod.Wrapf(err, "contract port id")
	}
	channelInfo, ok := i.channelKeeper.GetChannel(ctx, portID, channelID)
	if !ok {
		return errorsmod.Wrapf(channeltypes.ErrChannelNotFound, "port ID (%s) channel ID (%s)", portID, channelID)
	}
	appVersion, ok := i.appVersionGetter.GetAppVersion(ctx, portID, channelID)
	if !ok {
		return errorsmod.Wrapf(channeltypes.ErrInvalidChannelVersion, "port ID (%s) channel ID (%s)", portID, channelID)
	}
	msg := wasmvmtypes.IBCChannelConnectMsg{
		OpenConfirm: &wasmvmtypes.IBCOpenConfirm{
			Channel: toWasmVMChannel(portID, channelID, channelInfo, appVersion),
		},
	}
	return i.keeper.OnConnectChannel(ctx, contractAddr, msg)
}

// OnChanCloseInit implements the IBCModule interface
func (i IBCHandler) OnChanCloseInit(ctx sdk.Context, portID, channelID string) error {
	contractAddr, err := ContractFromPortID(portID)
	if err != nil {
		return errorsmod.Wrapf(err, "contract port id")
	}
	channelInfo, ok := i.channelKeeper.GetChannel(ctx, portID, channelID)
	if !ok {
		return errorsmod.Wrapf(channeltypes.ErrChannelNotFound, "port ID (%s) channel ID (%s)", portID, channelID)
	}
	appVersion, ok := i.appVersionGetter.GetAppVersion(ctx, portID, channelID)
	if !ok {
		return errorsmod.Wrapf(channeltypes.ErrInvalidChannelVersion, "port ID (%s) channel ID (%s)", portID, channelID)
	}

	msg := wasmvmtypes.IBCChannelCloseMsg{
		CloseInit: &wasmvmtypes.IBCCloseInit{Channel: toWasmVMChannel(portID, channelID, channelInfo, appVersion)},
	}
	err = i.keeper.OnCloseChannel(ctx, contractAddr, msg)
	if err != nil {
		return err
	}
	// emit events?

	return err
}

// OnChanCloseConfirm implements the IBCModule interface
func (i IBCHandler) OnChanCloseConfirm(ctx sdk.Context, portID, channelID string) error {
	// counterparty has closed the channel
	contractAddr, err := ContractFromPortID(portID)
	if err != nil {
		return errorsmod.Wrapf(err, "contract port id")
	}
	channelInfo, ok := i.channelKeeper.GetChannel(ctx, portID, channelID)
	if !ok {
		return errorsmod.Wrapf(channeltypes.ErrChannelNotFound, "port ID (%s) channel ID (%s)", portID, channelID)
	}
	appVersion, ok := i.appVersionGetter.GetAppVersion(ctx, portID, channelID)
	if !ok {
		return errorsmod.Wrapf(channeltypes.ErrInvalidChannelVersion, "port ID (%s) channel ID (%s)", portID, channelID)
	}

	msg := wasmvmtypes.IBCChannelCloseMsg{
		CloseConfirm: &wasmvmtypes.IBCCloseConfirm{Channel: toWasmVMChannel(portID, channelID, channelInfo, appVersion)},
	}
	err = i.keeper.OnCloseChannel(ctx, contractAddr, msg)
	if err != nil {
		return err
	}
	// emit events?

	return err
}

func toWasmVMChannel(portID, channelID string, channelInfo channeltypes.Channel, appVersion string) wasmvmtypes.IBCChannel {
	return wasmvmtypes.IBCChannel{
		Endpoint:             wasmvmtypes.IBCEndpoint{PortID: portID, ChannelID: channelID},
		CounterpartyEndpoint: wasmvmtypes.IBCEndpoint{PortID: channelInfo.Counterparty.PortId, ChannelID: channelInfo.Counterparty.ChannelId},
		Order:                channelInfo.Ordering.String(),
		Version:              appVersion,
		ConnectionID:         channelInfo.ConnectionHops[0], // At the moment this list must be of length 1. In the future multi-hop channels may be supported.
	}
}

// OnRecvPacket implements the IBCModule interface
func (i IBCHandler) OnRecvPacket(
	ctx sdk.Context,
	packet channeltypes.Packet,
	relayer sdk.AccAddress,
) ibcexported.Acknowledgement {
	contractAddr, err := ContractFromPortID(packet.DestinationPort)
	if err != nil {
		// this must not happen as ports were registered before
		panic(errorsmod.Wrapf(err, "contract port id"))
	}

	em := sdk.NewEventManager()
	msg := wasmvmtypes.IBCPacketReceiveMsg{Packet: newIBCPacket(packet), Relayer: relayer.String()}
	ack, err := i.keeper.OnRecvPacket(ctx.WithEventManager(em), contractAddr, msg)
	if err != nil {
		ack = channeltypes.NewErrorAcknowledgement(err)
		// the state gets reverted, so we drop all captured events
	} else if ack == nil || ack.Success() {
		// emit all contract and submessage events on success
		// nil ack is a success case, see: https://github.com/cosmos/ibc-go/blob/v7.0.0/modules/core/keeper/msg_server.go#L453
		ctx.EventManager().EmitEvents(em.Events())
	}
	types.EmitAcknowledgementEvent(ctx, contractAddr, ack, err)
	return ack
}

// OnAcknowledgementPacket implements the IBCModule interface
func (i IBCHandler) OnAcknowledgementPacket(
	ctx sdk.Context,
	packet channeltypes.Packet,
	acknowledgement []byte,
	relayer sdk.AccAddress,
) error {
	contractAddr, err := ContractFromPortID(packet.SourcePort)
	if err != nil {
		return errorsmod.Wrapf(err, "contract port id")
	}

	err = i.keeper.OnAckPacket(ctx, contractAddr, wasmvmtypes.IBCPacketAckMsg{
		Acknowledgement: wasmvmtypes.IBCAcknowledgement{Data: acknowledgement},
		OriginalPacket:  newIBCPacket(packet),
		Relayer:         relayer.String(),
	})
	if err != nil {
		return errorsmod.Wrap(err, "on ack")
	}
	return nil
}

// OnTimeoutPacket implements the IBCModule interface
func (i IBCHandler) OnTimeoutPacket(ctx sdk.Context, packet channeltypes.Packet, relayer sdk.AccAddress) error {
	contractAddr, err := ContractFromPortID(packet.SourcePort)
	if err != nil {
		return errorsmod.Wrapf(err, "contract port id")
	}
	msg := wasmvmtypes.IBCPacketTimeoutMsg{Packet: newIBCPacket(packet), Relayer: relayer.String()}
	err = i.keeper.OnTimeoutPacket(ctx, contractAddr, msg)
	if err != nil {
		return errorsmod.Wrap(err, "on timeout")
	}
	return nil
}

func newIBCPacket(packet channeltypes.Packet) wasmvmtypes.IBCPacket {
	timeout := wasmvmtypes.IBCTimeout{
		Timestamp: packet.TimeoutTimestamp,
	}
	if !packet.TimeoutHeight.IsZero() {
		timeout.Block = &wasmvmtypes.IBCTimeoutBlock{
			Height:   packet.TimeoutHeight.RevisionHeight,
			Revision: packet.TimeoutHeight.RevisionNumber,
		}
	}

	return wasmvmtypes.IBCPacket{
		Data:     packet.Data,
		Src:      wasmvmtypes.IBCEndpoint{ChannelID: packet.SourceChannel, PortID: packet.SourcePort},
		Dest:     wasmvmtypes.IBCEndpoint{ChannelID: packet.DestinationChannel, PortID: packet.DestinationPort},
		Sequence: packet.Sequence,
		Timeout:  timeout,
	}
}

func ValidateChannelParams(channelID string) error {
	// NOTE: for escrow address security only 2^32 channels are allowed to be created
	// Issue: https://github.com/cosmos/cosmos-sdk/issues/7737
	channelSequence, err := channeltypes.ParseChannelSequence(channelID)
	if err != nil {
		return err
	}
	if channelSequence > math.MaxUint32 {
		return errorsmod.Wrapf(types.ErrMaxIBCChannels, "channel sequence %d is greater than max allowed transfer channels %d", channelSequence, math.MaxUint32)
	}
	return nil
}
