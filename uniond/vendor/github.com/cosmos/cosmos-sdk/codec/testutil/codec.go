package testutil

import (
	"github.com/cosmos/gogoproto/proto"

	coreaddress "cosmossdk.io/core/address"
	"cosmossdk.io/x/tx/signing"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/address"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
)

// CodecOptions are options for creating a test codec. If set, provided address codecs will be prioritized when
// building the InterfaceRegistry and ProtoCodec. If not set, new address bech32 codecs will be created using
// the provided prefixes.
type CodecOptions struct {
	AccAddressPrefix string
	ValAddressPrefix string
	AddressCodec     coreaddress.Codec
	ValidatorCodec   coreaddress.Codec
}

// NewCodecOptionsWithPrefixes returns CodecOptions with provided prefixes.
func NewCodecOptionsWithPrefixes(addressPrefix, validatorPrefix string) CodecOptions {
	return CodecOptions{
		AccAddressPrefix: addressPrefix,
		ValAddressPrefix: validatorPrefix,
	}
}

// NewCodecOptionsWithCodecs returns CodecOptions with provided address codecs.
func NewCodecOptionsWithCodecs(addressCodec, validatorCodec coreaddress.Codec) CodecOptions {
	return CodecOptions{
		AddressCodec:   addressCodec,
		ValidatorCodec: validatorCodec,
	}
}

// NewInterfaceRegistry returns a new InterfaceRegistry with the given options.
func (o CodecOptions) NewInterfaceRegistry() codectypes.InterfaceRegistry {
	ir, err := codectypes.NewInterfaceRegistryWithOptions(codectypes.InterfaceRegistryOptions{
		ProtoFiles: proto.HybridResolver,
		SigningOptions: signing.Options{
			AddressCodec:          o.GetAddressCodec(),
			ValidatorAddressCodec: o.GetValidatorCodec(),
		},
	})
	if err != nil {
		panic(err)
	}

	return ir
}

// NewCodec returns a new codec with the given options.
func (o CodecOptions) NewCodec() *codec.ProtoCodec {
	return codec.NewProtoCodec(o.NewInterfaceRegistry())
}

// GetAddressCodec returns the address codec. If not address codec was provided it'll create a new one based on the
// bech32 prefix.
func (o CodecOptions) GetAddressCodec() coreaddress.Codec {
	if o.AddressCodec != nil {
		return o.AddressCodec
	}

	accAddressPrefix := o.AccAddressPrefix
	if accAddressPrefix == "" {
		accAddressPrefix = "cosmos"
	}

	return address.NewBech32Codec(accAddressPrefix)
}

// GetValidatorCodec returns the validator address codec. If not validator codec was provided it'll create a new one
// based on the bech32 prefix.
func (o CodecOptions) GetValidatorCodec() coreaddress.Codec {
	if o.ValidatorCodec != nil {
		return o.ValidatorCodec
	}

	valAddressPrefix := o.ValAddressPrefix
	if valAddressPrefix == "" {
		valAddressPrefix = "cosmosvaloper"
	}

	return address.NewBech32Codec(valAddressPrefix)
}
