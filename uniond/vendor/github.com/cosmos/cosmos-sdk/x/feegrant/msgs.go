package feegrant

import (
	"github.com/cosmos/gogoproto/proto"

	"github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/auth/migrations/legacytx"
)

var (
	_, _ sdk.Msg            = &MsgGrantAllowance{}, &MsgRevokeAllowance{}
	_, _ legacytx.LegacyMsg = &MsgGrantAllowance{}, &MsgRevokeAllowance{} // For amino support.

	_ types.UnpackInterfacesMessage = &MsgGrantAllowance{}
)

// NewMsgGrantAllowance creates a new MsgGrantAllowance.
//
//nolint:interfacer
func NewMsgGrantAllowance(feeAllowance FeeAllowanceI, granter, grantee sdk.AccAddress) (*MsgGrantAllowance, error) {
	msg, ok := feeAllowance.(proto.Message)
	if !ok {
		return nil, sdkerrors.Wrapf(sdkerrors.ErrPackAny, "cannot proto marshal %T", msg)
	}
	any, err := types.NewAnyWithValue(msg)
	if err != nil {
		return nil, err
	}

	return &MsgGrantAllowance{
		Granter:   granter.String(),
		Grantee:   grantee.String(),
		Allowance: any,
	}, nil
}

// ValidateBasic implements the sdk.Msg interface.
func (msg MsgGrantAllowance) ValidateBasic() error {
	if _, err := sdk.AccAddressFromBech32(msg.Granter); err != nil {
		return sdkerrors.ErrInvalidAddress.Wrapf("invalid granter address: %s", err)
	}
	if _, err := sdk.AccAddressFromBech32(msg.Grantee); err != nil {
		return sdkerrors.ErrInvalidAddress.Wrapf("invalid grantee address: %s", err)
	}
	if msg.Grantee == msg.Granter {
		return sdkerrors.Wrap(sdkerrors.ErrInvalidAddress, "cannot self-grant fee authorization")
	}
	allowance, err := msg.GetFeeAllowanceI()
	if err != nil {
		return err
	}

	return allowance.ValidateBasic()
}

// GetSigners gets the granter account associated with an allowance
func (msg MsgGrantAllowance) GetSigners() []sdk.AccAddress {
	granter, _ := sdk.AccAddressFromBech32(msg.Granter)
	return []sdk.AccAddress{granter}
}

// Type implements the LegacyMsg.Type method.
func (msg MsgGrantAllowance) Type() string {
	return sdk.MsgTypeURL(&msg)
}

// Route implements the LegacyMsg.Route method.
func (msg MsgGrantAllowance) Route() string {
	return sdk.MsgTypeURL(&msg)
}

// GetSignBytes implements the LegacyMsg.GetSignBytes method.
func (msg MsgGrantAllowance) GetSignBytes() []byte {
	return sdk.MustSortJSON(ModuleCdc.MustMarshalJSON(&msg))
}

// GetFeeAllowanceI returns unpacked FeeAllowance
func (msg MsgGrantAllowance) GetFeeAllowanceI() (FeeAllowanceI, error) {
	allowance, ok := msg.Allowance.GetCachedValue().(FeeAllowanceI)
	if !ok {
		return nil, sdkerrors.Wrap(ErrNoAllowance, "failed to get allowance")
	}

	return allowance, nil
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (msg MsgGrantAllowance) UnpackInterfaces(unpacker types.AnyUnpacker) error {
	var allowance FeeAllowanceI
	return unpacker.UnpackAny(msg.Allowance, &allowance)
}

// NewMsgRevokeAllowance returns a message to revoke a fee allowance for a given
// granter and grantee
//
//nolint:interfacer
func NewMsgRevokeAllowance(granter sdk.AccAddress, grantee sdk.AccAddress) MsgRevokeAllowance {
	return MsgRevokeAllowance{Granter: granter.String(), Grantee: grantee.String()}
}

// ValidateBasic implements the sdk.Msg interface.
func (msg MsgRevokeAllowance) ValidateBasic() error {
	if _, err := sdk.AccAddressFromBech32(msg.Granter); err != nil {
		return sdkerrors.ErrInvalidAddress.Wrapf("invalid granter address: %s", err)
	}
	if _, err := sdk.AccAddressFromBech32(msg.Grantee); err != nil {
		return sdkerrors.ErrInvalidAddress.Wrapf("invalid grantee address: %s", err)
	}
	if msg.Grantee == msg.Granter {
		return sdkerrors.Wrap(sdkerrors.ErrInvalidAddress, "addresses must be different")
	}

	return nil
}

// GetSigners gets the granter address associated with an Allowance
// to revoke.
func (msg MsgRevokeAllowance) GetSigners() []sdk.AccAddress {
	granter, _ := sdk.AccAddressFromBech32(msg.Granter)
	return []sdk.AccAddress{granter}
}

// Type implements the LegacyMsg.Type method.
func (msg MsgRevokeAllowance) Type() string {
	return sdk.MsgTypeURL(&msg)
}

// Route implements the LegacyMsg.Route method.
func (msg MsgRevokeAllowance) Route() string {
	return sdk.MsgTypeURL(&msg)
}

// GetSignBytes implements the LegacyMsg.GetSignBytes method.
func (msg MsgRevokeAllowance) GetSignBytes() []byte {
	return sdk.MustSortJSON(ModuleCdc.MustMarshalJSON(&msg))
}
