package cli

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/core/registry"
	banktypes "cosmossdk.io/x/bank/types"
	stakingtypes "cosmossdk.io/x/staking/types"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"

	icatypes "github.com/cosmos/ibc-go/v8/modules/apps/27-interchain-accounts/types"
)

const msgDelegateMessage = `{
	"@type": "/cosmos.staking.v1beta1.MsgDelegate",
	"delegator_address": "cosmos15ccshhmp0gsx29qpqq6g4zmltnnvgmyu9ueuadh9y2nc5zj0szls5gtddz",
	"validator_address": "cosmosvaloper1qnk2n4nlkpw9xfqntladh74w6ujtulwnmxnh3k",
	"amount": {
		"denom": "stake",
		"amount": "1000"
	}
}`

const bankSendMessage = `{
	"@type":"/cosmos.bank.v1beta1.MsgSend",
	"from_address":"cosmos15ccshhmp0gsx29qpqq6g4zmltnnvgmyu9ueuadh9y2nc5zj0szls5gtddz",
	"to_address":"cosmos10h9stc5v6ntgeygf5xf945njqq5h32r53uquvw",
	"amount": [
		{
			"denom": "stake",
			"amount": "1000"
		}
	]
}`

var multiMsg = fmt.Sprintf("[ %s, %s ]", msgDelegateMessage, bankSendMessage)

func TestGeneratePacketData(t *testing.T) {
	t.Helper()
	tests := []struct {
		name                string
		memo                string
		expectedPass        bool
		message             string
		registerInterfaceFn func(registry registry.InterfaceRegistrar)
		assertionFn         func(t *testing.T, msgs []sdk.Msg)
	}{
		{
			name:         "packet data generation succeeds (MsgDelegate & MsgSend)",
			memo:         "",
			expectedPass: true,
			message:      multiMsg,
			registerInterfaceFn: func(registry registry.InterfaceRegistrar) {
				stakingtypes.RegisterInterfaces(registry)
				banktypes.RegisterInterfaces(registry)
			},
			assertionFn: func(t *testing.T, msgs []sdk.Msg) {
				t.Helper()
				assertMsgDelegate(t, msgs[0])
				assertMsgBankSend(t, msgs[1])
			},
		},
		{
			name:                "packet data generation succeeds (MsgDelegate)",
			memo:                "non-empty-memo",
			expectedPass:        true,
			message:             msgDelegateMessage,
			registerInterfaceFn: stakingtypes.RegisterInterfaces,
			assertionFn: func(t *testing.T, msgs []sdk.Msg) {
				t.Helper()
				assertMsgDelegate(t, msgs[0])
			},
		},
		{
			name:                "packet data generation succeeds (MsgSend)",
			memo:                "non-empty-memo",
			expectedPass:        true,
			message:             bankSendMessage,
			registerInterfaceFn: banktypes.RegisterInterfaces,
			assertionFn: func(t *testing.T, msgs []sdk.Msg) {
				t.Helper()
				assertMsgBankSend(t, msgs[0])
			},
		},
		{
			name:                "empty memo is valid",
			memo:                "",
			expectedPass:        true,
			message:             msgDelegateMessage,
			registerInterfaceFn: stakingtypes.RegisterInterfaces,
			assertionFn:         nil,
		},
		{
			name:         "invalid message string",
			expectedPass: false,
			message:      "<invalid-message-body>",
		},
	}

	encodings := []string{icatypes.EncodingProtobuf, icatypes.EncodingProto3JSON}
	for _, encoding := range encodings {
		for _, tc := range tests {
			tc := tc
			ir := codectypes.NewInterfaceRegistry()
			if tc.registerInterfaceFn != nil {
				tc.registerInterfaceFn(ir)
			}

			cdc := codec.NewProtoCodec(ir)

			t.Run(fmt.Sprintf("%s with %s encoding", tc.name, encoding), func(t *testing.T) {
				bz, err := generatePacketData(cdc, []byte(tc.message), tc.memo, encoding)

				if tc.expectedPass {
					require.NoError(t, err)
					require.NotNil(t, bz)

					packetData := icatypes.InterchainAccountPacketData{}
					err = cdc.UnmarshalJSON(bz, &packetData)
					require.NoError(t, err)

					require.Equal(t, icatypes.EXECUTE_TX, packetData.Type)
					require.Equal(t, tc.memo, packetData.Memo)

					data := packetData.Data
					messages, err := icatypes.DeserializeCosmosTx(cdc, data, encoding)

					require.NoError(t, err)
					require.NotNil(t, messages)

					if tc.assertionFn != nil {
						tc.assertionFn(t, messages)
					}
				} else {
					require.Error(t, err)
					require.Nil(t, bz)
				}
			})
		}
	}
}

func assertMsgBankSend(t *testing.T, msg sdk.Msg) { //nolint:thelper
	bankSendMsg, ok := msg.(*banktypes.MsgSend)
	require.True(t, ok)
	require.Equal(t, "cosmos15ccshhmp0gsx29qpqq6g4zmltnnvgmyu9ueuadh9y2nc5zj0szls5gtddz", bankSendMsg.FromAddress)
	require.Equal(t, "cosmos10h9stc5v6ntgeygf5xf945njqq5h32r53uquvw", bankSendMsg.ToAddress)
	require.Equal(t, "stake", bankSendMsg.Amount.GetDenomByIndex(0))
	require.Equal(t, uint64(1000), bankSendMsg.Amount[0].Amount.Uint64())
}

func assertMsgDelegate(t *testing.T, msg sdk.Msg) { //nolint:thelper
	msgDelegate, ok := msg.(*stakingtypes.MsgDelegate)
	require.True(t, ok)
	require.Equal(t, "cosmos15ccshhmp0gsx29qpqq6g4zmltnnvgmyu9ueuadh9y2nc5zj0szls5gtddz", msgDelegate.DelegatorAddress)
	require.Equal(t, "cosmosvaloper1qnk2n4nlkpw9xfqntladh74w6ujtulwnmxnh3k", msgDelegate.ValidatorAddress)
	require.Equal(t, "stake", msgDelegate.Amount.Denom)
	require.Equal(t, uint64(1000), msgDelegate.Amount.Amount.Uint64())
}
