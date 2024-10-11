package codec

import (
	"fmt"

	"github.com/cosmos/gogoproto/proto"

	authmodulev1 "cosmossdk.io/api/cosmos/auth/module/v1"
	stakingmodulev1 "cosmossdk.io/api/cosmos/staking/module/v1"
	"cosmossdk.io/core/address"
	"cosmossdk.io/core/registry"
	"cosmossdk.io/depinject"
	"cosmossdk.io/x/tx/signing"

	addresscodec "github.com/cosmos/cosmos-sdk/codec/address"
	"github.com/cosmos/cosmos-sdk/codec/types"
)

func ProvideInterfaceRegistry(
	addressCodec address.Codec,
	validatorAddressCodec address.ValidatorAddressCodec,
	customGetSigners []signing.CustomGetSigner,
) (types.InterfaceRegistry, registry.InterfaceRegistrar, error) {
	signingOptions := signing.Options{
		AddressCodec:          addressCodec,
		ValidatorAddressCodec: validatorAddressCodec,
	}
	for _, signer := range customGetSigners {
		signingOptions.DefineCustomGetSigners(signer.MsgType, signer.Fn)
	}

	interfaceRegistry, err := types.NewInterfaceRegistryWithOptions(types.InterfaceRegistryOptions{
		ProtoFiles:     proto.HybridResolver,
		SigningOptions: signingOptions,
	})
	if err != nil {
		return nil, nil, fmt.Errorf("failed to create interface registry: %w", err)
	}

	if err := interfaceRegistry.SigningContext().Validate(); err != nil {
		return nil, nil, fmt.Errorf("failed to validate signing context: %w", err)
	}

	return interfaceRegistry, interfaceRegistry, nil
}

func ProvideLegacyAmino() registry.AminoRegistrar {
	return NewLegacyAmino()
}

func ProvideProtoCodec(interfaceRegistry types.InterfaceRegistry) *ProtoCodec {
	return NewProtoCodec(interfaceRegistry)
}

type AddressCodecInputs struct {
	depinject.In

	AuthConfig    *authmodulev1.Module    `optional:"true"`
	StakingConfig *stakingmodulev1.Module `optional:"true"`

	AddressCodecFactory          func() address.Codec                 `optional:"true"`
	ValidatorAddressCodecFactory func() address.ValidatorAddressCodec `optional:"true"`
	ConsensusAddressCodecFactory func() address.ConsensusAddressCodec `optional:"true"`
}

// ProvideAddressCodec provides an address.Codec to the container for any
// modules that want to do address string <> bytes conversion.
func ProvideAddressCodec(in AddressCodecInputs) (address.Codec, address.ValidatorAddressCodec, address.ConsensusAddressCodec) {
	if in.AddressCodecFactory != nil && in.ValidatorAddressCodecFactory != nil && in.ConsensusAddressCodecFactory != nil {
		return in.AddressCodecFactory(), in.ValidatorAddressCodecFactory(), in.ConsensusAddressCodecFactory()
	}

	if in.AuthConfig == nil || in.AuthConfig.Bech32Prefix == "" {
		panic("auth config bech32 prefix cannot be empty if no custom address codec is provided")
	}

	if in.StakingConfig == nil {
		in.StakingConfig = &stakingmodulev1.Module{}
	}

	if in.StakingConfig.Bech32PrefixValidator == "" {
		in.StakingConfig.Bech32PrefixValidator = fmt.Sprintf("%svaloper", in.AuthConfig.Bech32Prefix)
	}

	if in.StakingConfig.Bech32PrefixConsensus == "" {
		in.StakingConfig.Bech32PrefixConsensus = fmt.Sprintf("%svalcons", in.AuthConfig.Bech32Prefix)
	}

	return addresscodec.NewBech32Codec(in.AuthConfig.Bech32Prefix),
		addresscodec.NewBech32Codec(in.StakingConfig.Bech32PrefixValidator),
		addresscodec.NewBech32Codec(in.StakingConfig.Bech32PrefixConsensus)
}
