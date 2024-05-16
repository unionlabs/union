package types

import (
	sdk "github.com/cosmos/cosmos-sdk/types"

	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
)

const (
	TypeMsgWriteDiferredAck = "write_diferred_ack"
)

var _ sdk.Msg = &MsgWriteDiferredAck{}

func NewMsgWriteDiferredAck(info DiferredPacketInfo, ack channeltypes.Acknowledgement) *MsgWriteDiferredAck {
	return &MsgWriteDiferredAck{
		DiferredPacketInfo: &info,
		Ack:                &ack,
	}
}

func (m MsgWriteDiferredAck) Type() string { return TypeMsgWriteDiferredAck }
