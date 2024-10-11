package wasmtesting

// import (
// 	"context"
// 	"fmt"

// 	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"
// 	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
// 	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
// 	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
// 	ibcexported "github.com/cosmos/ibc-go/v8/modules/core/exported"

// 	sdk "github.com/cosmos/cosmos-sdk/types"

// 	"github.com/CosmWasm/wasmd/x/wasm/types"
// )

// type MockChannelKeeper struct {
// 	GetChannelFn                   func(ctx context.Context, srcPort, srcChan string) (channel channeltypes.Channel, found bool)
// 	GetNextSequenceSendFn          func(ctx context.Context, portID, channelID string) (uint64, bool)
// 	ChanCloseInitFn                func(ctx context.Context, portID, channelID string, chanCap *capabilitytypes.Capability) error
// 	GetAllChannelsFn               func(ctx context.Context) []channeltypes.IdentifiedChannel
// 	SetChannelFn                   func(ctx context.Context, portID, channelID string, channel channeltypes.Channel)
// 	GetAllChannelsWithPortPrefixFn func(ctx context.Context, portPrefix string) []channeltypes.IdentifiedChannel
// }

// func (m *MockChannelKeeper) GetChannel(ctx context.Context, srcPort, srcChan string) (channel channeltypes.Channel, found bool) {
// 	if m.GetChannelFn == nil {
// 		panic("not supposed to be called!")
// 	}
// 	return m.GetChannelFn(ctx, srcPort, srcChan)
// }

// func (m *MockChannelKeeper) GetAllChannels(ctx context.Context) []channeltypes.IdentifiedChannel {
// 	if m.GetAllChannelsFn == nil {
// 		panic("not supposed to be called!")
// 	}
// 	return m.GetAllChannelsFn(ctx)
// }

// func (m *MockChannelKeeper) GetNextSequenceSend(ctx context.Context, portID, channelID string) (uint64, bool) {
// 	if m.GetNextSequenceSendFn == nil {
// 		panic("not supposed to be called!")
// 	}
// 	return m.GetNextSequenceSendFn(ctx, portID, channelID)
// }

// func (m *MockChannelKeeper) ChanCloseInit(ctx context.Context, portID, channelID string, chanCap *capabilitytypes.Capability) error {
// 	if m.ChanCloseInitFn == nil {
// 		panic("not supposed to be called!")
// 	}
// 	return m.ChanCloseInitFn(ctx, portID, channelID, chanCap)
// }

// func (m *MockChannelKeeper) GetAllChannelsWithPortPrefix(ctx context.Context, portPrefix string) []channeltypes.IdentifiedChannel {
// 	if m.GetAllChannelsWithPortPrefixFn == nil {
// 		panic("not expected to be called")
// 	}
// 	return m.GetAllChannelsWithPortPrefixFn(ctx, portPrefix)
// }

// func (m *MockChannelKeeper) SetChannel(ctx context.Context, portID, channelID string, channel channeltypes.Channel) {
// 	if m.GetChannelFn == nil {
// 		panic("not supposed to be called!")
// 	}
// 	m.SetChannelFn(ctx, portID, channelID, channel)
// }

// var _ types.ICS4Wrapper = &MockICS4Wrapper{}

// type MockICS4Wrapper struct {
// 	SendPacketFn           func(ctx context.Context, channelCap *capabilitytypes.Capability, sourcePort, sourceChannel string, timeoutHeight clienttypes.Height, timeoutTimestamp uint64, data []byte) (uint64, error)
// 	WriteAcknowledgementFn func(ctx context.Context, chanCap *capabilitytypes.Capability, packet ibcexported.PacketI, acknowledgement ibcexported.Acknowledgement) error
// }

// func (m *MockICS4Wrapper) SendPacket(ctx context.Context, channelCap *capabilitytypes.Capability, sourcePort, sourceChannel string, timeoutHeight clienttypes.Height, timeoutTimestamp uint64, data []byte) (uint64, error) {
// 	if m.SendPacketFn == nil {
// 		panic("not supposed to be called!")
// 	}
// 	return m.SendPacketFn(ctx, channelCap, sourcePort, sourceChannel, timeoutHeight, timeoutTimestamp, data)
// }

// func (m *MockICS4Wrapper) WriteAcknowledgement(
// 	ctx context.Context,
// 	chanCap *capabilitytypes.Capability,
// 	packet ibcexported.PacketI,
// 	acknowledgement ibcexported.Acknowledgement,
// ) error {
// 	if m.WriteAcknowledgementFn == nil {
// 		panic("not supposed to be called!")
// 	}
// 	return m.WriteAcknowledgementFn(ctx, chanCap, packet, acknowledgement)
// }

// func MockChannelKeeperIterator(s []channeltypes.IdentifiedChannel) func(ctx context.Context, cb func(channeltypes.IdentifiedChannel) bool) {
// 	return func(ctx context.Context, cb func(channeltypes.IdentifiedChannel) bool) {
// 		for _, channel := range s {
// 			stop := cb(channel)
// 			if stop {
// 				break
// 			}
// 		}
// 	}
// }

// type MockCapabilityKeeper struct {
// 	GetCapabilityFn          func(ctx context.Context, name string) (*capabilitytypes.Capability, bool)
// 	ClaimCapabilityFn        func(ctx context.Context, cap *capabilitytypes.Capability, name string) error
// 	AuthenticateCapabilityFn func(ctx context.Context, capability *capabilitytypes.Capability, name string) bool
// }

// func (m MockCapabilityKeeper) GetCapability(ctx context.Context, name string) (*capabilitytypes.Capability, bool) {
// 	if m.GetCapabilityFn == nil {
// 		panic("not supposed to be called!")
// 	}
// 	return m.GetCapabilityFn(ctx, name)
// }

// func (m MockCapabilityKeeper) ClaimCapability(ctx context.Context, cap *capabilitytypes.Capability, name string) error {
// 	if m.ClaimCapabilityFn == nil {
// 		panic("not supposed to be called!")
// 	}
// 	return m.ClaimCapabilityFn(ctx, cap, name)
// }

// func (m MockCapabilityKeeper) AuthenticateCapability(ctx context.Context, capability *capabilitytypes.Capability, name string) bool {
// 	if m.AuthenticateCapabilityFn == nil {
// 		panic("not supposed to be called!")
// 	}
// 	return m.AuthenticateCapabilityFn(ctx, capability, name)
// }

// var _ types.ICS20TransferPortSource = &MockIBCTransferKeeper{}

// type MockIBCTransferKeeper struct {
// 	GetPortFn func(ctx context.Context) string
// }

// func (m MockIBCTransferKeeper) GetPort(ctx context.Context) string {
// 	if m.GetPortFn == nil {
// 		panic("not expected to be called")
// 	}
// 	return m.GetPortFn(ctx)
// }

// var _ types.IBCContractKeeper = &IBCContractKeeperMock{}

// type IBCContractKeeperMock struct {
// 	types.IBCContractKeeper
// 	OnRecvPacketFn func(ctx context.Context, contractAddr sdk.AccAddress, msg wasmvmtypes.IBCPacketReceiveMsg) (ibcexported.Acknowledgement, error)

// 	packets map[string]channeltypes.Packet
// }

// func (m *IBCContractKeeperMock) OnRecvPacket(ctx context.Context, contractAddr sdk.AccAddress, msg wasmvmtypes.IBCPacketReceiveMsg) (ibcexported.Acknowledgement, error) {
// 	if m.OnRecvPacketFn == nil {
// 		panic("not expected to be called")
// 	}
// 	return m.OnRecvPacketFn(ctx, contractAddr, msg)
// }

// func (m *IBCContractKeeperMock) LoadAsyncAckPacket(ctx context.Context, portID, channelID string, sequence uint64) (channeltypes.Packet, error) {
// 	if m.packets == nil {
// 		m.packets = make(map[string]channeltypes.Packet)
// 	}
// 	key := portID + fmt.Sprint(len(channelID)) + channelID
// 	packet, ok := m.packets[key]
// 	if !ok {
// 		return channeltypes.Packet{}, fmt.Errorf("packet not found")
// 	}
// 	return packet, nil
// }

// func (m *IBCContractKeeperMock) StoreAsyncAckPacket(ctx context.Context, packet channeltypes.Packet) error {
// 	if m.packets == nil {
// 		m.packets = make(map[string]channeltypes.Packet)
// 	}
// 	key := packet.DestinationPort + fmt.Sprint(len(packet.DestinationChannel)) + packet.DestinationChannel
// 	m.packets[key] = packet
// 	return nil
// }

// func (m *IBCContractKeeperMock) DeleteAsyncAckPacket(ctx context.Context, portID, channelID string, sequence uint64) {
// 	key := portID + fmt.Sprint(len(channelID)) + channelID
// 	delete(m.packets, key)
// }
