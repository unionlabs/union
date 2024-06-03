package types

import (
	sdk "github.com/cosmos/cosmos-sdk/types"

	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
)

const (
	TypeMsgWriteDeferredAck = "write_deferred_ack"
)

var _ sdk.Msg = &MsgWriteDeferredAck{}

func NewMsgWriteDeferredAck(info DeferredPacketInfo, ack channeltypes.Acknowledgement) *MsgWriteDeferredAck {
	return &MsgWriteDeferredAck{
		DeferredPacketInfo: &info,
		Ack:                &ack,
	}
}

func (m MsgWriteDeferredAck) Type() string { return TypeMsgWriteDeferredAck }
