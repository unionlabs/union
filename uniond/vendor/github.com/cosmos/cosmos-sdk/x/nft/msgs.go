package nft

import (
	"cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

const (
	// TypeMsgSend nft message types
	TypeMsgSend = "send"
)

var _ sdk.Msg = &MsgSend{}

// ValidateBasic implements the Msg.ValidateBasic method.
func (m MsgSend) ValidateBasic() error {
	if len(m.ClassId) == 0 {
		return ErrEmptyClassID
	}

	if len(m.Id) == 0 {
		return ErrEmptyNFTID
	}

	_, err := sdk.AccAddressFromBech32(m.Sender)
	if err != nil {
		return errors.Wrapf(sdkerrors.ErrInvalidAddress, "Invalid sender address (%s)", m.Sender)
	}

	_, err = sdk.AccAddressFromBech32(m.Receiver)
	if err != nil {
		return errors.Wrapf(sdkerrors.ErrInvalidAddress, "Invalid receiver address (%s)", m.Receiver)
	}
	return nil
}

// GetSigners returns the expected signers for MsgSend.
func (m MsgSend) GetSigners() []sdk.AccAddress {
	signer, _ := sdk.AccAddressFromBech32(m.Sender)
	return []sdk.AccAddress{signer}
}
