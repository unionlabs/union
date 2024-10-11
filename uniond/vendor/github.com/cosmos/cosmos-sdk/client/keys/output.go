package keys

import (
	"cosmossdk.io/core/address"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
)

// Use protobuf interface marshaler rather then generic JSON

// KeyOutput defines a structure wrapping around an Info object used for output
// functionality.
type KeyOutput struct {
	Name     string `json:"name" yaml:"name"`
	Type     string `json:"type" yaml:"type"`
	Address  string `json:"address" yaml:"address"`
	PubKey   string `json:"pubkey" yaml:"pubkey"`
	Mnemonic string `json:"mnemonic,omitempty" yaml:"mnemonic"`
}

// NewKeyOutput creates a default KeyOutput instance without Mnemonic, Threshold and PubKeys
func NewKeyOutput(name string, keyType keyring.KeyType, addr []byte, pk cryptotypes.PubKey, addressCodec address.Codec) (KeyOutput, error) {
	apk, err := codectypes.NewAnyWithValue(pk)
	if err != nil {
		return KeyOutput{}, err
	}

	bz, err := codec.ProtoMarshalJSON(apk, nil)
	if err != nil {
		return KeyOutput{}, err
	}

	addrStr, err := addressCodec.BytesToString(addr)
	if err != nil {
		return KeyOutput{}, err
	}

	return KeyOutput{
		Name:    name,
		Type:    keyType.String(),
		Address: addrStr,
		PubKey:  string(bz),
	}, nil
}

// MkConsKeyOutput create a KeyOutput for consensus addresses.
func MkConsKeyOutput(k *keyring.Record, consensusAddressCodec address.Codec) (KeyOutput, error) {
	pk, err := k.GetPubKey()
	if err != nil {
		return KeyOutput{}, err
	}
	return NewKeyOutput(k.Name, k.GetType(), pk.Address(), pk, consensusAddressCodec)
}

// MkValKeyOutput create a KeyOutput for validator addresses.
func MkValKeyOutput(k *keyring.Record, validatorAddressCodec address.Codec) (KeyOutput, error) {
	pk, err := k.GetPubKey()
	if err != nil {
		return KeyOutput{}, err
	}

	return NewKeyOutput(k.Name, k.GetType(), pk.Address(), pk, validatorAddressCodec)
}

// MkAccKeyOutput create a KeyOutput in with "acc" Bech32 prefixes. If the
// public key is a multisig public key, then the threshold and constituent
// public keys will be added.
func MkAccKeyOutput(k *keyring.Record, addressCodec address.Codec) (KeyOutput, error) {
	pk, err := k.GetPubKey()
	if err != nil {
		return KeyOutput{}, err
	}
	return NewKeyOutput(k.Name, k.GetType(), pk.Address(), pk, addressCodec)
}

// MkAccKeysOutput returns a slice of KeyOutput objects, each with the "acc"
// Bech32 prefixes, given a slice of Record objects. It returns an error if any
// call to MkKeyOutput fails.
func MkAccKeysOutput(records []*keyring.Record, addressCodec address.Codec) ([]KeyOutput, error) {
	kos := make([]KeyOutput, len(records))
	var err error
	for i, r := range records {
		kos[i], err = MkAccKeyOutput(r, addressCodec)
		if err != nil {
			return nil, err
		}
	}

	return kos, nil
}
