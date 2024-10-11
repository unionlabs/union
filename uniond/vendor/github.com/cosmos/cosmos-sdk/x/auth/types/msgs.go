package types

import (
	coretransaction "cosmossdk.io/core/transaction"

	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// GetMessages returns the cache values from the MsgNonAtomicExec.Msgs if present.
func (msg MsgNonAtomicExec) GetMessages() ([]coretransaction.Msg, error) {
	msgs := make([]coretransaction.Msg, len(msg.Msgs))
	for i, msgAny := range msg.Msgs {
		msg, ok := msgAny.GetCachedValue().(coretransaction.Msg)
		if !ok {
			return nil, sdkerrors.ErrInvalidRequest.Wrapf("messages contains %T which is not a sdk.Msg", msgAny)
		}
		msgs[i] = msg
	}

	return msgs, nil
}
