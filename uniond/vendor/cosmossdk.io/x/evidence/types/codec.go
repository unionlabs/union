package types

import (
	"cosmossdk.io/core/registry"
	coretransaction "cosmossdk.io/core/transaction"
	"cosmossdk.io/x/evidence/exported"

	"github.com/cosmos/cosmos-sdk/codec/legacy"
	"github.com/cosmos/cosmos-sdk/types/msgservice"
)

// RegisterLegacyAminoCodec registers all the necessary types and interfaces for the
// evidence module.
func RegisterLegacyAminoCodec(registrar registry.AminoRegistrar) {
	registrar.RegisterInterface((*exported.Evidence)(nil), nil)
	legacy.RegisterAminoMsg(registrar, &MsgSubmitEvidence{}, "cosmos-sdk/MsgSubmitEvidence")
	registrar.RegisterConcrete(&Equivocation{}, "cosmos-sdk/Equivocation")
}

// RegisterInterfaces registers the interfaces types with the interface registry.
func RegisterInterfaces(registrar registry.InterfaceRegistrar) {
	registrar.RegisterImplementations((*coretransaction.Msg)(nil), &MsgSubmitEvidence{})
	registrar.RegisterInterface(
		"cosmos.evidence.v1beta1.Evidence",
		(*exported.Evidence)(nil),
		&Equivocation{},
	)

	msgservice.RegisterMsgServiceDesc(registrar, &_Msg_serviceDesc)
}
