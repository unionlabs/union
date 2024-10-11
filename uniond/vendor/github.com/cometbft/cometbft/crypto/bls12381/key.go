//go:build !bls12381

package bls12381

import (
	"errors"

	"github.com/cometbft/cometbft/crypto"
)

const (
	// Enabled indicates if this curve is enabled.
	Enabled = false
)

// ErrDisabled is returned if the caller didn't use the `bls12381` build tag or has an incompatible OS.
var ErrDisabled = errors.New("bls12_381 is disabled")

// ===============================================================================================
// Private Key
// ===============================================================================================

// PrivKey is a wrapper around the Ethereum BLS12-381 private key type. This
// wrapper conforms to crypto.Pubkey to allow for the use of the Ethereum
// BLS12-381 private key type.

// Compile-time type assertion.
var _ crypto.PrivKey = &PrivKey{}

// PrivKey represents a BLS private key noop when blst is not set as a build flag and cgo is disabled.
type PrivKey []byte

// NewPrivateKeyFromBytes returns ErrDisabled.
func NewPrivateKeyFromBytes([]byte) (PrivKey, error) {
	return nil, ErrDisabled
}

// GenPrivKey returns ErrDisabled.
func GenPrivKey() (PrivKey, error) {
	return nil, ErrDisabled
}

// Bytes returns the byte representation of the Key.
func (privKey PrivKey) Bytes() []byte {
	return privKey
}

// PubKey always panics.
func (PrivKey) PubKey() crypto.PubKey {
	panic("bls12_381 is disabled")
}

// Type returns the key's type.
func (PrivKey) Type() string {
	return KeyType
}

// Sign always panics.
func (PrivKey) Sign([]byte) ([]byte, error) {
	panic("bls12_381 is disabled")
}

// Zeroize always panics.
func (PrivKey) Zeroize() {
	panic("bls12_381 is disabled")
}

// ===============================================================================================
// Public Key
// ===============================================================================================

// Pubkey is a wrapper around the Ethereum BLS12-381 public key type. This
// wrapper conforms to crypto.Pubkey to allow for the use of the Ethereum
// BLS12-381 public key type.

// Compile-time type assertion.
var _ crypto.PubKey = &PubKey{}

// PubKey represents a BLS private key noop when blst is not set as a build flag and cgo is disabled.
type PubKey []byte

// NewPublicKeyFromBytes returns ErrDisabled.
func NewPublicKeyFromBytes([]byte) (*PubKey, error) {
	return nil, ErrDisabled
}

// Address always panics.
func (PubKey) Address() crypto.Address {
	panic("bls12_381 is disabled")
}

// VerifySignature always panics.
func (PubKey) VerifySignature([]byte, []byte) bool {
	panic("bls12_381 is disabled")
}

// Bytes always panics.
func (PubKey) Bytes() []byte {
	panic("bls12_381 is disabled")
}

// Type returns the key's type.
func (PubKey) Type() string {
	return KeyType
}
