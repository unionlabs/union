//go:build !bls12381

package bls12_381

import (
	"bytes"
	"errors"
	"fmt"

	"github.com/cometbft/cometbft/crypto"

	"github.com/cosmos/cosmos-sdk/codec"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
)

// ===============================================================================================
// Private Key
// ===============================================================================================

// PrivKey is a wrapper around the Ethereum BLS12-381 private key type. This
// wrapper conforms to crypto.Pubkey to allow for the use of the Ethereum
// BLS12-381 private key type.

var (
	_ cryptotypes.PrivKey  = &PrivKey{}
	_ codec.AminoMarshaler = &PrivKey{}
)

// NewPrivateKeyFromBytes build a new key from the given bytes.
func NewPrivateKeyFromBytes(bz []byte) (PrivKey, error) {
	panic("not implemented, build flags are required to use bls12_381 keys")
}

// GenPrivKey generates a new key.
func GenPrivKey() (PrivKey, error) {
	panic("not implemented, build flags are required to use bls12_381 keys")
}

// Bytes returns the byte representation of the Key.
func (privKey PrivKey) Bytes() []byte {
	panic("not implemented, build flags are required to use bls12_381 keys")
}

// PubKey returns the private key's public key. If the privkey is not valid
// it returns a nil value.
func (privKey PrivKey) PubKey() cryptotypes.PubKey {
	panic("not implemented, build flags are required to use bls12_381 keys")
}

// Equals returns true if two keys are equal and false otherwise.
func (privKey PrivKey) Equals(other cryptotypes.LedgerPrivKey) bool {
	panic("not implemented, build flags are required to use bls12_381 keys")
}

// Type returns the type.
func (PrivKey) Type() string {
	return KeyType
}

// Sign signs the given byte array. If msg is larger than
// MaxMsgLen, SHA256 sum will be signed instead of the raw bytes.
func (privKey PrivKey) Sign(msg []byte) ([]byte, error) {
	panic("not implemented, build flags are required to use bls12_381 keys")
}

// MarshalAmino overrides Amino binary marshaling.
func (privKey PrivKey) MarshalAmino() ([]byte, error) {
	return privKey.Key, nil
}

// UnmarshalAmino overrides Amino binary marshaling.
func (privKey *PrivKey) UnmarshalAmino(bz []byte) error {
	if len(bz) != PrivKeySize {
		return errors.New("invalid privkey size")
	}
	privKey.Key = bz

	return nil
}

// MarshalAminoJSON overrides Amino JSON marshaling.
func (privKey PrivKey) MarshalAminoJSON() ([]byte, error) {
	// When we marshal to Amino JSON, we don't marshal the "key" field itself,
	// just its contents (i.e. the key bytes).
	return privKey.MarshalAmino()
}

// UnmarshalAminoJSON overrides Amino JSON marshaling.
func (privKey *PrivKey) UnmarshalAminoJSON(bz []byte) error {
	return privKey.UnmarshalAmino(bz)
}

// ===============================================================================================
// Public Key
// ===============================================================================================

// Pubkey is a wrapper around the Ethereum BLS12-381 public key type. This
// wrapper conforms to crypto.Pubkey to allow for the use of the Ethereum
// BLS12-381 public key type.

var _ cryptotypes.PubKey = &PubKey{}

// Address returns the address of the key.
//
// The function will panic if the public key is invalid.
func (pubKey PubKey) Address() crypto.Address {
	panic("not implemented, build flags are required to use bls12_381 keys")
}

// VerifySignature verifies the given signature.
func (pubKey PubKey) VerifySignature(msg, sig []byte) bool {
	panic("not implemented, build flags are required to use bls12_381 keys")
}

// Bytes returns the byte format.
func (pubKey PubKey) Bytes() []byte {
	return pubKey.Key
}

// Type returns the key's type.
func (PubKey) Type() string {
	return KeyType
}

// Equals returns true if the other's type is the same and their bytes are deeply equal.
func (pubKey PubKey) Equals(other cryptotypes.PubKey) bool {
	return pubKey.Type() == other.Type() && bytes.Equal(pubKey.Bytes(), other.Bytes())
}

// String returns Hex representation of a pubkey with it's type
func (pubKey PubKey) String() string {
	return fmt.Sprintf("PubKeyBLS12_381{%X}", pubKey.Key)
}
