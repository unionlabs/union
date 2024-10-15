package codec

import (
	"cosmossdk.io/core/registry"

	bls12_381 "github.com/cosmos/cosmos-sdk/crypto/keys/bls12_381"
	"github.com/cosmos/cosmos-sdk/crypto/keys/bn254"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	kmultisig "github.com/cosmos/cosmos-sdk/crypto/keys/multisig"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
)

// RegisterCrypto registers all crypto dependency types with the provided Amino
// codec.
func RegisterCrypto(registrar registry.AminoRegistrar) {
	registrar.RegisterInterface((*cryptotypes.PubKey)(nil), nil)
	registrar.RegisterConcrete(&ed25519.PubKey{},
		ed25519.PubKeyName)
	registrar.RegisterConcrete(&secp256k1.PubKey{},
		secp256k1.PubKeyName)
	registrar.RegisterConcrete(&bls12_381.PubKey{}, bls12_381.PubKeyName)
	registrar.RegisterConcrete(&bn254.PubKey{}, bn254.PubKeyName)
	registrar.RegisterConcrete(&kmultisig.LegacyAminoPubKey{},
		kmultisig.PubKeyAminoRoute)
	registrar.RegisterInterface((*cryptotypes.PrivKey)(nil), nil)
	registrar.RegisterConcrete(&ed25519.PrivKey{},
		ed25519.PrivKeyName)
	registrar.RegisterConcrete(&secp256k1.PrivKey{},
		secp256k1.PrivKeyName)
	registrar.RegisterConcrete(&bls12_381.PrivKey{}, bls12_381.PrivKeyName)
	registrar.RegisterConcrete(&bn254.PrivKey{}, bn254.PrivKeyName)
}
