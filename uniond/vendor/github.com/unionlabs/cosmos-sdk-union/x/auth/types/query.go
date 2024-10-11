package types

import (
	gogoprotoany "github.com/cosmos/gogoproto/types/any"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

func (m *QueryAccountResponse) UnpackInterfaces(unpacker gogoprotoany.AnyUnpacker) error {
	var account sdk.AccountI
	return unpacker.UnpackAny(m.Account, &account)
}

var _ gogoprotoany.UnpackInterfacesMessage = &QueryAccountResponse{}
