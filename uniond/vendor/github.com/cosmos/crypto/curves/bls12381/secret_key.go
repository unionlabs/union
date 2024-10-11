package blst

import (
	"crypto/subtle"
	"errors"
	"fmt"

	blst "github.com/supranational/blst/bindings/go"

	"github.com/cosmos/crypto/internal/rand"
)

// bls12SecretKey used in the BLS signature scheme.
type bls12SecretKey struct {
	p *blst.SecretKey
}

// RandKey creates a new private key using a random method provided as an io.Reader.
func RandKey() (SecretKey, error) {
	// Generate 32 bytes of randomness
	var ikm [32]byte
	_, err := rand.NewGenerator().Read(ikm[:])
	if err != nil {
		return nil, err
	}
	return GenPrivKeyFromSeed(ikm)
}

// GenPrivKeyFromSeed creates a new private key directly from the seed passed as parameter
func GenPrivKeyFromSeed(seed [32]byte) (SecretKey, error) {
	// Defensive check, that we have not generated a secret key,
	secKey := &bls12SecretKey{blst.KeyGen(seed[:])}
	if IsZero(secKey.Marshal()) {
		return nil, errors.New("received secret key is zero")
	}
	return secKey, nil
}

// SecretKeyFromBytes creates a BLS private key from a BigEndian byte slice.
func SecretKeyFromBytes(privKey []byte) (SecretKey, error) {
	if len(privKey) != 32 {
		return nil, fmt.Errorf("secret key must be %d bytes", 32)
	}
	if IsZero(privKey) {
		return nil, errors.New("received secret key is zero")
	}
	secKey := new(blst.SecretKey).Deserialize(privKey)
	if secKey == nil {
		return nil, errors.New("could not unmarshal bytes into secret key")
	}
	wrappedKey := &bls12SecretKey{p: secKey}
	return wrappedKey, nil
}

// IsZero checks if the secret key is a zero key.
func IsZero(sKey []byte) bool {
	b := byte(0)
	for _, s := range sKey {
		b |= s
	}
	return subtle.ConstantTimeByteEq(b, 0) == 1
}

func (s *bls12SecretKey) Sign(msg []byte) SignatureI {
	signature := new(blstSignature).Sign(s.p, msg, dst)
	return &Signature{s: signature}
}

// Marshal a secret key into a LittleEndian byte slice.
func (s *bls12SecretKey) Marshal() []byte {
	keyBytes := s.p.Serialize()
	return keyBytes
}

// PublicKey obtains the public key corresponding to the BLS secret key.
func (s *bls12SecretKey) PublicKey() PubKey {
	return &PublicKey{p: new(blstPublicKey).From(s.p)}
}
