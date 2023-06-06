package keeper

import (
	"errors"
	"fmt"

	errorsmod "cosmossdk.io/errors"

	wasmvmtypes "github.com/CosmWasm/wasmvm/types"
	"github.com/cosmos/cosmos-sdk/baseapp"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	channeltypes "github.com/cosmos/ibc-go/v7/modules/core/04-channel/types"
	host "github.com/cosmos/ibc-go/v7/modules/core/24-host"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

// msgEncoder is an extension point to customize encodings
type msgEncoder interface {
	// Encode converts wasmvm message to n cosmos message types
	Encode(ctx sdk.Context, contractAddr sdk.AccAddress, contractIBCPortID string, msg wasmvmtypes.CosmosMsg) ([]sdk.Msg, error)
}

// MessageRouter ADR 031 request type routing
type MessageRouter interface {
	Handler(msg sdk.Msg) baseapp.MsgServiceHandler
}

// SDKMessageHandler can handles messages that can be encoded into sdk.Message types and routed.
type SDKMessageHandler struct {
	router   MessageRouter
	encoders msgEncoder
}

// NewDefaultMessageHandler constructor
func NewDefaultMessageHandler(
	router MessageRouter,
	ics4Wrapper types.ICS4Wrapper,
	channelKeeper types.ChannelKeeper,
	capabilityKeeper types.CapabilityKeeper,
	bankKeeper types.Burner,
	unpacker codectypes.AnyUnpacker,
	portSource types.ICS20TransferPortSource,
	customEncoders ...*MessageEncoders,
) Messenger {
	encoders := DefaultEncoders(unpacker, portSource)
	for _, e := range customEncoders {
		encoders = encoders.Merge(e)
	}
	return NewMessageHandlerChain(
		NewSDKMessageHandler(router, encoders),
		NewIBCRawPacketHandler(ics4Wrapper, channelKeeper, capabilityKeeper),
		NewBurnCoinMessageHandler(bankKeeper),
	)
}

func NewSDKMessageHandler(router MessageRouter, encoders msgEncoder) SDKMessageHandler {
	return SDKMessageHandler{
		router:   router,
		encoders: encoders,
	}
}

func (h SDKMessageHandler) DispatchMsg(ctx sdk.Context, contractAddr sdk.AccAddress, contractIBCPortID string, msg wasmvmtypes.CosmosMsg) (events []sdk.Event, data [][]byte, err error) {
	sdkMsgs, err := h.encoders.Encode(ctx, contractAddr, contractIBCPortID, msg)
	if err != nil {
		return nil, nil, err
	}
	for _, sdkMsg := range sdkMsgs {
		res, err := h.handleSdkMessage(ctx, contractAddr, sdkMsg)
		if err != nil {
			return nil, nil, err
		}
		// append data
		data = append(data, res.Data)
		// append events
		sdkEvents := make([]sdk.Event, len(res.Events))
		for i := range res.Events {
			sdkEvents[i] = sdk.Event(res.Events[i])
		}
		events = append(events, sdkEvents...)
	}
	return
}

func (h SDKMessageHandler) handleSdkMessage(ctx sdk.Context, contractAddr sdk.Address, msg sdk.Msg) (*sdk.Result, error) {
	if err := msg.ValidateBasic(); err != nil {
		return nil, err
	}
	// make sure this account can send it
	for _, acct := range msg.GetSigners() {
		if !acct.Equals(contractAddr) {
			return nil, errorsmod.Wrap(sdkerrors.ErrUnauthorized, "contract doesn't have permission")
		}
	}

	// find the handler and execute it
	if handler := h.router.Handler(msg); handler != nil {
		// ADR 031 request type routing
		msgResult, err := handler(ctx, msg)
		return msgResult, err
	}
	// legacy sdk.Msg routing
	// Assuming that the app developer has migrated all their Msgs to
	// proto messages and has registered all `Msg services`, then this
	// path should never be called, because all those Msgs should be
	// registered within the `msgServiceRouter` already.
	return nil, errorsmod.Wrapf(sdkerrors.ErrUnknownRequest, "can't route message %+v", msg)
}

// MessageHandlerChain defines a chain of handlers that are called one by one until it can be handled.
type MessageHandlerChain struct {
	handlers []Messenger
}

func NewMessageHandlerChain(first Messenger, others ...Messenger) *MessageHandlerChain {
	r := &MessageHandlerChain{handlers: append([]Messenger{first}, others...)}
	for i := range r.handlers {
		if r.handlers[i] == nil {
			panic(fmt.Sprintf("handler must not be nil at position : %d", i))
		}
	}
	return r
}

// DispatchMsg dispatch message and calls chained handlers one after another in
// order to find the right one to process given message. If a handler cannot
// process given message (returns ErrUnknownMsg), its result is ignored and the
// next handler is executed.
func (m MessageHandlerChain) DispatchMsg(ctx sdk.Context, contractAddr sdk.AccAddress, contractIBCPortID string, msg wasmvmtypes.CosmosMsg) ([]sdk.Event, [][]byte, error) {
	for _, h := range m.handlers {
		events, data, err := h.DispatchMsg(ctx, contractAddr, contractIBCPortID, msg)
		switch {
		case err == nil:
			return events, data, nil
		case errors.Is(err, types.ErrUnknownMsg):
			continue
		default:
			return events, data, err
		}
	}
	return nil, nil, errorsmod.Wrap(types.ErrUnknownMsg, "no handler found")
}

// IBCRawPacketHandler handles IBC.SendPacket messages which are published to an IBC channel.
type IBCRawPacketHandler struct {
	ics4Wrapper      types.ICS4Wrapper
	channelKeeper    types.ChannelKeeper
	capabilityKeeper types.CapabilityKeeper
}

// NewIBCRawPacketHandler constructor
func NewIBCRawPacketHandler(ics4Wrapper types.ICS4Wrapper, channelKeeper types.ChannelKeeper, capabilityKeeper types.CapabilityKeeper) IBCRawPacketHandler {
	return IBCRawPacketHandler{
		ics4Wrapper:      ics4Wrapper,
		channelKeeper:    channelKeeper,
		capabilityKeeper: capabilityKeeper,
	}
}

// DispatchMsg publishes a raw IBC packet onto the channel.
func (h IBCRawPacketHandler) DispatchMsg(ctx sdk.Context, _ sdk.AccAddress, contractIBCPortID string, msg wasmvmtypes.CosmosMsg) ([]sdk.Event, [][]byte, error) {
	if msg.IBC == nil || msg.IBC.SendPacket == nil {
		return nil, nil, types.ErrUnknownMsg
	}
	if contractIBCPortID == "" {
		return nil, nil, errorsmod.Wrapf(types.ErrUnsupportedForContract, "ibc not supported")
	}
	contractIBCChannelID := msg.IBC.SendPacket.ChannelID
	if contractIBCChannelID == "" {
		return nil, nil, errorsmod.Wrapf(types.ErrEmpty, "ibc channel")
	}

	channelCap, ok := h.capabilityKeeper.GetCapability(ctx, host.ChannelCapabilityPath(contractIBCPortID, contractIBCChannelID))
	if !ok {
		return nil, nil, errorsmod.Wrap(channeltypes.ErrChannelCapabilityNotFound, "module does not own channel capability")
	}
	seq, err := h.ics4Wrapper.SendPacket(ctx, channelCap, contractIBCPortID, contractIBCChannelID, ConvertWasmIBCTimeoutHeightToCosmosHeight(msg.IBC.SendPacket.Timeout.Block), msg.IBC.SendPacket.Timeout.Timestamp, msg.IBC.SendPacket.Data)
	if err != nil {
		return nil, nil, errorsmod.Wrap(err, "channel")
	}
	moduleLogger(ctx).Debug("ibc packet set", "seq", seq)

	resp := &types.MsgIBCSendResponse{Sequence: seq}
	val, err := resp.Marshal()
	if err != nil {
		return nil, nil, errorsmod.Wrap(err, "failed to marshal IBC send response")
	}

	return nil, [][]byte{val}, nil
}

var _ Messenger = MessageHandlerFunc(nil)

// MessageHandlerFunc is a helper to construct a function based message handler.
type MessageHandlerFunc func(ctx sdk.Context, contractAddr sdk.AccAddress, contractIBCPortID string, msg wasmvmtypes.CosmosMsg) (events []sdk.Event, data [][]byte, err error)

// DispatchMsg delegates dispatching of provided message into the MessageHandlerFunc.
func (m MessageHandlerFunc) DispatchMsg(ctx sdk.Context, contractAddr sdk.AccAddress, contractIBCPortID string, msg wasmvmtypes.CosmosMsg) (events []sdk.Event, data [][]byte, err error) {
	return m(ctx, contractAddr, contractIBCPortID, msg)
}

// NewBurnCoinMessageHandler handles wasmvm.BurnMsg messages
func NewBurnCoinMessageHandler(burner types.Burner) MessageHandlerFunc {
	return func(ctx sdk.Context, contractAddr sdk.AccAddress, _ string, msg wasmvmtypes.CosmosMsg) (events []sdk.Event, data [][]byte, err error) {
		if msg.Bank != nil && msg.Bank.Burn != nil {
			coins, err := ConvertWasmCoinsToSdkCoins(msg.Bank.Burn.Amount)
			if err != nil {
				return nil, nil, err
			}
			if coins.IsZero() {
				return nil, nil, types.ErrEmpty.Wrap("amount")
			}
			if err := burner.SendCoinsFromAccountToModule(ctx, contractAddr, types.ModuleName, coins); err != nil {
				return nil, nil, errorsmod.Wrap(err, "transfer to module")
			}
			if err := burner.BurnCoins(ctx, types.ModuleName, coins); err != nil {
				return nil, nil, errorsmod.Wrap(err, "burn coins")
			}
			moduleLogger(ctx).Info("Burned", "amount", coins)
			return nil, nil, nil
		}
		return nil, nil, types.ErrUnknownMsg
	}
}
