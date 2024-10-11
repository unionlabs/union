package legacytx

import "cosmossdk.io/core/registry"

func RegisterLegacyAminoCodec(registrar registry.AminoRegistrar) {
	registrar.RegisterConcrete(StdTx{}, "cosmos-sdk/StdTx")
}
