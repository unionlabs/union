package types

import (
	sdk "github.com/cosmos/cosmos-sdk/types"

	transfertypes "github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
)

const (
	TypeMsgWriteDifferedAck = "write_differed_ack"
)

var _ sdk.Msg = &MsgWriteDifferedAck{}

func NewMsgWriteDifferedAck(packet channeltypes.Packet, data transfertypes.FungibleTokenPacketData, info DifferedPacketInfo, ack channeltypes.Acknowledgement) *MsgWriteDifferedAck {
	return &MsgWriteDifferedAck{
		Packet:             &packet,
		Data:               &data,
		DifferedPacketInfo: &info,
		Ack:                &ack,
	}
}
