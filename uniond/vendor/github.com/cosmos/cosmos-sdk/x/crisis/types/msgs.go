package types

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// ensure Msg interface compliance at compile time
var (
	_, _ sdk.Msg = &MsgVerifyInvariant{}, &MsgUpdateParams{}
)

// NewMsgVerifyInvariant creates a new MsgVerifyInvariant object
func NewMsgVerifyInvariant(sender sdk.AccAddress, invModeName, invRoute string) *MsgVerifyInvariant {
	return &MsgVerifyInvariant{
		Sender:              sender.String(),
		InvariantModuleName: invModeName,
		InvariantRoute:      invRoute,
	}
}

// FullInvariantRoute - get the messages full invariant route
func (msg MsgVerifyInvariant) FullInvariantRoute() string {
	return msg.InvariantModuleName + "/" + msg.InvariantRoute
}
