package types

import (
	"fmt"

	errorsmod "cosmossdk.io/errors"
	sdk "github.com/cosmos/cosmos-sdk/types"

	ibcexported "github.com/cosmos/ibc-go/v8/modules/core/exported"
)

const (
	TypeMsgWriteDeferredAck = "write_deferred_ack"
)

var ErrInvalidAcknowledgement = fmt.Errorf("invalid acknowledgement")

var _ sdk.Msg = &MsgWriteDeferredAck{}

var _ ibcexported.Acknowledgement = Acknowledgement{}

type Acknowledgement []byte

func (w Acknowledgement) Success() bool {
	return true // always commit state
}

func (w Acknowledgement) Acknowledgement() []byte {
	return w
}

func NewMsgWriteDeferredAck(info DeferredPacketInfo, ack Acknowledgement) *MsgWriteDeferredAck {
	return &MsgWriteDeferredAck{
		DeferredPacketInfo: &info,
		Ack:                ack.Acknowledgement(),
	}
}

func (m MsgWriteDeferredAck) Type() string { return TypeMsgWriteDeferredAck }

// ValidateBasic performs a basic validation of the acknowledgement
func (ack Acknowledgement) ValidateBasic() error {
	if len(ack) == 0 {
		return errorsmod.Wrap(ErrInvalidAcknowledgement, "acknowledgement result cannot be empty")
	}
	return nil
}
