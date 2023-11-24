package wasmtesting

import (
	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types" //nolint:staticcheck
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

type MockChannelKeeper struct {
	GetChannelFn                   func(ctx sdk.Context, srcPort, srcChan string) (channel channeltypes.Channel, found bool)
	GetNextSequenceSendFn          func(ctx sdk.Context, portID, channelID string) (uint64, bool)
	ChanCloseInitFn                func(ctx sdk.Context, portID, channelID string, chanCap *capabilitytypes.Capability) error
	GetAllChannelsFn               func(ctx sdk.Context) []channeltypes.IdentifiedChannel
	SetChannelFn                   func(ctx sdk.Context, portID, channelID string, channel channeltypes.Channel)
	GetAllChannelsWithPortPrefixFn func(ctx sdk.Context, portPrefix string) []channeltypes.IdentifiedChannel
}

func (m *MockChannelKeeper) GetChannel(ctx sdk.Context, srcPort, srcChan string) (channel channeltypes.Channel, found bool) {
	if m.GetChannelFn == nil {
		panic("not supposed to be called!")
	}
	return m.GetChannelFn(ctx, srcPort, srcChan)
}

func (m *MockChannelKeeper) GetAllChannels(ctx sdk.Context) []channeltypes.IdentifiedChannel {
	if m.GetAllChannelsFn == nil {
		panic("not supposed to be called!")
	}
	return m.GetAllChannelsFn(ctx)
}

func (m *MockChannelKeeper) GetNextSequenceSend(ctx sdk.Context, portID, channelID string) (uint64, bool) {
	if m.GetNextSequenceSendFn == nil {
		panic("not supposed to be called!")
	}
	return m.GetNextSequenceSendFn(ctx, portID, channelID)
}

func (m *MockChannelKeeper) ChanCloseInit(ctx sdk.Context, portID, channelID string, chanCap *capabilitytypes.Capability) error {
	if m.ChanCloseInitFn == nil {
		panic("not supposed to be called!")
	}
	return m.ChanCloseInitFn(ctx, portID, channelID, chanCap)
}

func (m *MockChannelKeeper) GetAllChannelsWithPortPrefix(ctx sdk.Context, portPrefix string) []channeltypes.IdentifiedChannel {
	if m.GetAllChannelsWithPortPrefixFn == nil {
		panic("not expected to be called")
	}
	return m.GetAllChannelsWithPortPrefixFn(ctx, portPrefix)
}

func (m *MockChannelKeeper) SetChannel(ctx sdk.Context, portID, channelID string, channel channeltypes.Channel) {
	if m.GetChannelFn == nil {
		panic("not supposed to be called!")
	}
	m.SetChannelFn(ctx, portID, channelID, channel)
}

type MockIBCPacketSender struct {
	SendPacketFn func(ctx sdk.Context, channelCap *capabilitytypes.Capability, sourcePort, sourceChannel string, timeoutHeight clienttypes.Height, timeoutTimestamp uint64, data []byte) (uint64, error)
}

func (m *MockIBCPacketSender) SendPacket(ctx sdk.Context, channelCap *capabilitytypes.Capability, sourcePort, sourceChannel string, timeoutHeight clienttypes.Height, timeoutTimestamp uint64, data []byte) (uint64, error) {
	if m.SendPacketFn == nil {
		panic("not supposed to be called!")
	}
	return m.SendPacketFn(ctx, channelCap, sourcePort, sourceChannel, timeoutHeight, timeoutTimestamp, data)
}

func MockChannelKeeperIterator(s []channeltypes.IdentifiedChannel) func(ctx sdk.Context, cb func(channeltypes.IdentifiedChannel) bool) {
	return func(ctx sdk.Context, cb func(channeltypes.IdentifiedChannel) bool) {
		for _, channel := range s {
			stop := cb(channel)
			if stop {
				break
			}
		}
	}
}

type MockCapabilityKeeper struct {
	GetCapabilityFn          func(ctx sdk.Context, name string) (*capabilitytypes.Capability, bool)
	ClaimCapabilityFn        func(ctx sdk.Context, cap *capabilitytypes.Capability, name string) error
	AuthenticateCapabilityFn func(ctx sdk.Context, capability *capabilitytypes.Capability, name string) bool
}

func (m MockCapabilityKeeper) GetCapability(ctx sdk.Context, name string) (*capabilitytypes.Capability, bool) {
	if m.GetCapabilityFn == nil {
		panic("not supposed to be called!")
	}
	return m.GetCapabilityFn(ctx, name)
}

func (m MockCapabilityKeeper) ClaimCapability(ctx sdk.Context, cap *capabilitytypes.Capability, name string) error {
	if m.ClaimCapabilityFn == nil {
		panic("not supposed to be called!")
	}
	return m.ClaimCapabilityFn(ctx, cap, name)
}

func (m MockCapabilityKeeper) AuthenticateCapability(ctx sdk.Context, capability *capabilitytypes.Capability, name string) bool {
	if m.AuthenticateCapabilityFn == nil {
		panic("not supposed to be called!")
	}
	return m.AuthenticateCapabilityFn(ctx, capability, name)
}

var _ types.ICS20TransferPortSource = &MockIBCTransferKeeper{}

type MockIBCTransferKeeper struct {
	GetPortFn func(ctx sdk.Context) string
}

func (m MockIBCTransferKeeper) GetPort(ctx sdk.Context) string {
	if m.GetPortFn == nil {
		panic("not expected to be called")
	}
	return m.GetPortFn(ctx)
}
