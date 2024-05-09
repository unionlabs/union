package types

import (
	sdk "github.com/cosmos/cosmos-sdk/types"

	transfertypes "github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
)

const (
	TypeMsgWriteDiferredAck = "write_diferred_ack"
)

var _ sdk.Msg = &MsgWriteDiferredAck{}

func NewMsgWriteDiferredAck(packet channeltypes.Packet, data transfertypes.FungibleTokenPacketData, info DiferredPacketInfo, ack channeltypes.Acknowledgement) *MsgWriteDiferredAck {
	return &MsgWriteDiferredAck{
		Packet:             &packet,
		Data:               &data,
		DiferredPacketInfo: &info,
		Ack:                &ack,
	}
}

func (m MsgWriteDiferredAck) Type() string { return TypeMsgWriteDiferredAck }
