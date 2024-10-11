package types

import (
	"cosmossdk.io/core/registry"
	coretransaction "cosmossdk.io/core/transaction"

	"github.com/cosmos/cosmos-sdk/codec/legacy"
	"github.com/cosmos/cosmos-sdk/types/msgservice"
)

// RegisterLegacyAminoCodec registers concrete types on the LegacyAmino codec
func RegisterLegacyAminoCodec(registrar registry.AminoRegistrar) {
	registrar.RegisterConcrete(Plan{}, "cosmos-sdk/Plan")
	registrar.RegisterConcrete(&SoftwareUpgradeProposal{}, "cosmos-sdk/SoftwareUpgradeProposal")
	registrar.RegisterConcrete(&CancelSoftwareUpgradeProposal{}, "cosmos-sdk/CancelSoftwareUpgradeProposal")
	legacy.RegisterAminoMsg(registrar, &MsgSoftwareUpgrade{}, "cosmos-sdk/MsgSoftwareUpgrade")
	legacy.RegisterAminoMsg(registrar, &MsgCancelUpgrade{}, "cosmos-sdk/MsgCancelUpgrade")
}

// RegisterInterfaces registers the interfaces types with the Interface Registry.
func RegisterInterfaces(registrar registry.InterfaceRegistrar) {
	registrar.RegisterImplementations((*coretransaction.Msg)(nil),
		&MsgSoftwareUpgrade{},
		&MsgCancelUpgrade{},
	)

	msgservice.RegisterMsgServiceDesc(registrar, &_Msg_serviceDesc)
}
