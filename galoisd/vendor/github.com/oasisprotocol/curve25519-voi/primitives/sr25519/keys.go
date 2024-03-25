// Copyright (c) 2019 isis agora lovecruft. All rights reserved.
// Copyright (c) 2019 Web 3 Foundation. All rights reserved.
// Copyright (c) 2021 Oasis Labs Inc. All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
// 1. Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright
// notice, this list of conditions and the following disclaimer in the
// documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS
// IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED
// TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A
// PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED
// TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

package sr25519

import (
	"crypto/rand"
	"crypto/sha512"
	"crypto/subtle"
	"fmt"
	"io"

	"github.com/oasisprotocol/curve25519-voi/curve"
	"github.com/oasisprotocol/curve25519-voi/curve/scalar"
	"github.com/oasisprotocol/curve25519-voi/primitives/merlin"
)

const (
	// MiniSecretKeySize is the size of a MiniSecretKey in bytes.
	MiniSecretKeySize = 32

	// SecretKeyScalarSize is the size of the scalar component of a
	// SecretKey in bytes.
	SecretKeyScalarSize = scalar.ScalarSize

	// SecretKeyNonceSize is the size of the nonce component of a
	// SecretKey in bytes.
	SecretKeyNonceSize = 32

	// SecretKeySize is the size of a SecretKey in bytes.
	SecretKeySize = SecretKeyScalarSize + SecretKeyNonceSize

	// PublicKeySize is the size of a PublicKey in bytes.
	PublicKeySize = curve.CompressedPointSize

	// KeyPairSize is the size of a KeyPair in bytes.
	KeyPairSize = SecretKeySize + PublicKeySize
)

func scalarDivideByCofactor(b []byte) (*scalar.Scalar, error) {
	var (
		scalarBytes [scalar.ScalarSize]byte
		low         byte
	)
	for i := scalar.ScalarSize - 1; i >= 0; i-- {
		v := b[i]
		r := v & 7 // save remainder
		v = v >> 3 // divide by 8
		scalarBytes[i] = v + low
		low = r << 5
	}

	return scalar.NewFromBits(scalarBytes[:])
}

// MiniSecretKey is an EdDSA-like seed, from which the expanded secret
// key is generated.
type MiniSecretKey [MiniSecretKeySize]byte

// MarshalBinary encodes a MiniSecretKey into binary form.
func (msk *MiniSecretKey) MarshalBinary() ([]byte, error) {
	return append([]byte{}, msk[:]...), nil
}

// UnmarshalBinary decodes a binary marshaled MiniSecretKey.
func (msk *MiniSecretKey) UnmarshalBinary(data []byte) error {
	if l := len(data); l != MiniSecretKeySize {
		return fmt.Errorf("sr25519: bad MiniSecretKey size: %v", l)
	}

	copy(msk[:], data)

	return nil
}

// Equal reports if msk and other have the same value.  This function
// will execute in constant time.
func (msk *MiniSecretKey) Equal(other *MiniSecretKey) bool {
	return subtle.ConstantTimeCompare(msk[:], other[:]) == 1
}

// ExpandUniform expands a MiniSecretKey into a SecretKey using merlin.
func (msk *MiniSecretKey) ExpandUniform() *SecretKey {
	t := merlin.NewTranscript("ExpandSecretKeys")
	t.AppendMessage("mini", msk[:])

	var scalarBytes [scalar.ScalarWideSize]byte
	t.ExtractBytes(scalarBytes[:], "sk")
	keyScalar, err := scalar.NewFromBytesModOrderWide(scalarBytes[:])
	if err != nil {
		panic("sr25519: scalar.NewFromBytesModOrderWide: " + err.Error())
	}

	sk := &SecretKey{
		key: keyScalar,
	}
	t.ExtractBytes(sk.nonce[:], "no")

	return sk
}

// ExpandEd25519 expands a MiniSecretKey into a SecretKey using
// Ed25519-style bit clamping.
//
// Note: Unless there is a specific reason to do so (eg: compatibility),
// the use of this method is discouraged.
func (msk *MiniSecretKey) ExpandEd25519() *SecretKey {
	digest := sha512.Sum512(msk[:])
	digest[0] &= 248
	digest[31] &= 63
	digest[31] |= 64

	keyScalar, err := scalarDivideByCofactor(digest[:32])
	if err != nil {
		panic("sr25519: failed to deserialize key scalar: " + err.Error())
	}

	sk := &SecretKey{
		key: keyScalar,
	}
	copy(sk.nonce[:], digest[32:])

	return sk
}

// NewMiniSecretKeyFromBytes constructs a MiniSecretKey from the byte
// representation.
func NewMiniSecretKeyFromBytes(b []byte) (*MiniSecretKey, error) {
	var msk MiniSecretKey
	if err := msk.UnmarshalBinary(b); err != nil {
		return nil, err
	}
	return &msk, nil
}

// GenerateMiniSecretKey generates a MiniSecretKey using entropy from rng.
// If rng is nil, crypto/rand.Reader will be used.
func GenerateMiniSecretKey(rng io.Reader) (*MiniSecretKey, error) {
	if rng == nil {
		rng = rand.Reader
	}

	var msk MiniSecretKey
	if _, err := io.ReadFull(rng, msk[:]); err != nil {
		return nil, fmt.Errorf("sr25519: failed to read entropy: %w", err)
	}

	return &msk, nil
}

// SecretKey is an expanded secret key.
type SecretKey struct {
	key   *scalar.Scalar
	nonce [SecretKeyNonceSize]byte
}

// MarshalBinary encodes a SecretKey into binary form.
func (sk *SecretKey) MarshalBinary() ([]byte, error) {
	b := make([]byte, SecretKeyScalarSize, SecretKeySize)

	if sk.key != nil {
		if err := sk.key.ToBytes(b[:SecretKeyScalarSize]); err != nil {
			return nil, fmt.Errorf("sr25519: failed to serialize key scalar: %w", err)
		}
	}

	b = append(b, sk.nonce[:]...)

	return b, nil
}

// UnmarshalBinary decodes a binary marshaled SecretKey.
func (sk *SecretKey) UnmarshalBinary(data []byte) error {
	if l := len(data); l != SecretKeySize {
		return fmt.Errorf("sr25519: bad SecretKey size: %v", l)
	}

	keyScalar, err := scalar.NewFromCanonicalBytes(data[:SecretKeyScalarSize])
	if err != nil {
		return fmt.Errorf("sr25519: failed to deserialize key scalar: %w", err)
	}

	sk.key = keyScalar
	copy(sk.nonce[:], data[32:])

	return nil
}

// PublicKey derives the public key corresponding to the SecretKey.
func (sk *SecretKey) PublicKey() *PublicKey {
	if sk.key == nil {
		panic("sr25519: attempted to derive public key from uninitialized SecretKey")
	}

	var A curve.RistrettoPoint
	A.MulBasepoint(curve.RISTRETTO_BASEPOINT_TABLE, sk.key)

	return newPublicKeyFromPoint(&A)
}

// KeyPair returns the key pair corresponding to the SecretKey.
func (sk *SecretKey) KeyPair() *KeyPair {
	return &KeyPair{
		sk: sk,
		pk: sk.PublicKey(),
	}
}

// Equal reports if sk and other have the same value, where equality checks
// both the scalar and the nonce components.  This function will execute
// in constant time.
func (sk *SecretKey) Equal(other *SecretKey) bool {
	cmp := sk.key.Equal(other.key)
	cmp = cmp & subtle.ConstantTimeCompare(sk.nonce[:], other.nonce[:])
	return cmp == 1
}

// NewSecretKeyFromBytes constructs a SecretKey from the byte representation.
func NewSecretKeyFromBytes(b []byte) (*SecretKey, error) {
	var sk SecretKey
	if err := sk.UnmarshalBinary(b); err != nil {
		return nil, err
	}
	return &sk, nil
}

// NewSecretKeyFromEd25519Bytes constructs a SecretKey from the byte representation
// of an expanded Ed25519 key.
func NewSecretKeyFromEd25519Bytes(b []byte) (*SecretKey, error) {
	if len(b) != SecretKeySize {
		return nil, fmt.Errorf("sr25519: invalid expanded ed25519 key length")
	}

	// Check the scalar is clamped.
	s := b[:SecretKeyScalarSize]
	if (s[0]&0b0000_0111) != 0 || (s[31]&0b1100_0000) != 0b0100_0000 {
		return nil, fmt.Errorf("sr25519: ed25519 scalar invalid")
	}

	scalar, err := scalarDivideByCofactor(s)
	if err != nil {
		return nil, err
	}

	var nonce [SecretKeyNonceSize]byte
	copy(nonce[:], b[SecretKeyScalarSize:])

	sk := &SecretKey{
		key:   scalar,
		nonce: nonce,
	}
	return sk, nil
}

// GenerateSecretKey generates a SecretKey using entropy from rng.
// If rng is nil, crypto/rand.Reader will be used.
func GenerateSecretKey(rng io.Reader) (*SecretKey, error) {
	if rng == nil {
		rng = rand.Reader
	}

	sk := &SecretKey{
		key: scalar.New(),
	}
	if _, err := sk.key.SetRandom(rng); err != nil {
		return nil, fmt.Errorf("sr25519: failed to generate random scalar: %w", err)
	}

	if _, err := io.ReadFull(rng, sk.nonce[:]); err != nil {
		return nil, fmt.Errorf("sr25519: failed to generate random nonce: %w", err)
	}

	return sk, nil
}

// PublicKey is a public key.
type PublicKey struct {
	compressed curve.CompressedRistretto
	point      *curve.RistrettoPoint
}

// MarshalBinary encodes a PublicKey into binary form.
func (pk *PublicKey) MarshalBinary() ([]byte, error) {
	switch pk.point {
	case nil:
		// Uninitialized, this could return an error, but an all 0
		// CompressedRistretto is the identity element, so it is
		// "fine".
		return make([]byte, PublicKeySize), nil
	default:
		return append([]byte{}, pk.compressed[:]...), nil
	}
}

// UnmarshalBinary decodes a binary marshaled PublicKey.
func (pk *PublicKey) UnmarshalBinary(data []byte) error {
	pk.compressed.Identity()
	pk.point = nil

	if l := len(data); l != PublicKeySize {
		return fmt.Errorf("sr25519: bad PublicKey size: %v", l)
	}

	var compressedA curve.CompressedRistretto
	if err := compressedA.UnmarshalBinary(data); err != nil {
		return fmt.Errorf("sr25519: failed to deserialize public key: %w", err)
	}

	var A curve.RistrettoPoint
	if _, err := A.SetCompressed(&compressedA); err != nil {
		return fmt.Errorf("sr25519: failed to decompress public key: %w", err)
	}

	pk.compressed = compressedA
	pk.point = &A

	return nil
}

// Equal reports if pk and other have the same value.  This function will
// execute in constant time.
func (pk *PublicKey) Equal(other *PublicKey) bool {
	return pk.compressed.Equal(&other.compressed) == 1
}

// NewPublicKeyFromBytes constructs a PublicKey from the byte representation.
func NewPublicKeyFromBytes(b []byte) (*PublicKey, error) {
	var pk PublicKey
	if err := pk.UnmarshalBinary(b); err != nil {
		return nil, err
	}
	return &pk, nil
}

func newPublicKeyFromPoint(point *curve.RistrettoPoint) *PublicKey {
	var pk PublicKey
	pk.compressed.SetRistrettoPoint(point)
	pk.point = curve.NewRistrettoPoint().Set(point)

	return &pk
}

// KeyPair encapsulates a SecretKey and PublicKey.
type KeyPair struct {
	sk *SecretKey
	pk *PublicKey
}

// MarshalBinary encodes a KeyPair into binary form.
func (kp *KeyPair) MarshalBinary() ([]byte, error) {
	// Uninitialized keypair, assume the default value of the secret key.
	if kp.sk == nil || kp.pk == nil {
		return make([]byte, KeyPairSize), nil
	}

	skBytes, err := kp.sk.MarshalBinary()
	if err != nil {
		return nil, err
	}

	pkBytes, err := kp.pk.MarshalBinary()
	if err != nil {
		return nil, err
	}

	b := make([]byte, 0, KeyPairSize)
	b = append(b, skBytes...)
	b = append(b, pkBytes...)

	return b, nil
}

// UnmarshalBinary decodes a binary marshaled KeyPair.
func (kp *KeyPair) UnmarshalBinary(data []byte) error {
	kp.sk = nil
	kp.pk = nil

	if l := len(data); l != KeyPairSize {
		return fmt.Errorf("sr25519: bad KeyPair size: %v", l)
	}

	var sk SecretKey
	if err := sk.UnmarshalBinary(data[:SecretKeySize]); err != nil {
		return err
	}

	var pk PublicKey
	if err := pk.UnmarshalBinary(data[SecretKeySize:]); err != nil {
		return err
	}

	if !sk.PublicKey().Equal(&pk) {
		return fmt.Errorf("sr25519: bad KeyPair, public key mismatch")
	}

	kp.sk = &sk
	kp.pk = &pk

	return nil
}

// SecretKey returns the secret key component of the KeyPair.
func (kp *KeyPair) SecretKey() *SecretKey {
	return kp.sk
}

// PublicKey returns the public key component of the KeyPair.
func (kp *KeyPair) PublicKey() *PublicKey {
	return kp.pk
}

// NewKeyPairFromBytes constructs a KeyPair from the byte representation.
func NewKeyPairFromBytes(b []byte) (*KeyPair, error) {
	var kp KeyPair
	if err := kp.UnmarshalBinary(b); err != nil {
		return nil, err
	}
	return &kp, nil
}

// GenerateKeyPair generates a KeyPair using entropy from rng.
// If rng is nil, crypto/rand.Reader will be used.
func GenerateKeyPair(rng io.Reader) (*KeyPair, error) {
	sk, err := GenerateSecretKey(rng)
	if err != nil {
		return nil, err
	}
	return sk.KeyPair(), nil
}
