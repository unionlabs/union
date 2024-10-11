package proposal

import (
	"cosmossdk.io/core/registry"
	govtypes "cosmossdk.io/x/gov/types/v1beta1"
)

// RegisterLegacyAminoCodec registers all necessary param module types with a given LegacyAmino codec.
func RegisterLegacyAminoCodec(registrar registry.AminoRegistrar) {
	registrar.RegisterConcrete(&ParameterChangeProposal{}, "cosmos-sdk/ParameterChangeProposal")
}

func RegisterInterfaces(registrar registry.InterfaceRegistrar) {
	registrar.RegisterImplementations(
		(*govtypes.Content)(nil),
		&ParameterChangeProposal{},
	)
}
