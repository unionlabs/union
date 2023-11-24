package feegrant

import (
	"github.com/cosmos/gogoproto/proto"

	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

var _ types.UnpackInterfacesMessage = &Grant{}

// NewGrant creates a new FeeAllowanceGrant.
func NewGrant(granter, grantee sdk.AccAddress, feeAllowance FeeAllowanceI) (Grant, error) {
	msg, ok := feeAllowance.(proto.Message)
	if !ok {
		return Grant{}, errorsmod.Wrapf(sdkerrors.ErrPackAny, "cannot proto marshal %T", feeAllowance)
	}

	any, err := types.NewAnyWithValue(msg)
	if err != nil {
		return Grant{}, err
	}

	return Grant{
		Granter:   granter.String(),
		Grantee:   grantee.String(),
		Allowance: any,
	}, nil
}

// ValidateBasic performs basic validation on
// FeeAllowanceGrant
func (a Grant) ValidateBasic() error {
	if a.Granter == "" {
		return errorsmod.Wrap(sdkerrors.ErrInvalidAddress, "missing granter address")
	}
	if a.Grantee == "" {
		return errorsmod.Wrap(sdkerrors.ErrInvalidAddress, "missing grantee address")
	}
	if a.Grantee == a.Granter {
		return errorsmod.Wrap(sdkerrors.ErrInvalidAddress, "cannot self-grant fee authorization")
	}

	f, err := a.GetGrant()
	if err != nil {
		return err
	}

	return f.ValidateBasic()
}

// GetGrant unpacks allowance
func (a Grant) GetGrant() (FeeAllowanceI, error) {
	allowance, ok := a.Allowance.GetCachedValue().(FeeAllowanceI)
	if !ok {
		return nil, errorsmod.Wrap(ErrNoAllowance, "failed to get allowance")
	}

	return allowance, nil
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (a Grant) UnpackInterfaces(unpacker types.AnyUnpacker) error {
	var allowance FeeAllowanceI
	return unpacker.UnpackAny(a.Allowance, &allowance)
}
