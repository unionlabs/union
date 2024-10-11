package feegrant

import (
	"github.com/cosmos/gogoproto/proto"
	gogoprotoany "github.com/cosmos/gogoproto/types/any"

	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

var (
	_, _ sdk.Msg                              = &MsgGrantAllowance{}, &MsgRevokeAllowance{}
	_    gogoprotoany.UnpackInterfacesMessage = &MsgGrantAllowance{}
)

// NewMsgGrantAllowance creates a new MsgGrantAllowance.
func NewMsgGrantAllowance(feeAllowance FeeAllowanceI, granter, grantee string) (*MsgGrantAllowance, error) {
	msg, ok := feeAllowance.(proto.Message)
	if !ok {
		return nil, errorsmod.Wrapf(sdkerrors.ErrPackAny, "cannot proto marshal %T", msg)
	}
	any, err := types.NewAnyWithValue(msg)
	if err != nil {
		return nil, err
	}

	return &MsgGrantAllowance{
		Granter:   granter,
		Grantee:   grantee,
		Allowance: any,
	}, nil
}

// GetFeeAllowanceI returns unpacked FeeAllowance
func (msg MsgGrantAllowance) GetFeeAllowanceI() (FeeAllowanceI, error) {
	allowance, ok := msg.Allowance.GetCachedValue().(FeeAllowanceI)
	if !ok {
		return nil, errorsmod.Wrap(ErrNoAllowance, "failed to get allowance")
	}

	return allowance, nil
}

// UnpackInterfaces implements UnpackInterfacesMessage.UnpackInterfaces
func (msg MsgGrantAllowance) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	var allowance FeeAllowanceI
	return unpacker.UnpackAny(msg.Allowance, &allowance)
}

// NewMsgRevokeAllowance returns a message to revoke a fee allowance for a given
// granter and grantee
func NewMsgRevokeAllowance(granter, grantee string) MsgRevokeAllowance {
	return MsgRevokeAllowance{Granter: granter, Grantee: grantee}
}
