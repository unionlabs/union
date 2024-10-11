package codec

import (
	"cosmossdk.io/errors"

	cryptokeys "github.com/cosmos/cosmos-sdk/crypto/keys"
	bls12_381 "github.com/cosmos/cosmos-sdk/crypto/keys/bls12_381"
	bn254 "github.com/cosmos/cosmos-sdk/crypto/keys/bn254"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

func PubKeyToProto(pk cryptokeys.JSONPubkey) (cryptotypes.PubKey, error) {
	switch pk.KeyType {
	case ed25519.PubKeyName:
		return &ed25519.PubKey{
			Key: pk.Value,
		}, nil
	case secp256k1.PubKeyName:
		return &secp256k1.PubKey{
			Key: pk.Value,
		}, nil
	case bls12_381.PubKeyName:
		return &bls12_381.PubKey{
			Key: pk.Value,
		}, nil
	case bn254.PubKeyName:
		return &bn254.PubKey{
			Key: pk.Value,
		}, nil
	default:
		return nil, errors.Wrapf(sdkerrors.ErrInvalidType, "cannot convert %v to proto public key", pk)
	}
}

func PubKeyFromProto(pk cryptotypes.PubKey) (cryptokeys.JSONPubkey, error) {
	switch pk := pk.(type) {
	case *ed25519.PubKey:
		return cryptokeys.JSONPubkey{
			KeyType: ed25519.PubKeyName,
			Value:   pk.Bytes(),
		}, nil
	case *secp256k1.PubKey:
		return cryptokeys.JSONPubkey{
			KeyType: secp256k1.PubKeyName,
			Value:   pk.Bytes(),
		}, nil
	case *bls12_381.PubKey:
		return cryptokeys.JSONPubkey{
			KeyType: bls12_381.PubKeyName,
			Value:   pk.Bytes(),
		}, nil
	case *bn254.PubKey:
		return cryptokeys.JSONPubkey{
			KeyType: bn254.PrivKeyName,
			Value:   pk.Bytes(),
		}, nil
	default:
		return cryptokeys.JSONPubkey{}, errors.Wrapf(sdkerrors.ErrInvalidType, "cannot convert %v from proto public key", pk)
	}
}
