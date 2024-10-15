package feegrant

import (
	"cosmossdk.io/core/registry"
	coretransaction "cosmossdk.io/core/transaction"

	"github.com/cosmos/cosmos-sdk/codec/legacy"
	"github.com/cosmos/cosmos-sdk/types/msgservice"
)

// RegisterLegacyAminoCodec registers the necessary x/feegrant interfaces and concrete types
// on the provided LegacyAmino codec. These types are used for Amino JSON serialization.
func RegisterLegacyAminoCodec(registrar registry.AminoRegistrar) {
	legacy.RegisterAminoMsg(registrar, &MsgGrantAllowance{}, "cosmos-sdk/MsgGrantAllowance")
	legacy.RegisterAminoMsg(registrar, &MsgRevokeAllowance{}, "cosmos-sdk/MsgRevokeAllowance")

	registrar.RegisterInterface((*FeeAllowanceI)(nil), nil)
	registrar.RegisterConcrete(&BasicAllowance{}, "cosmos-sdk/BasicAllowance")
	registrar.RegisterConcrete(&PeriodicAllowance{}, "cosmos-sdk/PeriodicAllowance")
	registrar.RegisterConcrete(&AllowedMsgAllowance{}, "cosmos-sdk/AllowedMsgAllowance")
}

// RegisterInterfaces registers the interfaces types with the interface registry
func RegisterInterfaces(registrar registry.InterfaceRegistrar) {
	registrar.RegisterImplementations((*coretransaction.Msg)(nil),
		&MsgGrantAllowance{},
		&MsgRevokeAllowance{},
	)

	registrar.RegisterInterface(
		"cosmos.feegrant.v1beta1.FeeAllowanceI",
		(*FeeAllowanceI)(nil),
		&BasicAllowance{},
		&PeriodicAllowance{},
		&AllowedMsgAllowance{},
	)

	msgservice.RegisterMsgServiceDesc(registrar, &_Msg_serviceDesc)
}
