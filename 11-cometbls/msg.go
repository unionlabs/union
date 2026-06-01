package cometbls

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// MsgMyFunction defines a message with sender and string parameter.
type MsgMyFunction struct {
	Sender sdk.AccAddress `json:"sender" yaml:"sender"` // This is msg.sender
	vkHex  string         `json:"vkHex" yaml:"vkHex"`
}

// NewMsgMyFunction creates a new MsgMyFunction.
func NewMsgMyFunction(sender sdk.AccAddress, vkHex string) MsgMyFunction {
	return MsgMyFunction{
		Sender: sender,
		vkHex:  vkHex,
	}
}
