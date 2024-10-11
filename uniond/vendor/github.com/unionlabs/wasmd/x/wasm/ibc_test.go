package wasm

import (
	"testing"

	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"
	abci "github.com/cometbft/cometbft/abci/types"
	"cosmossdk.io/math/unsafe"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types" //nolint:staticcheck
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	ibcexported "github.com/cosmos/ibc-go/v8/modules/core/exported"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"

	"github.com/CosmWasm/wasmd/x/wasm/keeper"
	"github.com/CosmWasm/wasmd/x/wasm/keeper/wasmtesting"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

func TestOnRecvPacket(t *testing.T) {
	anyRelayerAddr := sdk.AccAddress(rand.Bytes(address.Len))
	anyContractIBCPkg := IBCPacketFixture(func(p *channeltypes.Packet) {
		p.DestinationPort = "wasm.cosmos1w09vr7rpe2agu0kg2zlpkdckce865l3zps8mxjurxthfh3m7035qe5hh7f"
	})
	myCustomEvent := sdk.NewEvent("testing")
	specs := map[string]struct {
		ibcPkg               channeltypes.Packet
		contractRsp          ibcexported.Acknowledgement
		contractOkMsgExecErr error
		expEvents            sdk.Events
		expPanic             bool
		expAck               ibcexported.Acknowledgement
	}{
		"contract returns success response": {
			ibcPkg:      anyContractIBCPkg,
			contractRsp: keeper.ContractConfirmStateAck([]byte{1}),
			expAck:      keeper.ContractConfirmStateAck([]byte{1}),
			expEvents: sdk.Events{
				myCustomEvent,
				{
					Type: "ibc_packet_received",
					Attributes: []abci.EventAttribute{
						{Key: "module", Value: "wasm"},
						{Key: "_contract_address", Value: "cosmos1w09vr7rpe2agu0kg2zlpkdckce865l3zps8mxjurxthfh3m7035qe5hh7f"},
						{Key: "success", Value: "true"},
					},
				},
			},
		},
		"contract returns err response": {
			ibcPkg:      anyContractIBCPkg,
			contractRsp: CreateErrorAcknowledgement(types.ErrInvalid.Wrap("testing")),
			expAck:      CreateErrorAcknowledgement(types.ErrInvalid.Wrap("testing")),
			expEvents: sdk.Events{
				{
					Type: "ibc_packet_received",
					Attributes: []abci.EventAttribute{
						{Key: "module", Value: "wasm"},
						{Key: "_contract_address", Value: "cosmos1w09vr7rpe2agu0kg2zlpkdckce865l3zps8mxjurxthfh3m7035qe5hh7f"},
						{Key: "success", Value: "false"},
					},
				},
			},
		},
		"nil considered success response": { // regression only
			ibcPkg: anyContractIBCPkg,
			expEvents: sdk.Events{
				myCustomEvent,
				{
					Type: "ibc_packet_received",
					Attributes: []abci.EventAttribute{
						{Key: "module", Value: "wasm"},
						{Key: "_contract_address", Value: "cosmos1w09vr7rpe2agu0kg2zlpkdckce865l3zps8mxjurxthfh3m7035qe5hh7f"},
						{Key: "success", Value: "true"},
					},
				},
			},
		},
		"unknown contract port": {
			ibcPkg: IBCPacketFixture(func(p *channeltypes.Packet) {
				p.DestinationPort = "not-a-contract-port"
			}),
			expPanic: true,
		},
		"returned messages executed with error": {
			ibcPkg:               anyContractIBCPkg,
			contractOkMsgExecErr: types.ErrInvalid.Wrap("testing"),
			expAck:               CreateErrorAcknowledgement(types.ErrInvalid.Wrap("testing")),
			expEvents: sdk.Events{{
				Type: "ibc_packet_received",
				Attributes: []abci.EventAttribute{
					{Key: "module", Value: "wasm"},
					{Key: "_contract_address", Value: "cosmos1w09vr7rpe2agu0kg2zlpkdckce865l3zps8mxjurxthfh3m7035qe5hh7f"},
					{Key: "success", Value: "false"},
					{Key: "error", Value: "testing: invalid"}, // not redacted
				},
			}},
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			mock := wasmtesting.IBCContractKeeperMock{
				OnRecvPacketFn: func(ctx sdk.Context, contractAddr sdk.AccAddress, msg wasmvmtypes.IBCPacketReceiveMsg) (ibcexported.Acknowledgement, error) {
					// additional custom event to confirm event handling on state commit/ rollback
					ctx.EventManager().EmitEvent(myCustomEvent)
					return spec.contractRsp, spec.contractOkMsgExecErr
				},
			}
			h := NewIBCHandler(&mock, nil, nil)
			em := &sdk.EventManager{}
			ctx := sdk.Context{}.WithEventManager(em)
			if spec.expPanic {
				require.Panics(t, func() {
					_ = h.OnRecvPacket(ctx, spec.ibcPkg, anyRelayerAddr)
				})
				return
			}
			gotAck := h.OnRecvPacket(ctx, spec.ibcPkg, anyRelayerAddr)
			assert.Equal(t, spec.expAck, gotAck)
			assert.Equal(t, spec.expEvents, em.Events())
		})
	}
}

func TestMapToWasmVMIBCPacket(t *testing.T) {
	var myTimestamp uint64 = 1
	specs := map[string]struct {
		src channeltypes.Packet
		exp wasmvmtypes.IBCPacket
	}{
		"with height timeout": {
			src: IBCPacketFixture(),
			exp: wasmvmtypes.IBCPacket{
				Data:     []byte("myData"),
				Src:      wasmvmtypes.IBCEndpoint{PortID: "srcPort", ChannelID: "channel-1"},
				Dest:     wasmvmtypes.IBCEndpoint{PortID: "destPort", ChannelID: "channel-2"},
				Sequence: 1,
				Timeout:  wasmvmtypes.IBCTimeout{Block: &wasmvmtypes.IBCTimeoutBlock{Height: 1, Revision: 2}},
			},
		},
		"with time timeout": {
			src: IBCPacketFixture(func(p *channeltypes.Packet) {
				p.TimeoutTimestamp = myTimestamp
				p.TimeoutHeight = clienttypes.Height{}
			}),
			exp: wasmvmtypes.IBCPacket{
				Data:     []byte("myData"),
				Src:      wasmvmtypes.IBCEndpoint{PortID: "srcPort", ChannelID: "channel-1"},
				Dest:     wasmvmtypes.IBCEndpoint{PortID: "destPort", ChannelID: "channel-2"},
				Sequence: 1,
				Timeout:  wasmvmtypes.IBCTimeout{Timestamp: myTimestamp},
			},
		}, "with time and height timeout": {
			src: IBCPacketFixture(func(p *channeltypes.Packet) {
				p.TimeoutTimestamp = myTimestamp
			}),
			exp: wasmvmtypes.IBCPacket{
				Data:     []byte("myData"),
				Src:      wasmvmtypes.IBCEndpoint{PortID: "srcPort", ChannelID: "channel-1"},
				Dest:     wasmvmtypes.IBCEndpoint{PortID: "destPort", ChannelID: "channel-2"},
				Sequence: 1,
				Timeout: wasmvmtypes.IBCTimeout{
					Block:     &wasmvmtypes.IBCTimeoutBlock{Height: 1, Revision: 2},
					Timestamp: myTimestamp,
				},
			},
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			got := newIBCPacket(spec.src)
			assert.Equal(t, spec.exp, got)
		})
	}
}

func IBCPacketFixture(mutators ...func(p *channeltypes.Packet)) channeltypes.Packet {
	r := channeltypes.Packet{
		Sequence:           1,
		SourcePort:         "srcPort",
		SourceChannel:      "channel-1",
		DestinationPort:    "destPort",
		DestinationChannel: "channel-2",
		Data:               []byte("myData"),
		TimeoutHeight: clienttypes.Height{
			RevisionHeight: 1,
			RevisionNumber: 2,
		},
		TimeoutTimestamp: 0,
	}
	for _, m := range mutators {
		m(&r)
	}
	return r
}
