package keeper

import (
	"encoding/json"
	"testing"

	wasmvm "github.com/CosmWasm/wasmvm/v2"
	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"
	"github.com/cosmos/gogoproto/proto"
	capabilitytypes "github.com/cosmos/ibc-go/modules/capability/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types" //nolint:staticcheck
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	ibcexported "github.com/cosmos/ibc-go/v8/modules/core/exported"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/log"
	sdkmath "cosmossdk.io/math"

	"github.com/cosmos/cosmos-sdk/baseapp"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	banktypes "cosmossdk.io/x/bank/types"

	"github.com/CosmWasm/wasmd/x/wasm/keeper/wasmtesting"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

func TestMessageHandlerChainDispatch(t *testing.T) {
	capturingHandler, gotMsgs := wasmtesting.NewCapturingMessageHandler()

	alwaysUnknownMsgHandler := &wasmtesting.MockMessageHandler{
		DispatchMsgFn: func(ctx sdk.Context, contractAddr sdk.AccAddress, contractIBCPortID string, msg wasmvmtypes.CosmosMsg) (events []sdk.Event, data [][]byte, msgResponses [][]*codectypes.Any, err error) {
			return nil, nil, [][]*codectypes.Any{}, types.ErrUnknownMsg
		},
	}

	assertNotCalledHandler := &wasmtesting.MockMessageHandler{
		DispatchMsgFn: func(ctx sdk.Context, contractAddr sdk.AccAddress, contractIBCPortID string, msg wasmvmtypes.CosmosMsg) (events []sdk.Event, data [][]byte, msgResponses [][]*codectypes.Any, err error) {
			t.Fatal("not expected to be called")
			return
		},
	}

	myMsg := wasmvmtypes.CosmosMsg{Custom: []byte(`{}`)}
	specs := map[string]struct {
		handlers  []Messenger
		expErr    *errorsmod.Error
		expEvents []sdk.Event
	}{
		"single handler": {
			handlers: []Messenger{capturingHandler},
		},
		"passed to next handler": {
			handlers: []Messenger{alwaysUnknownMsgHandler, capturingHandler},
		},
		"stops iteration when handled": {
			handlers: []Messenger{capturingHandler, assertNotCalledHandler},
		},
		"stops iteration on handler error": {
			handlers: []Messenger{&wasmtesting.MockMessageHandler{
				DispatchMsgFn: func(ctx sdk.Context, contractAddr sdk.AccAddress, contractIBCPortID string, msg wasmvmtypes.CosmosMsg) (events []sdk.Event, data [][]byte, msgResponses [][]*codectypes.Any, err error) {
					return nil, nil, [][]*codectypes.Any{}, types.ErrInvalidMsg
				},
			}, assertNotCalledHandler},
			expErr: types.ErrInvalidMsg,
		},
		"return events when handle": {
			handlers: []Messenger{
				&wasmtesting.MockMessageHandler{
					DispatchMsgFn: func(ctx sdk.Context, contractAddr sdk.AccAddress, contractIBCPortID string, msg wasmvmtypes.CosmosMsg) (events []sdk.Event, data [][]byte, msgResponses [][]*codectypes.Any, err error) {
						_, data, msgResponses, _ = capturingHandler.DispatchMsg(ctx, contractAddr, contractIBCPortID, msg)
						return []sdk.Event{sdk.NewEvent("myEvent", sdk.NewAttribute("foo", "bar"))}, data, msgResponses, nil
					},
				},
			},
			expEvents: []sdk.Event{sdk.NewEvent("myEvent", sdk.NewAttribute("foo", "bar"))},
		},
		"return error when none can handle": {
			handlers: []Messenger{alwaysUnknownMsgHandler},
			expErr:   types.ErrUnknownMsg,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			*gotMsgs = make([]wasmvmtypes.CosmosMsg, 0)

			// when
			h := MessageHandlerChain{spec.handlers}
			gotEvents, gotData, gotMsgResponses, gotErr := h.DispatchMsg(sdk.Context{}, RandomAccountAddress(t), "anyPort", myMsg)

			// then
			require.True(t, spec.expErr.Is(gotErr), "exp %v but got %#+v", spec.expErr, gotErr)
			if spec.expErr != nil {
				return
			}
			assert.Equal(t, []wasmvmtypes.CosmosMsg{myMsg}, *gotMsgs)
			assert.Equal(t, [][]byte{{1}}, gotData) // {1} is default in capturing handler
			assert.Equal(t, [][]*codectypes.Any{}, gotMsgResponses)
			assert.Equal(t, spec.expEvents, gotEvents)
		})
	}
}

func TestSDKMessageHandlerDispatch(t *testing.T) {
	myEvent := sdk.NewEvent("myEvent", sdk.NewAttribute("foo", "bar"))
	const myData = "myData"
	myRouterResult := sdk.Result{
		Data:   []byte(myData),
		Events: sdk.Events{myEvent}.ToABCIEvents(),
	}

	var gotMsg []sdk.Msg
	capturingMessageRouter := wasmtesting.MessageRouterFunc(func(msg sdk.Msg) baseapp.MsgServiceHandler {
		return func(ctx sdk.Context, req sdk.Msg) (*sdk.Result, error) {
			gotMsg = append(gotMsg, msg)
			return &myRouterResult, nil
		}
	})
	noRouteMessageRouter := wasmtesting.MessageRouterFunc(func(msg sdk.Msg) baseapp.MsgServiceHandler {
		return nil
	})
	myContractAddr := RandomAccountAddress(t)
	myContractMessage := wasmvmtypes.CosmosMsg{Custom: []byte("{}")}

	specs := map[string]struct {
		srcRoute         MessageRouter
		srcEncoder       CustomEncoder
		expErr           *errorsmod.Error
		expMsgDispatched int
	}{
		"all good": {
			srcRoute: capturingMessageRouter,
			srcEncoder: func(sender sdk.AccAddress, msg json.RawMessage) ([]sdk.Msg, error) {
				myMsg := types.MsgExecuteContract{
					Sender:   myContractAddr.String(),
					Contract: RandomBech32AccountAddress(t),
					Msg:      []byte("{}"),
				}
				return []sdk.Msg{&myMsg}, nil
			},
			expMsgDispatched: 1,
		},
		"multiple output msgs": {
			srcRoute: capturingMessageRouter,
			srcEncoder: func(sender sdk.AccAddress, msg json.RawMessage) ([]sdk.Msg, error) {
				first := &types.MsgExecuteContract{
					Sender:   myContractAddr.String(),
					Contract: RandomBech32AccountAddress(t),
					Msg:      []byte("{}"),
				}
				second := &types.MsgExecuteContract{
					Sender:   myContractAddr.String(),
					Contract: RandomBech32AccountAddress(t),
					Msg:      []byte("{}"),
				}
				return []sdk.Msg{first, second}, nil
			},
			expMsgDispatched: 2,
		},
		"invalid sdk message rejected": {
			srcRoute: capturingMessageRouter,
			srcEncoder: func(sender sdk.AccAddress, msg json.RawMessage) ([]sdk.Msg, error) {
				invalidMsg := types.MsgExecuteContract{
					Sender:   myContractAddr.String(),
					Contract: RandomBech32AccountAddress(t),
					Msg:      []byte("INVALID_JSON"),
				}
				return []sdk.Msg{&invalidMsg}, nil
			},
			expErr: types.ErrInvalid,
		},
		"invalid sender rejected": {
			srcRoute: capturingMessageRouter,
			srcEncoder: func(sender sdk.AccAddress, msg json.RawMessage) ([]sdk.Msg, error) {
				invalidMsg := types.MsgExecuteContract{
					Sender:   RandomBech32AccountAddress(t),
					Contract: RandomBech32AccountAddress(t),
					Msg:      []byte("{}"),
				}
				return []sdk.Msg{&invalidMsg}, nil
			},
			expErr: sdkerrors.ErrUnauthorized,
		},
		"unroutable message rejected": {
			srcRoute: noRouteMessageRouter,
			srcEncoder: func(sender sdk.AccAddress, msg json.RawMessage) ([]sdk.Msg, error) {
				myMsg := types.MsgExecuteContract{
					Sender:   myContractAddr.String(),
					Contract: RandomBech32AccountAddress(t),
					Msg:      []byte("{}"),
				}
				return []sdk.Msg{&myMsg}, nil
			},
			expErr: sdkerrors.ErrUnknownRequest,
		},
		"encoding error passed": {
			srcRoute: capturingMessageRouter,
			srcEncoder: func(sender sdk.AccAddress, msg json.RawMessage) ([]sdk.Msg, error) {
				myErr := types.ErrUnpinContractFailed // any error that is not used
				return nil, myErr
			},
			expErr: types.ErrUnpinContractFailed,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			gotMsg = make([]sdk.Msg, 0)

			// when
			ctx := sdk.Context{}
			h := NewSDKMessageHandler(MakeTestCodec(t), spec.srcRoute, MessageEncoders{Custom: spec.srcEncoder})
			gotEvents, gotData, gotMsgResponses, gotErr := h.DispatchMsg(ctx, myContractAddr, "myPort", myContractMessage)

			// then
			require.True(t, spec.expErr.Is(gotErr), "exp %v but got %#+v", spec.expErr, gotErr)
			if spec.expErr != nil {
				require.Len(t, gotMsg, 0)
				return
			}
			assert.Len(t, gotMsg, spec.expMsgDispatched)
			assert.Len(t, gotMsgResponses, spec.expMsgDispatched)
			for i := 0; i < spec.expMsgDispatched; i++ {
				assert.Equal(t, myEvent, gotEvents[i])
				assert.Equal(t, []byte(myData), gotData[i])
				assert.Equal(t, []*codectypes.Any(nil), gotMsgResponses[i])
			}
		})
	}
}

func TestIBCRawPacketHandler(t *testing.T) {
	ibcPort := "contractsIBCPort"
	ctx := sdk.Context{}.WithLogger(log.NewTestLogger(t))

	type CapturedPacket struct {
		sourcePort       string
		sourceChannel    string
		timeoutHeight    clienttypes.Height
		timeoutTimestamp uint64
		data             []byte
	}
	var capturedPacketSent *CapturedPacket
	var capturedAck []byte
	var capturedPacketAck *CapturedPacket

	capturingICS4Mock := &wasmtesting.MockICS4Wrapper{
		SendPacketFn: func(ctx sdk.Context, channelCap *capabilitytypes.Capability, sourcePort, sourceChannel string, timeoutHeight clienttypes.Height, timeoutTimestamp uint64, data []byte) (uint64, error) {
			capturedPacketSent = &CapturedPacket{
				sourcePort:       sourcePort,
				sourceChannel:    sourceChannel,
				timeoutHeight:    timeoutHeight,
				timeoutTimestamp: timeoutTimestamp,
				data:             data,
			}
			return 1, nil
		},
		WriteAcknowledgementFn: func(ctx sdk.Context, chanCap *capabilitytypes.Capability, packet ibcexported.PacketI, acknowledgement ibcexported.Acknowledgement) error {
			capturedPacketAck = &CapturedPacket{
				sourcePort:       packet.GetSourcePort(),
				sourceChannel:    packet.GetSourceChannel(),
				timeoutHeight:    packet.GetTimeoutHeight().(clienttypes.Height),
				timeoutTimestamp: packet.GetTimeoutTimestamp(),
				data:             packet.GetData(),
			}
			capturedAck = acknowledgement.Acknowledgement()
			return nil
		},
	}
	chanKeeper := &wasmtesting.MockChannelKeeper{
		GetChannelFn: func(ctx sdk.Context, srcPort, srcChan string) (channeltypes.Channel, bool) {
			return channeltypes.Channel{
				Counterparty: channeltypes.NewCounterparty(
					"other-port",
					"other-channel-1",
				),
			}, true
		},
	}
	capKeeper := &wasmtesting.MockCapabilityKeeper{
		GetCapabilityFn: func(ctx sdk.Context, name string) (*capabilitytypes.Capability, bool) {
			return &capabilitytypes.Capability{}, true
		},
	}
	contractKeeper := wasmtesting.IBCContractKeeperMock{}
	// also store a packet to be acked
	ackPacket := channeltypes.Packet{
		Sequence:           1,
		SourcePort:         "src-port",
		SourceChannel:      "channel-0",
		DestinationPort:    ibcPort,
		DestinationChannel: "channel-1",
		Data:               []byte{},
		TimeoutHeight:      clienttypes.Height{},
		TimeoutTimestamp:   1720000000000000000,
	}
	contractKeeper.StoreAsyncAckPacket(ctx, ackPacket)

	sendResponse := types.MsgIBCSendResponse{Sequence: 1}
	ackResponse := types.MsgIBCWriteAcknowledgementResponse{}

	specs := map[string]struct {
		srcMsg        wasmvmtypes.IBCMsg
		chanKeeper    types.ChannelKeeper
		capKeeper     types.CapabilityKeeper
		expPacketSent *CapturedPacket
		expPacketAck  *CapturedPacket
		expAck        []byte
		expErr        *errorsmod.Error
		expResp       proto.Message
	}{
		"send packet, all good": {
			srcMsg: wasmvmtypes.IBCMsg{
				SendPacket: &wasmvmtypes.SendPacketMsg{
					ChannelID: "channel-1",
					Data:      []byte("myData"),
					Timeout:   wasmvmtypes.IBCTimeout{Block: &wasmvmtypes.IBCTimeoutBlock{Revision: 1, Height: 2}},
				},
			},
			chanKeeper: chanKeeper,
			capKeeper:  capKeeper,
			expPacketSent: &CapturedPacket{
				sourcePort:    ibcPort,
				sourceChannel: "channel-1",
				timeoutHeight: clienttypes.Height{RevisionNumber: 1, RevisionHeight: 2},
				data:          []byte("myData"),
			},
			expResp: &sendResponse,
		},
		"send packet, capability not found returns error": {
			srcMsg: wasmvmtypes.IBCMsg{
				SendPacket: &wasmvmtypes.SendPacketMsg{
					ChannelID: "channel-1",
					Data:      []byte("myData"),
					Timeout:   wasmvmtypes.IBCTimeout{Block: &wasmvmtypes.IBCTimeoutBlock{Revision: 1, Height: 2}},
				},
			},
			chanKeeper: chanKeeper,
			capKeeper: wasmtesting.MockCapabilityKeeper{
				GetCapabilityFn: func(ctx sdk.Context, name string) (*capabilitytypes.Capability, bool) {
					return nil, false
				},
			},
			expErr: channeltypes.ErrChannelCapabilityNotFound,
		},
		"async ack, all good": {
			srcMsg: wasmvmtypes.IBCMsg{
				WriteAcknowledgement: &wasmvmtypes.WriteAcknowledgementMsg{
					ChannelID:      "channel-1",
					PacketSequence: 1,
					Ack:            wasmvmtypes.IBCAcknowledgement{Data: []byte("myAck")},
				},
			},
			chanKeeper: chanKeeper,
			capKeeper:  capKeeper,
			expPacketAck: &CapturedPacket{
				sourcePort:       ackPacket.SourcePort,
				sourceChannel:    ackPacket.SourceChannel,
				timeoutHeight:    ackPacket.TimeoutHeight,
				timeoutTimestamp: ackPacket.TimeoutTimestamp,
				data:             ackPacket.Data,
			},
			expAck:  []byte("myAck"),
			expResp: &ackResponse,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			capturedPacketSent = nil
			capturedAck = nil
			capturedPacketAck = nil

			// when
			h := NewIBCRawPacketHandler(capturingICS4Mock, &contractKeeper, spec.chanKeeper, spec.capKeeper)
			evts, data, msgResponses, gotErr := h.DispatchMsg(ctx, RandomAccountAddress(t), ibcPort, wasmvmtypes.CosmosMsg{IBC: &spec.srcMsg}) //nolint:gosec

			// then
			require.True(t, spec.expErr.Is(gotErr), "exp %v but got %#+v", spec.expErr, gotErr)
			if spec.expErr != nil {
				return
			}

			assert.Nil(t, evts)
			require.NotNil(t, data)
			assert.Len(t, msgResponses, 1)
			assert.Equal(t, "/"+proto.MessageName(spec.expResp), msgResponses[0][0].TypeUrl)

			// compare expected responses
			expResp, err := proto.Marshal(spec.expResp)
			require.NoError(t, err)
			assert.Equal(t, expResp, msgResponses[0][0].Value)

			assert.Equal(t, spec.expPacketSent, capturedPacketSent)
			assert.Equal(t, spec.expAck, capturedAck)
			assert.Equal(t, spec.expPacketAck, capturedPacketAck)
		})
	}
}

func TestBurnCoinMessageHandlerIntegration(t *testing.T) {
	// testing via full keeper setup so that we are confident the
	// module permissions are set correct and no other handler
	// picks the message in the default handler chain
	ctx, keepers := CreateDefaultTestInput(t)
	// set some supply
	keepers.Faucet.NewFundedRandomAccount(ctx, sdk.NewCoin("denom", sdkmath.NewInt(10_000_000)))
	k := keepers.WasmKeeper

	example := InstantiateHackatomExampleContract(t, ctx, keepers) // with deposit of 100 stake

	before, err := keepers.BankKeeper.TotalSupply(ctx, &banktypes.QueryTotalSupplyRequest{})
	require.NoError(t, err)

	specs := map[string]struct {
		msg    wasmvmtypes.BurnMsg
		expErr bool
	}{
		"all good": {
			msg: wasmvmtypes.BurnMsg{
				Amount: wasmvmtypes.Array[wasmvmtypes.Coin]{{
					Denom:  "denom",
					Amount: "100",
				}},
			},
		},
		"not enough funds in contract": {
			msg: wasmvmtypes.BurnMsg{
				Amount: wasmvmtypes.Array[wasmvmtypes.Coin]{{
					Denom:  "denom",
					Amount: "101",
				}},
			},
			expErr: true,
		},
		"zero amount rejected": {
			msg: wasmvmtypes.BurnMsg{
				Amount: wasmvmtypes.Array[wasmvmtypes.Coin]{{
					Denom:  "denom",
					Amount: "0",
				}},
			},
			expErr: true,
		},
		"unknown denom - insufficient funds": {
			msg: wasmvmtypes.BurnMsg{
				Amount: wasmvmtypes.Array[wasmvmtypes.Coin]{{
					Denom:  "unknown",
					Amount: "1",
				}},
			},
			expErr: true,
		},
	}
	parentCtx := ctx
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			ctx, _ = parentCtx.CacheContext()
			k.wasmVM = &wasmtesting.MockWasmEngine{ExecuteFn: func(codeID wasmvm.Checksum, env wasmvmtypes.Env, info wasmvmtypes.MessageInfo, executeMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
				return &wasmvmtypes.ContractResult{
					Ok: &wasmvmtypes.Response{
						Messages: []wasmvmtypes.SubMsg{
							{Msg: wasmvmtypes.CosmosMsg{Bank: &wasmvmtypes.BankMsg{Burn: &spec.msg}}, ReplyOn: wasmvmtypes.ReplyNever}, //nolint:gosec
						},
					},
				}, 0, nil
			}}

			// when
			_, err = k.execute(ctx, example.Contract, example.CreatorAddr, nil, nil)

			// then
			if spec.expErr {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)

			// and total supply reduced by burned amount
			after, err := keepers.BankKeeper.TotalSupply(ctx, &banktypes.QueryTotalSupplyRequest{})
			require.NoError(t, err)
			diff := before.Supply.Sub(after.Supply...)
			assert.Equal(t, sdk.NewCoins(sdk.NewCoin("denom", sdkmath.NewInt(100))), diff)
		})
	}

	// test cases:
	// not enough money to burn
}
