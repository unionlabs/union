package types

import (
	"cosmossdk.io/core/registry"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/msgservice"
)

// RegisterInterfaces registers the interchain accounts controller message types using the provided InterfaceRegistry
func RegisterInterfaces(registry registry.InterfaceRegistrar) {
	registry.RegisterImplementations(
		(*sdk.Msg)(nil),
		&MsgRegisterInterchainAccount{},
		&MsgSendTx{},
		&MsgUpdateParams{},
	)
	msgservice.RegisterMsgServiceDesc(registry, &_Msg_serviceDesc)
}
