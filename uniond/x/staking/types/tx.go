package types

import (
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var (
	_ sdk.Msg                            = MsgCreateUnionValidator{}
	_ codectypes.UnpackInterfacesMessage = (*MsgCreateUnionValidator)(nil)
)

func (msg MsgCreateUnionValidator) UnpackInterfaces(unpacker codectypes.AnyUnpacker) error {
	return msg.Underlying.UnpackInterfaces(unpacker)
}
