package types_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cometbft/cometbft/crypto/secp256k1"

	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

func TestMsgRegisterPayeeValidation(t *testing.T) {
	var msg *types.MsgRegisterPayee

	testCases := []struct {
		name     string
		malleate func()
		expPass  bool
	}{
		{
			"success",
			func() {},
			true,
		},
		{
			"success: relayer and payee are equal",
			func() {
				msg.Relayer = defaultAccAddress
				msg.Payee = defaultAccAddress
			},
			true,
		},
		{
			"invalid portID",
			func() {
				msg.PortId = ""
			},
			false,
		},
		{
			"invalid channelID",
			func() {
				msg.ChannelId = ""
			},
			false,
		},
		{
			"invalid relayer address",
			func() {
				msg.Relayer = invalidAddress
			},
			false,
		},
		{
			"invalid payee address",
			func() {
				msg.Payee = invalidAddress
			},
			false,
		},
	}

	for i, tc := range testCases {
		tc := tc

		relayerAddr := sdk.AccAddress(secp256k1.GenPrivKey().PubKey().Address())
		payeeAddr := sdk.AccAddress(secp256k1.GenPrivKey().PubKey().Address())

		msg = types.NewMsgRegisterPayee(ibctesting.MockPort, ibctesting.FirstChannelID, relayerAddr.String(), payeeAddr.String())

		tc.malleate()

		err := msg.ValidateBasic()

		if tc.expPass {
			require.NoError(t, err, "valid test case %d failed: %s", i, tc.name)
		} else {
			require.Error(t, err, "invalid test case %d passed: %s", i, tc.name)
		}
	}
}

func TestRegisterPayeeGetSigners(t *testing.T) {
	accAddress := sdk.AccAddress(secp256k1.GenPrivKey().PubKey().Address())
	msg := types.NewMsgRegisterPayee(ibctesting.MockPort, ibctesting.FirstChannelID, accAddress.String(), defaultAccAddress)
	require.Equal(t, []sdk.AccAddress{accAddress}, msg.GetSigners())
}

func TestMsgRegisterCountepartyPayeeValidation(t *testing.T) {
	var msg *types.MsgRegisterCounterpartyPayee

	testCases := []struct {
		name     string
		malleate func()
		expPass  bool
	}{
		{
			"success",
			func() {},
			true,
		},
		{
			"invalid portID",
			func() {
				msg.PortId = ""
			},
			false,
		},
		{
			"invalid channelID",
			func() {
				msg.ChannelId = ""
			},
			false,
		},
		{
			"validate with incorrect destination relayer address",
			func() {
				msg.Relayer = invalidAddress
			},
			false,
		},
		{
			"invalid counterparty payee address",
			func() {
				msg.CounterpartyPayee = ""
			},
			false,
		},
		{
			"invalid counterparty payee address: whitespaced empty string",
			func() {
				msg.CounterpartyPayee = "  "
			},
			false,
		},
		{
			"invalid counterparty payee address: too long",
			func() {
				msg.CounterpartyPayee = ibctesting.GenerateString(types.MaximumCounterpartyPayeeLength + 1)
			},
			false,
		},
	}

	for i, tc := range testCases {
		i, tc := i, tc

		msg = types.NewMsgRegisterCounterpartyPayee(ibctesting.MockPort, ibctesting.FirstChannelID, defaultAccAddress, defaultAccAddress)

		tc.malleate()

		err := msg.ValidateBasic()

		if tc.expPass {
			require.NoError(t, err, "valid test case %d failed: %s", i, tc.name)
		} else {
			require.Error(t, err, "invalid test case %d passed: %s", i, tc.name)
		}
	}
}

func TestRegisterCountepartyAddressGetSigners(t *testing.T) {
	accAddress := sdk.AccAddress(secp256k1.GenPrivKey().PubKey().Address())
	msg := types.NewMsgRegisterCounterpartyPayee(ibctesting.MockPort, ibctesting.FirstChannelID, accAddress.String(), defaultAccAddress)
	require.Equal(t, []sdk.AccAddress{accAddress}, msg.GetSigners())
}

func TestMsgPayPacketFeeValidation(t *testing.T) {
	var msg *types.MsgPayPacketFee

	testCases := []struct {
		name     string
		malleate func()
		expPass  bool
	}{
		{
			"success",
			func() {},
			true,
		},
		{
			"success with empty relayers",
			func() {
				msg.Relayers = []string{}
			},
			true,
		},
		{
			"invalid channelID",
			func() {
				msg.SourceChannelId = ""
			},
			false,
		},
		{
			"invalid portID",
			func() {
				msg.SourcePortId = ""
			},
			false,
		},
		{
			"relayers is not nil",
			func() {
				msg.Relayers = []string{defaultAccAddress}
			},
			false,
		},
		{
			"invalid signer address",
			func() {
				msg.Signer = invalidAddress
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		fee := types.NewFee(defaultRecvFee, defaultAckFee, defaultTimeoutFee)
		msg = types.NewMsgPayPacketFee(fee, ibctesting.MockFeePort, ibctesting.FirstChannelID, defaultAccAddress, nil)

		tc.malleate() // malleate mutates test data

		err := msg.ValidateBasic()

		if tc.expPass {
			require.NoError(t, err, tc.name)
		} else {
			require.Error(t, err, tc.name)
		}
	}
}

func TestPayPacketFeeGetSigners(t *testing.T) {
	refundAddr := sdk.AccAddress(secp256k1.GenPrivKey().PubKey().Address())
	fee := types.NewFee(defaultRecvFee, defaultAckFee, defaultTimeoutFee)
	msg := types.NewMsgPayPacketFee(fee, ibctesting.MockFeePort, ibctesting.FirstChannelID, refundAddr.String(), nil)

	require.Equal(t, []sdk.AccAddress{refundAddr}, msg.GetSigners())
}

func TestMsgPayPacketFeeAsyncValidation(t *testing.T) {
	var msg *types.MsgPayPacketFeeAsync

	testCases := []struct {
		name     string
		malleate func()
		expPass  bool
	}{
		{
			"success",
			func() {},
			true,
		},
		{
			"success with empty relayers",
			func() {
				msg.PacketFee.Relayers = []string{}
			},
			true,
		},
		{
			"invalid channelID",
			func() {
				msg.PacketId.ChannelId = ""
			},
			false,
		},
		{
			"invalid portID",
			func() {
				msg.PacketId.PortId = ""
			},
			false,
		},
		{
			"invalid sequence",
			func() {
				msg.PacketId.Sequence = 0
			},
			false,
		},
		{
			"relayers is not nil",
			func() {
				msg.PacketFee.Relayers = []string{defaultAccAddress}
			},
			false,
		},
		{
			"invalid signer address",
			func() {
				msg.PacketFee.RefundAddress = "invalid-addr"
			},
			false,
		},
		{
			"should fail when all fees are invalid",
			func() {
				msg.PacketFee.Fee.AckFee = invalidFee
				msg.PacketFee.Fee.RecvFee = invalidFee
				msg.PacketFee.Fee.TimeoutFee = invalidFee
			},
			false,
		},
		{
			"should fail with single invalid fee",
			func() {
				msg.PacketFee.Fee.AckFee = invalidFee
			},
			false,
		},
		{
			"should fail with two invalid fees",
			func() {
				msg.PacketFee.Fee.AckFee = invalidFee
				msg.PacketFee.Fee.TimeoutFee = invalidFee
			},
			false,
		},
		{
			"should pass with two empty fees",
			func() {
				msg.PacketFee.Fee.AckFee = sdk.Coins{}
				msg.PacketFee.Fee.TimeoutFee = sdk.Coins{}
			},
			true,
		},
		{
			"should pass with one empty fee",
			func() {
				msg.PacketFee.Fee.TimeoutFee = sdk.Coins{}
			},
			true,
		},
		{
			"should fail if all fees are empty",
			func() {
				msg.PacketFee.Fee.AckFee = sdk.Coins{}
				msg.PacketFee.Fee.RecvFee = sdk.Coins{}
				msg.PacketFee.Fee.TimeoutFee = sdk.Coins{}
			},
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		packetID := channeltypes.NewPacketID(ibctesting.MockFeePort, ibctesting.FirstChannelID, 1)
		fee := types.NewFee(defaultRecvFee, defaultAckFee, defaultTimeoutFee)
		packetFee := types.NewPacketFee(fee, defaultAccAddress, nil)

		msg = types.NewMsgPayPacketFeeAsync(packetID, packetFee)

		tc.malleate() // malleate mutates test data

		err := msg.ValidateBasic()

		if tc.expPass {
			require.NoError(t, err, tc.name)
		} else {
			require.Error(t, err, tc.name)
		}
	}
}

func TestPayPacketFeeAsyncGetSigners(t *testing.T) {
	refundAddr := sdk.AccAddress(secp256k1.GenPrivKey().PubKey().Address())
	packetID := channeltypes.NewPacketID(ibctesting.MockFeePort, ibctesting.FirstChannelID, 1)
	fee := types.NewFee(defaultRecvFee, defaultAckFee, defaultTimeoutFee)
	packetFee := types.NewPacketFee(fee, refundAddr.String(), nil)

	msg := types.NewMsgPayPacketFeeAsync(packetID, packetFee)

	require.Equal(t, []sdk.AccAddress{refundAddr}, msg.GetSigners())
}
