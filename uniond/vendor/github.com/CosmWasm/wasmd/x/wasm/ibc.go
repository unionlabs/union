package wasm

import (
	"math"
	"context"

	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"
	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	porttypes "github.com/cosmos/ibc-go/v8/modules/core/05-port/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	ibcexported "github.com/cosmos/ibc-go/v8/modules/core/exported"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/x/wasm/keeper"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

// DefaultMaxIBCCallbackGas is the default value of maximum gas that an IBC callback can use.
// If the callback uses more gas, it will be out of gas and the contract state changes will be reverted,
// but the transaction will be committed.
// Pass this to the callbacks middleware or choose a custom value.
const DefaultMaxIBCCallbackGas = uint64(1_000_000)

var _ porttypes.IBCModule = IBCHandler{}

// internal interface that is implemented by ibc middleware
type appVersionGetter interface {
	// GetAppVersion returns the application level version with all middleware data stripped out
	GetAppVersion(ctx context.Context, portID, channelID string) (string, bool)
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
	ctx context.Context,
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
	contractAddr, err := keeper.ContractFromPortID(portID)
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
	acceptedVersion, err := i.keeper.OnOpenChannel(sdk.UnwrapSDKContext(ctx), contractAddr, msg)
	if err != nil {
		return "", err
	}
	if acceptedVersion == "" { // accept incoming version when nothing returned by contract
		if version == "" {
			return "", types.ErrEmpty.Wrap("version")
		}
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
	ctx context.Context,
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

	contractAddr, err := keeper.ContractFromPortID(portID)
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
	ctx context.Context,
	portID, channelID string,
	counterpartyChannelID string,
	counterpartyVersion string,
) error {
	contractAddr, err := keeper.ContractFromPortID(portID)
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
func (i IBCHandler) OnChanOpenConfirm(ctx context.Context, portID, channelID string) error {
	contractAddr, err := keeper.ContractFromPortID(portID)
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
func (i IBCHandler) OnChanCloseInit(ctx context.Context, portID, channelID string) error {
	contractAddr, err := keeper.ContractFromPortID(portID)
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
func (i IBCHandler) OnChanCloseConfirm(ctx context.Context, portID, channelID string) error {
	// counterparty has closed the channel
	contractAddr, err := keeper.ContractFromPortID(portID)
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
	ctx context.Context,
	packet channeltypes.Packet,
	relayer sdk.AccAddress,
) ibcexported.Acknowledgement {
	contractAddr, err := keeper.ContractFromPortID(packet.DestinationPort)
	if err != nil {
		// this must not happen as ports were registered before
		panic(errorsmod.Wrapf(err, "contract port id"))
	}

	em := sdk.NewEventManager()
	msg := wasmvmtypes.IBCPacketReceiveMsg{Packet: newIBCPacket(packet), Relayer: relayer.String()}
	ack, err := i.keeper.OnRecvPacket(sdk.UnwrapSDKContext(ctx).WithEventManager(em), contractAddr, msg)
	if err != nil {
		ack = CreateErrorAcknowledgement(err)
		// the state gets reverted, so we drop all captured events
	} else if ack == nil || ack.Success() {
		// emit all contract and submessage events on success
		// nil ack is a success case, see: https://github.com/cosmos/ibc-go/blob/v7.0.0/modules/core/keeper/msg_server.go#L453
		sdk.UnwrapSDKContext(ctx).EventManager().EmitEvents(em.Events())
	}
	types.EmitAcknowledgementEvent(sdk.UnwrapSDKContext(ctx), contractAddr, ack, err)
	return ack
}

// OnAcknowledgementPacket implements the IBCModule interface
func (i IBCHandler) OnAcknowledgementPacket(
	ctx context.Context,
	packet channeltypes.Packet,
	acknowledgement []byte,
	relayer sdk.AccAddress,
) error {
	contractAddr, err := keeper.ContractFromPortID(packet.SourcePort)
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
func (i IBCHandler) OnTimeoutPacket(ctx context.Context, packet channeltypes.Packet, relayer sdk.AccAddress) error {
	contractAddr, err := keeper.ContractFromPortID(packet.SourcePort)
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

// IBCSendPacketCallback implements the IBC Callbacks ContractKeeper interface
// see https://github.com/cosmos/ibc-go/blob/main/docs/architecture/adr-008-app-caller-cbs.md#contractkeeper
func (i IBCHandler) IBCSendPacketCallback(
	cachedCtx sdk.Context,
	sourcePort string,
	sourceChannel string,
	timeoutHeight clienttypes.Height,
	timeoutTimestamp uint64,
	packetData []byte,
	contractAddress,
	packetSenderAddress string,
) error {
	_, err := validateSender(contractAddress, packetSenderAddress)
	if err != nil {
		return err
	}

	// no-op, since we are not interested in this callback
	return nil
}

// IBCOnAcknowledgementPacketCallback implements the IBC Callbacks ContractKeeper interface
// see https://github.com/cosmos/ibc-go/blob/main/docs/architecture/adr-008-app-caller-cbs.md#contractkeeper
func (i IBCHandler) IBCOnAcknowledgementPacketCallback(
	cachedCtx sdk.Context,
	packet channeltypes.Packet,
	acknowledgement []byte,
	relayer sdk.AccAddress,
	contractAddress,
	packetSenderAddress string,
) error {
	contractAddr, err := validateSender(contractAddress, packetSenderAddress)
	if err != nil {
		return err
	}

	msg := wasmvmtypes.IBCSourceCallbackMsg{
		Acknowledgement: &wasmvmtypes.IBCAckCallbackMsg{
			Acknowledgement: wasmvmtypes.IBCAcknowledgement{Data: acknowledgement},
			OriginalPacket:  newIBCPacket(packet),
			Relayer:         relayer.String(),
		},
	}
	err = i.keeper.IBCSourceCallback(cachedCtx, contractAddr, msg)
	if err != nil {
		return errorsmod.Wrap(err, "on source chain callback ack")
	}

	return nil
}

// IBCOnTimeoutPacketCallback implements the IBC Callbacks ContractKeeper interface
// see https://github.com/cosmos/ibc-go/blob/main/docs/architecture/adr-008-app-caller-cbs.md#contractkeeper
func (i IBCHandler) IBCOnTimeoutPacketCallback(
	cachedCtx sdk.Context,
	packet channeltypes.Packet,
	relayer sdk.AccAddress,
	contractAddress,
	packetSenderAddress string,
) error {
	contractAddr, err := validateSender(contractAddress, packetSenderAddress)
	if err != nil {
		return err
	}

	msg := wasmvmtypes.IBCSourceCallbackMsg{
		Timeout: &wasmvmtypes.IBCTimeoutCallbackMsg{
			Packet:  newIBCPacket(packet),
			Relayer: relayer.String(),
		},
	}
	err = i.keeper.IBCSourceCallback(cachedCtx, contractAddr, msg)
	if err != nil {
		return errorsmod.Wrap(err, "on source chain callback timeout")
	}
	return nil
}

// IBCReceivePacketCallback implements the IBC Callbacks ContractKeeper interface
// see https://github.com/cosmos/ibc-go/blob/main/docs/architecture/adr-008-app-caller-cbs.md#contractkeeper
func (i IBCHandler) IBCReceivePacketCallback(
	cachedCtx sdk.Context,
	packet ibcexported.PacketI,
	ack ibcexported.Acknowledgement,
	contractAddress string,
) error {
	// sender validation makes no sense here, as the receiver is never the sender
	contractAddr, err := sdk.AccAddressFromBech32(contractAddress)
	if err != nil {
		return err
	}

	msg := wasmvmtypes.IBCDestinationCallbackMsg{
		Ack:    wasmvmtypes.IBCAcknowledgement{Data: ack.Acknowledgement()},
		Packet: newIBCPacket(packet),
	}

	err = i.keeper.IBCDestinationCallback(cachedCtx, contractAddr, msg)
	if err != nil {
		return errorsmod.Wrap(err, "on destination chain callback")
	}

	return nil
}

func validateSender(contractAddr, senderAddr string) (sdk.AccAddress, error) {
	contractAddress, err := sdk.AccAddressFromBech32(contractAddr)
	if err != nil {
		return nil, errorsmod.Wrapf(err, "contract address")
	}
	senderAddress, err := sdk.AccAddressFromBech32(senderAddr)
	if err != nil {
		return nil, errorsmod.Wrapf(err, "packet sender address")
	}

	// We only allow the contract that sent the message to receive source chain callbacks for it.
	if !contractAddress.Equals(senderAddress) {
		return nil, errorsmod.Wrapf(types.ErrExecuteFailed, "contract address %s does not match packet sender %s", contractAddr, senderAddress)
	}

	return contractAddress, nil
}

func newIBCPacket(packet ibcexported.PacketI) wasmvmtypes.IBCPacket {
	timeout := wasmvmtypes.IBCTimeout{
		Timestamp: packet.GetTimeoutTimestamp(),
	}
	timeoutHeight := packet.GetTimeoutHeight()
	if !timeoutHeight.IsZero() {
		timeout.Block = &wasmvmtypes.IBCTimeoutBlock{
			Height:   timeoutHeight.GetRevisionHeight(),
			Revision: timeoutHeight.GetRevisionNumber(),
		}
	}

	return wasmvmtypes.IBCPacket{
		Data:     packet.GetData(),
		Src:      wasmvmtypes.IBCEndpoint{ChannelID: packet.GetSourceChannel(), PortID: packet.GetSourcePort()},
		Dest:     wasmvmtypes.IBCEndpoint{ChannelID: packet.GetDestChannel(), PortID: packet.GetDestPort()},
		Sequence: packet.GetSequence(),
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

// CreateErrorAcknowledgement turns an error into an error acknowledgement.
//
// This function is x/wasm specific and might include the full error text in the future
// as we gain confidence that it is deterministic. Don't use it in other contexts.
// See also https://github.com/CosmWasm/wasmd/issues/1740.
func CreateErrorAcknowledgement(err error) ibcexported.Acknowledgement {
	return channeltypes.NewErrorAcknowledgementWithCodespace(err)
}
