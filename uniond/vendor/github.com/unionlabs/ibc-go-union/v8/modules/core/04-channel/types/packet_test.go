package types_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
)

func TestCommitPacket(t *testing.T) {
	packet := types.NewPacket(validPacketData, 1, portid, chanid, cpportid, cpchanid, timeoutHeight, timeoutTimestamp)

	registry := codectypes.NewInterfaceRegistry()
	clienttypes.RegisterInterfaces(registry)
	types.RegisterInterfaces(registry)

	cdc := codec.NewProtoCodec(registry)

	commitment := types.CommitPacket(cdc, &packet)
	require.NotNil(t, commitment)
}

func TestPacketValidateBasic(t *testing.T) {
	testCases := []struct {
		packet  types.Packet
		expPass bool
		errMsg  string
	}{
		{types.NewPacket(validPacketData, 1, portid, chanid, cpportid, cpchanid, timeoutHeight, timeoutTimestamp), true, ""},
		{types.NewPacket(validPacketData, 0, portid, chanid, cpportid, cpchanid, timeoutHeight, timeoutTimestamp), false, "invalid sequence"},
		{types.NewPacket(validPacketData, 1, invalidPort, chanid, cpportid, cpchanid, timeoutHeight, timeoutTimestamp), false, "invalid source port"},
		{types.NewPacket(validPacketData, 1, portid, invalidChannel, cpportid, cpchanid, timeoutHeight, timeoutTimestamp), false, "invalid source channel"},
		{types.NewPacket(validPacketData, 1, portid, chanid, invalidPort, cpchanid, timeoutHeight, timeoutTimestamp), false, "invalid destination port"},
		{types.NewPacket(validPacketData, 1, portid, chanid, cpportid, invalidChannel, timeoutHeight, timeoutTimestamp), false, "invalid destination channel"},
		{types.NewPacket(validPacketData, 1, portid, chanid, cpportid, cpchanid, disabledTimeout, 0), false, "disabled both timeout height and timestamp"},
		{types.NewPacket(validPacketData, 1, portid, chanid, cpportid, cpchanid, disabledTimeout, timeoutTimestamp), true, "disabled timeout height, valid timeout timestamp"},
		{types.NewPacket(validPacketData, 1, portid, chanid, cpportid, cpchanid, timeoutHeight, 0), true, "disabled timeout timestamp, valid timeout height"},
		{types.NewPacket(unknownPacketData, 1, portid, chanid, cpportid, cpchanid, timeoutHeight, timeoutTimestamp), true, ""},
	}

	for i, tc := range testCases {
		tc := tc

		err := tc.packet.ValidateBasic()
		if tc.expPass {
			require.NoError(t, err, "Msg %d failed: %s", i, tc.errMsg)
		} else {
			require.Error(t, err, "Invalid Msg %d passed: %s", i, tc.errMsg)
		}
	}
}
