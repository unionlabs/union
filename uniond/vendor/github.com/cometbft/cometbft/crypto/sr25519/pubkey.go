package sr25519

import (
	"bytes"
	"fmt"

	"github.com/oasisprotocol/curve25519-voi/primitives/sr25519"

	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/tmhash"
)

var _ crypto.PubKey = PubKey{}

const (
	// PubKeySize is the number of bytes in an Sr25519 public key.
	PubKeySize = 32

	// SignatureSize is the size of a Sr25519 signature in bytes.
	SignatureSize = 64
)

// PubKey implements crypto.PubKey for the Sr25519 signature scheme.
type PubKey []byte

// Address is the SHA256-20 of the raw pubkey bytes.
func (pubKey PubKey) Address() crypto.Address {
	if len(pubKey) != PubKeySize {
		panic("pubkey is incorrect size")
	}
	return crypto.Address(tmhash.SumTruncated(pubKey[:]))
}

// Bytes returns the byte representation of the PubKey.
func (pubKey PubKey) Bytes() []byte {
	return []byte(pubKey)
}

// Equals - checks that two public keys are the same time
// Runs in constant time based on length of the keys.
func (pubKey PubKey) Equals(other crypto.PubKey) bool {
	if otherSr, ok := other.(PubKey); ok {
		return bytes.Equal(pubKey[:], otherSr[:])
	}

	return false
}

func (pubKey PubKey) VerifySignature(msg []byte, sigBytes []byte) bool {
	var srpk sr25519.PublicKey
	if err := srpk.UnmarshalBinary(pubKey); err != nil {
		return false
	}

	var sig sr25519.Signature
	if err := sig.UnmarshalBinary(sigBytes); err != nil {
		return false
	}

	st := signingCtx.NewTranscriptBytes(msg)
	return srpk.Verify(st, &sig)
}

func (pubKey PubKey) String() string {
	return fmt.Sprintf("PubKeySr25519{%X}", []byte(pubKey))
}

func (pubKey PubKey) Type() string {
	return KeyType
}
