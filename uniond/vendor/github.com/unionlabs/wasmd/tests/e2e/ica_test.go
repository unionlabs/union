package e2e

import (
	"testing"
	"time"

	abci "github.com/cometbft/cometbft/abci/types"
	"cosmossdk.io/math/unsafe"
	"github.com/cosmos/gogoproto/proto"
	icacontrollertypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/controller/types"
	hosttypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/host/types"
	icatypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	sdkmath "cosmossdk.io/math"

	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	banktypes "cosmossdk.io/x/bank/types"

	"github.com/CosmWasm/wasmd/app"
	wasmibctesting "github.com/CosmWasm/wasmd/tests/ibctesting"
)

func TestICA(t *testing.T) {
	// scenario:
	// given a host and controller chain
	// when an ica is registered on the controller chain
	// and the channel is established to the host chain
	// then the ICA owner can submit a message via IBC
	//      to control their account on the host chain
	coord := wasmibctesting.NewCoordinator(t, 2)
	hostChain := coord.GetChain(ibctesting.GetChainID(1))
	hostParams := hosttypes.NewParams(true, []string{sdk.MsgTypeURL(&banktypes.MsgSend{})})
	hostApp := hostChain.App.(*app.WasmApp)
	hostApp.ICAHostKeeper.SetParams(hostChain.GetContext(), hostParams)

	controllerChain := coord.GetChain(ibctesting.GetChainID(2))

	path := wasmibctesting.NewPath(controllerChain, hostChain)
	coord.SetupConnections(path)

	specs := map[string]struct {
		icaVersion string
		encoding   string
	}{
		"proto": {
			icaVersion: "", // empty string defaults to the proto3 encoding type
			encoding:   icatypes.EncodingProtobuf,
		},
		"json": {
			icaVersion: string(icatypes.ModuleCdc.MustMarshalJSON(&icatypes.Metadata{
				Version:                icatypes.Version,
				ControllerConnectionId: path.EndpointA.ConnectionID,
				HostConnectionId:       path.EndpointB.ConnectionID,
				Encoding:               icatypes.EncodingProto3JSON, // use proto3json
				TxType:                 icatypes.TxTypeSDKMultiMsg,
			})),
			encoding: icatypes.EncodingProto3JSON,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			icaControllerKey := secp256k1.GenPrivKey()
			icaControllerAddr := sdk.AccAddress(icaControllerKey.PubKey().Address().Bytes())
			controllerChain.Fund(icaControllerAddr, sdkmath.NewInt(1_000))

			msg := icacontrollertypes.NewMsgRegisterInterchainAccount(path.EndpointA.ConnectionID, icaControllerAddr.String(), spec.icaVersion)
			res, err := controllerChain.SendNonDefaultSenderMsgs(icaControllerKey, msg)
			require.NoError(t, err)
			chanID, portID, version := parseIBCChannelEvents(t, res)

			// next open channels on both sides
			path.EndpointA.ChannelID = chanID
			path.EndpointA.ChannelConfig = &ibctesting.ChannelConfig{
				PortID:  portID,
				Version: version,
				Order:   channeltypes.ORDERED,
			}
			path.EndpointB.ChannelID = ""
			path.EndpointB.ChannelConfig = &ibctesting.ChannelConfig{
				PortID:  icatypes.HostPortID,
				Version: icatypes.Version,
				Order:   channeltypes.ORDERED,
			}
			coord.CreateChannels(path)

			// assert ICA exists on controller
			contApp := controllerChain.App.(*app.WasmApp)
			icaRsp, err := contApp.ICAControllerKeeper.InterchainAccount(controllerChain.GetContext(), &icacontrollertypes.QueryInterchainAccountRequest{
				Owner:        icaControllerAddr.String(),
				ConnectionId: path.EndpointA.ConnectionID,
			})
			require.NoError(t, err)
			icaAddr := sdk.MustAccAddressFromBech32(icaRsp.GetAddress())
			hostChain.Fund(icaAddr, sdkmath.NewInt(1_000))

			// submit a tx
			targetAddr := sdk.AccAddress(rand.Bytes(address.Len))
			sendCoin := sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(100))
			payloadMsg := banktypes.NewMsgSend(icaAddr, targetAddr, sdk.NewCoins(sendCoin))
			rawPayloadData, err := icatypes.SerializeCosmosTx(controllerChain.Codec, []proto.Message{payloadMsg}, spec.encoding)
			require.NoError(t, err)
			payloadPacket := icatypes.InterchainAccountPacketData{
				Type: icatypes.EXECUTE_TX,
				Data: rawPayloadData,
				Memo: "testing",
			}
			relativeTimeout := uint64(time.Minute.Nanoseconds()) // note this is in nanoseconds
			msgSendTx := icacontrollertypes.NewMsgSendTx(icaControllerAddr.String(), path.EndpointA.ConnectionID, relativeTimeout, payloadPacket)
			_, err = controllerChain.SendNonDefaultSenderMsgs(icaControllerKey, msgSendTx)
			require.NoError(t, err)

			assert.Equal(t, 1, len(controllerChain.PendingSendPackets))
			require.NoError(t, coord.RelayAndAckPendingPackets(path))

			gotBalance := hostChain.Balance(targetAddr, sdk.DefaultBondDenom)
			assert.Equal(t, sendCoin.String(), gotBalance.String())
		})
	}
}

func parseIBCChannelEvents(t *testing.T, res *abci.ExecTxResult) (string, string, string) {
	t.Helper()
	chanID, err := wasmibctesting.ParseChannelIDFromEvents(res.GetEvents())
	require.NoError(t, err)
	portID, err := wasmibctesting.ParsePortIDFromEvents(res.GetEvents())
	require.NoError(t, err)
	version, err := wasmibctesting.ParseChannelVersionFromEvents(res.GetEvents())
	require.NoError(t, err)
	return chanID, portID, version
}
