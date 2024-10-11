package bn254

import (
	"crypto/sha256"
	"crypto/sha512"
	"crypto/subtle"
	"fmt"

	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/bn254"

	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/codec"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/types/errors"
)

//-------------------------------------

const (
	PrivKeyName = bn254.PrivKeyName
	PubKeyName  = bn254.PubKeyName
	PubKeySize  = bn254.PubKeySize
	PrivKeySize = bn254.PrivKeySize
	KeyType     = bn254.KeyType
)

var (
	_ cryptotypes.PrivKey  = &PrivKey{}
	_ codec.AminoMarshaler = &PrivKey{}
)

// Bytes returns the privkey byte format.
func (privKey *PrivKey) Bytes() []byte {
	return privKey.Key.Bytes()
}

// Sign produces a signature on the provided message.
// This assumes the privkey is wellformed in the golang format.
// The first 32 bytes should be random,
// corresponding to the normal ed25519 private key.
// The latter 32 bytes should be the compressed public key.
// If these conditions aren't met, Sign will panic or produce an
// incorrect signature.
func (privKey *PrivKey) Sign(msg []byte) ([]byte, error) {
	return privKey.Key.Sign(msg)
}

func GenPrivKey() *PrivKey {
	return &PrivKey{Key: bn254.GenPrivKey()}
}

func GenPrivKeyFromSecret(secret []byte) *PrivKey {
	hasher := sha512.New()
	hasher.Write(secret)
	seed := hasher.Sum(nil)
	return &PrivKey{Key: bn254.GenPrivKeyFromSeed(seed)}
}

// PubKey gets the corresponding public key from the private key.
//
// Panics if the private key is not initialized.
func (privKey *PrivKey) PubKey() cryptotypes.PubKey {
	pk := privKey.Key.PubKey().Bytes()
	return &PubKey{Key: pk}
}

// Equals - you probably don't need to use this.
// Runs in constant time based on length of the keys.
func (privKey *PrivKey) Equals(other cryptotypes.LedgerPrivKey) bool {
	if privKey.Type() != other.Type() {
		return false
	}
	return subtle.ConstantTimeCompare(privKey.Bytes(), other.Bytes()) == 1
}

func (privKey *PrivKey) Type() string {
	return privKey.Key.Type()
}

// MarshalAmino overrides Amino binary marshalling.
func (privKey PrivKey) MarshalAmino() ([]byte, error) {
	return privKey.Key, nil
}

// UnmarshalAmino overrides Amino binary marshalling.
func (privKey *PrivKey) UnmarshalAmino(bz []byte) error {
	if len(bz) != PrivKeySize {
		return fmt.Errorf("invalid privkey size")
	}
	privKey.Key = bz

	return nil
}

// MarshalAminoJSON overrides Amino JSON marshalling.
func (privKey PrivKey) MarshalAminoJSON() ([]byte, error) {
	// When we marshal to Amino JSON, we don't marshal the "key" field itself,
	// just its contents (i.e. the key bytes).
	return privKey.MarshalAmino()
}

// UnmarshalAminoJSON overrides Amino JSON marshalling.
func (privKey *PrivKey) UnmarshalAminoJSON(bz []byte) error {
	return privKey.UnmarshalAmino(bz)
}

//-------------------------------------

var (
	_ cryptotypes.PubKey   = &PubKey{}
	_ codec.AminoMarshaler = &PubKey{}
)

const TruncatedSize = 20

// Address is the SHA256-20 of the raw pubkey bytes.
// It doesn't implement ADR-28 addresses and it must not be used
// in SDK except in a tendermint validator context.
func (pubKey *PubKey) Address() crypto.Address {
	if len(pubKey.Key) != PubKeySize {
		panic("pubkey is incorrect size")
	}
	var pKey bn254.PubKey = pubKey.Bytes()
	if err := pKey.EnsureValid(); err != nil {
		panic(err)
	}
	// For ADR-28 compatible address we would need to
	// return address.Hash(proto.MessageName(pubKey), pubKey.Key)
	hash := sha256.Sum256(pubKey.Key.Bytes())
	return crypto.Address(hash[:TruncatedSize])
}

// Bytes returns the PubKey byte format.
func (pubKey *PubKey) Bytes() []byte {
	return pubKey.Key.Bytes()
}

func (pubKey *PubKey) VerifySignature(msg []byte, sig []byte) bool {
	return pubKey.Key.VerifySignature(msg, sig)
}

// String returns Hex representation of a pubkey with it's type
func (pubKey *PubKey) String() string {
	return fmt.Sprintf("PubKeyBn256{%X}", pubKey.Key)
}

func (pubKey *PubKey) Type() string {
	return pubKey.Key.Type()
}

func (pubKey *PubKey) Equals(other cryptotypes.PubKey) bool {
	if pubKey.Type() != other.Type() {
		return false
	}
	return subtle.ConstantTimeCompare(pubKey.Bytes(), other.Bytes()) == 1
}

// MarshalAmino overrides Amino binary marshalling.
func (pubKey PubKey) MarshalAmino() ([]byte, error) {
	return pubKey.Key.Bytes(), nil
}

// UnmarshalAmino overrides Amino binary marshalling.
func (pubKey *PubKey) UnmarshalAmino(bz []byte) error {
	if len(bz) != PubKeySize {
		return errorsmod.Wrap(errors.ErrInvalidPubKey, "invalid pubkey size")
	}
	var pKey bn254.PubKey = bz
	if err := pKey.EnsureValid(); err != nil {
		panic(err)
	}
	pubKey.Key = bz

	return nil
}

// MarshalAminoJSON overrides Amino JSON marshalling.
func (pubKey PubKey) MarshalAminoJSON() ([]byte, error) {
	// When we marshal to Amino JSON, we don't marshal the "key" field itself,
	// just its contents (i.e. the key bytes).
	return pubKey.MarshalAmino()
}

// UnmarshalAminoJSON overrides Amino JSON marshalling.
func (pubKey *PubKey) UnmarshalAminoJSON(bz []byte) error {
	return pubKey.UnmarshalAmino(bz)
}
