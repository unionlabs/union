package types

import (
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/msgservice"

	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// RegisterInterfaces registers the Wasm concrete client-related
// implementations and interfaces.
func RegisterInterfaces(registry codectypes.InterfaceRegistry) {
	registry.RegisterImplementations(
		(*exported.ClientState)(nil),
		&ClientState{},
	)
	registry.RegisterImplementations(
		(*exported.ConsensusState)(nil),
		&ConsensusState{},
	)
	registry.RegisterImplementations(
		(*exported.ClientMessage)(nil),
		&ClientMessage{},
	)
	registry.RegisterImplementations(
		(*sdk.Msg)(nil),
		&MsgStoreCode{},
	)
	registry.RegisterImplementations(
		(*sdk.Msg)(nil),
		&MsgMigrateContract{},
	)
	registry.RegisterImplementations(
		(*sdk.Msg)(nil),
		&MsgRemoveChecksum{},
	)

	msgservice.RegisterMsgServiceDesc(registry, &_Msg_serviceDesc)
}
