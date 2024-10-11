package codec

import (
	cmtprotocrypto "github.com/cometbft/cometbft/api/cometbft/crypto/v1"
	cmtcrypto "github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/encoding"

	"cosmossdk.io/errors"

	bls12_381 "github.com/cosmos/cosmos-sdk/crypto/keys/bls12_381"
	"github.com/cosmos/cosmos-sdk/crypto/keys/bn254"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// FromCmtProtoPublicKey converts a CMT's cmtprotocrypto.PublicKey into our own PubKey.
func FromCmtProtoPublicKey(protoPk cmtprotocrypto.PublicKey) (cryptotypes.PubKey, error) {
	switch protoPk := protoPk.Sum.(type) {
	case *cmtprotocrypto.PublicKey_Ed25519:
		return &ed25519.PubKey{
			Key: protoPk.Ed25519,
		}, nil
	case *cmtprotocrypto.PublicKey_Secp256K1:
		return &secp256k1.PubKey{
			Key: protoPk.Secp256K1,
		}, nil
	case *cmtprotocrypto.PublicKey_Bls12381:
		return &bls12_381.PubKey{
			Key: protoPk.Bls12381,
		}, nil
	case *cmtprotocrypto.PublicKey_Bn254:
		return &bn254.PubKey{
			Key: protoPk.Bn254,
		}, nil
	default:
		return nil, errors.Wrapf(sdkerrors.ErrInvalidType, "cannot convert %v from Tendermint public key", protoPk)
	}
}

// ToCmtProtoPublicKey converts our own PubKey to Cmt's cmtprotocrypto.PublicKey.
func ToCmtProtoPublicKey(pk cryptotypes.PubKey) (cmtprotocrypto.PublicKey, error) {
	switch pk := pk.(type) {
	case *ed25519.PubKey:
		return cmtprotocrypto.PublicKey{
			Sum: &cmtprotocrypto.PublicKey_Ed25519{
				Ed25519: pk.Key,
			},
		}, nil
	case *secp256k1.PubKey:
		return cmtprotocrypto.PublicKey{
			Sum: &cmtprotocrypto.PublicKey_Secp256K1{
				Secp256K1: pk.Key,
			},
		}, nil
	case *bls12_381.PubKey:
		return cmtprotocrypto.PublicKey{
			Sum: &cmtprotocrypto.PublicKey_Bls12381{
				Bls12381: pk.Key,
			},
		}, nil
	case *bn254.PubKey:
		return cmtprotocrypto.PublicKey{
			Sum: &cmtprotocrypto.PublicKey_Bn254{
				Bn254: pk.Key,
			},
		}, nil
	default:
		return cmtprotocrypto.PublicKey{}, errors.Wrapf(sdkerrors.ErrInvalidType, "cannot convert %v to Tendermint public key", pk)
	}
}

// FromCmtPubKeyInterface converts CMT's cmtcrypto.PubKey to our own PubKey.
func FromCmtPubKeyInterface(tmPk cmtcrypto.PubKey) (cryptotypes.PubKey, error) {
	tmProtoPk, err := encoding.PubKeyToProto(tmPk)
	if err != nil {
		return nil, err
	}

	return FromCmtProtoPublicKey(tmProtoPk)
}

// ToCmtPubKeyInterface converts our own PubKey to CMT's cmtcrypto.PubKey.
func ToCmtPubKeyInterface(pk cryptotypes.PubKey) (cmtcrypto.PubKey, error) {
	tmProtoPk, err := ToCmtProtoPublicKey(pk)
	if err != nil {
		return nil, err
	}

	return encoding.PubKeyFromProto(tmProtoPk)
}
