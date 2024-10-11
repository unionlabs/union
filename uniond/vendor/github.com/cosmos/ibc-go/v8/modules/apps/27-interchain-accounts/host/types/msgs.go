package types

import (
	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"

	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
)

var (
	_ sdk.Msg              = (*MsgUpdateParams)(nil)
	_ sdk.HasValidateBasic = (*MsgUpdateParams)(nil)

	_ sdk.Msg              = (*MsgModuleQuerySafe)(nil)
	_ sdk.HasValidateBasic = (*MsgModuleQuerySafe)(nil)
)

// NewMsgUpdateParams creates a new MsgUpdateParams instance
func NewMsgUpdateParams(signer string, params Params) *MsgUpdateParams {
	return &MsgUpdateParams{
		Signer: signer,
		Params: params,
	}
}

// ValidateBasic implements sdk.HasValidateBasic
func (msg MsgUpdateParams) ValidateBasic() error {
	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}

	return msg.Params.Validate()
}

// GetSigners implements sdk.Msg
func (msg MsgUpdateParams) GetSigners() []sdk.AccAddress {
	accAddr, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}

	return []sdk.AccAddress{accAddr}
}

// NewMsgModuleQuerySafe creates a new MsgModuleQuerySafe instance
func NewMsgModuleQuerySafe(signer string, requests []*QueryRequest) *MsgModuleQuerySafe {
	return &MsgModuleQuerySafe{
		Signer:   signer,
		Requests: requests,
	}
}

// ValidateBasic implements sdk.HasValidateBasic
func (msg MsgModuleQuerySafe) ValidateBasic() error {
	_, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidAddress, "string could not be parsed as address: %v", err)
	}

	if len(msg.Requests) == 0 {
		return errorsmod.Wrapf(ibcerrors.ErrInvalidRequest, "no queries provided")
	}

	return nil
}

// GetSigners implements sdk.Msg
func (msg MsgModuleQuerySafe) GetSigners() []sdk.AccAddress {
	accAddr, err := sdk.AccAddressFromBech32(msg.Signer)
	if err != nil {
		panic(err)
	}

	return []sdk.AccAddress{accAddr}
}
