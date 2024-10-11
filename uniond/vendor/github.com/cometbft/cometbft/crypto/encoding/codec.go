package encoding

import (
	"fmt"
	"reflect"

	pc "github.com/cometbft/cometbft/api/cometbft/crypto/v1"
	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/bls12381"
	"github.com/cometbft/cometbft/crypto/bn254"
	"github.com/cometbft/cometbft/crypto/ed25519"
	"github.com/cometbft/cometbft/crypto/secp256k1"
	"github.com/cometbft/cometbft/libs/json"
)

// ErrUnsupportedKey describes an error resulting from the use of an
// unsupported key in [PubKeyToProto] or [PubKeyFromProto].
type ErrUnsupportedKey struct {
	KeyType string
}

func (e ErrUnsupportedKey) Error() string {
	return "encoding: unsupported key " + e.KeyType
}

// InvalidKeyLen describes an error resulting from the use of a key with
// an invalid length in [PubKeyFromProto].
type ErrInvalidKeyLen struct {
	Key       any
	Got, Want int
}

func (e ErrInvalidKeyLen) Error() string {
	return fmt.Sprintf("encoding: invalid key length for %v, got %d, want %d", e.Key, e.Got, e.Want)
}

func init() {
	json.RegisterType((*pc.PublicKey)(nil), "tendermint.crypto.PublicKey")
	json.RegisterType((*pc.PublicKey_Ed25519)(nil), "tendermint.crypto.PublicKey_Ed25519")
	json.RegisterType((*pc.PublicKey_Secp256K1)(nil), "tendermint.crypto.PublicKey_Secp256K1")
	if bls12381.Enabled {
		json.RegisterType((*pc.PublicKey_Bls12381)(nil), "tendermint.crypto.PublicKey_Bls12381")
	}
}

// PubKeyToProto takes crypto.PubKey and transforms it to a protobuf Pubkey. It
// returns ErrUnsupportedKey if the pubkey type is unsupported.
func PubKeyToProto(k crypto.PubKey) (pc.PublicKey, error) {
	var kp pc.PublicKey

	if k == nil {
		return kp, ErrUnsupportedKey{KeyType: "<nil>"}
	}

	switch k.Type() {
	case ed25519.KeyType:
		kp = pc.PublicKey{
			Sum: &pc.PublicKey_Ed25519{
				Ed25519: k.Bytes(),
			},
		}
	case secp256k1.KeyType:
		kp = pc.PublicKey{
			Sum: &pc.PublicKey_Secp256K1{
				Secp256K1: k.Bytes(),
			},
		}
	case bls12381.KeyType:
		if !bls12381.Enabled {
			return kp, ErrUnsupportedKey{KeyType: bls12381.KeyType}
		}

		kp = pc.PublicKey{
			Sum: &pc.PublicKey_Bls12381{
				Bls12381: k.Bytes(),
			},
		}
	case bn254.KeyType:
		kp = pc.PublicKey{
			Sum: &pc.PublicKey_Bn254{
				Bn254: k.Bytes(),
			},
		}
	default:
		return kp, ErrUnsupportedKey{KeyType: k.Type()}
	}
	return kp, nil
}

// PubKeyFromProto takes a protobuf Pubkey and transforms it to a
// crypto.Pubkey. It returns ErrUnsupportedKey if the pubkey type is
// unsupported or ErrInvalidKeyLen if the key length is invalid.
func PubKeyFromProto(k pc.PublicKey) (crypto.PubKey, error) {
	switch k := k.Sum.(type) {
	case *pc.PublicKey_Ed25519:
		if len(k.Ed25519) != ed25519.PubKeySize {
			return nil, ErrInvalidKeyLen{
				Key:  k,
				Got:  len(k.Ed25519),
				Want: ed25519.PubKeySize,
			}
		}
		pk := make(ed25519.PubKey, ed25519.PubKeySize)
		copy(pk, k.Ed25519)
		return pk, nil
	case *pc.PublicKey_Secp256K1:
		if len(k.Secp256K1) != secp256k1.PubKeySize {
			return nil, ErrInvalidKeyLen{
				Key:  k,
				Got:  len(k.Secp256K1),
				Want: secp256k1.PubKeySize,
			}
		}
		pk := make(secp256k1.PubKey, secp256k1.PubKeySize)
		copy(pk, k.Secp256K1)
		return pk, nil
	case *pc.PublicKey_Bls12381:
		if !bls12381.Enabled {
			return nil, ErrUnsupportedKey{KeyType: bls12381.KeyType}
		}

		if len(k.Bls12381) != bls12381.PubKeySize {
			return nil, ErrInvalidKeyLen{
				Key:  k,
				Got:  len(k.Bls12381),
				Want: bls12381.PubKeySize,
			}
		}
		return bls12381.NewPublicKeyFromBytes(k.Bls12381)
	case *pc.PublicKey_Bn254:
		if len(k.Bn254) != bn254.PubKeySize {
			return nil, ErrInvalidKeyLen{
				Key:  k,
				Got:  len(k.Bn254),
				Want: bn254.PubKeySize,
			}
		}
		pk := make(bn254.PubKey, bn254.PubKeySize)
		copy(pk, k.Bn254)
		return pk, nil
	default:
		kt := reflect.TypeOf(k)
		if kt == nil {
			return nil, ErrUnsupportedKey{KeyType: "<nil>"}
		} else {
			return nil, ErrUnsupportedKey{KeyType: kt.String()}
		}
	}
}

// PubKeyFromTypeAndBytes builds a crypto.PubKey from the given type and bytes.
// It returns ErrUnsupportedKey if the pubkey type is unsupported or
// ErrInvalidKeyLen if the key length is invalid.
func PubKeyFromTypeAndBytes(pkType string, bytes []byte) (crypto.PubKey, error) {
	var pubKey crypto.PubKey
	switch pkType {
	case ed25519.KeyType:
		if len(bytes) != ed25519.PubKeySize {
			return nil, ErrInvalidKeyLen{
				Key:  pkType,
				Got:  len(bytes),
				Want: ed25519.PubKeySize,
			}
		}

		pk := make(ed25519.PubKey, ed25519.PubKeySize)
		copy(pk, bytes)
		pubKey = pk
	case secp256k1.KeyType:
		if len(bytes) != secp256k1.PubKeySize {
			return nil, ErrInvalidKeyLen{
				Key:  pkType,
				Got:  len(bytes),
				Want: secp256k1.PubKeySize,
			}
		}

		pk := make(secp256k1.PubKey, secp256k1.PubKeySize)
		copy(pk, bytes)
		pubKey = pk
	case bls12381.KeyType:
		if !bls12381.Enabled {
			return nil, ErrUnsupportedKey{KeyType: pkType}
		}

		if len(bytes) != bls12381.PubKeySize {
			return nil, ErrInvalidKeyLen{
				Key:  pkType,
				Got:  len(bytes),
				Want: bls12381.PubKeySize,
			}
		}

		return bls12381.NewPublicKeyFromBytes(bytes)
	case bn254.KeyType:
		if len(bytes) != bn254.PubKeySize {
			return nil, ErrInvalidKeyLen{
				Key:  pkType,
				Got:  len(bytes),
				Want: bn254.PubKeySize,
			}
		}

		pk := make(bn254.PubKey, bn254.PubKeySize)
		copy(pk, bytes)
		pubKey = pk
	default:
		return nil, ErrUnsupportedKey{KeyType: pkType}
	}
	return pubKey, nil
}
