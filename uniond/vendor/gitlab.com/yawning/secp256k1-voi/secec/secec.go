// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

// Package secec implements the common primitives on top of secp256k1,
// with an API that is close to the runtime library's `crypto/ecdsa`
// and `crypto/ecdh` packages.
package secec

import (
	"bytes"
	"crypto"
	"crypto/rand"
	"crypto/subtle"
	"errors"
	"fmt"

	"gitlab.com/yawning/secp256k1-voi"
	"gitlab.com/yawning/secp256k1-voi/internal/disalloweq"
)

// PrivateKeySize is the size of a secp256k1 private key in bytes.
const PrivateKeySize = 32

var (
	errAIsInfinity       = errors.New("secp256k1/secec: public key is the point at infinity")
	errAIsUninitialized  = errors.New("secp256k1/secec: uninitialized public key")
	errInvalidPrivateKey = errors.New("secp256k1/secec: invalid private key")
)

// PrivateKey is a secp256k1 private key.
type PrivateKey struct {
	_ disalloweq.DisallowEqual

	scalar    *secp256k1.Scalar // INVARIANT: Always [1,n)
	publicKey *PublicKey
}

// Bytes returns a copy of the encoding of the private key.
func (k *PrivateKey) Bytes() []byte {
	return k.scalar.Bytes()
}

// Scalar returns a copy of the scalar underlying `k`.
func (k *PrivateKey) Scalar() *secp256k1.Scalar {
	return secp256k1.NewScalarFrom(k.scalar)
}

// ECDH performs a ECDH exchange and returns the shared secret as
// specified in SEC 1, Version 2.0, Section 3.3.1, and returns the
// x-coordinate encoded according to SEC 1, Version 2.0, Section 2.3.5.
// The result is never the point at infinity.
func (k *PrivateKey) ECDH(remote *PublicKey) ([]byte, error) {
	pt := secp256k1.NewIdentityPoint().ScalarMult(k.scalar, remote.point)
	return pt.XBytes()
}

// Equal returns whether `x` represents the same private key as `k`.
// This check is performed in constant time as long as the key types
// match.
func (k *PrivateKey) Equal(x crypto.PrivateKey) bool {
	other, ok := x.(*PrivateKey)
	if !ok {
		return false
	}

	return other.scalar.Equal(k.scalar) == 1
}

func (k *PrivateKey) Public() crypto.PublicKey {
	return k.publicKey
}

// PublicKey returns the ECDSA/ECDH public key corresponding to `k`.
func (k *PrivateKey) PublicKey() *PublicKey {
	return k.publicKey
}

// PublicKey is a secp256k1 public key.
type PublicKey struct {
	_ disalloweq.DisallowEqual

	point      *secp256k1.Point // INVARIANT: Never identity
	pointBytes []byte           // Uncompressed SEC 1 encoding
}

// Bytes returns a copy of the uncompressed encoding of the public key.
func (k *PublicKey) Bytes() []byte {
	if k.pointBytes == nil {
		panic(errAIsUninitialized)
	}

	return bytes.Clone(k.pointBytes)
}

// CompressedBytes returns a copy of the compressed encoding of the public
// key.
func (k *PublicKey) CompressedBytes() []byte {
	xBytes, yIsOdd := secp256k1.SplitUncompressedPoint(k.pointBytes)
	buf := make([]byte, 0, secp256k1.CompressedPointSize)
	buf = append(buf, byte(yIsOdd)+0x02) // 0x02 -> even, 0x03 -> odd
	buf = append(buf, xBytes...)

	return buf
}

// ASN1Bytes returns a copy of the ASN.1 encoding of the public key,
// as specified in SEC 1, Version 2.0, Appendix C.3.
func (k *PublicKey) ASN1Bytes() []byte {
	return buildASN1PublicKey(k)
}

// Point returns a copy of the point underlying `k`.
func (k *PublicKey) Point() *secp256k1.Point {
	return secp256k1.NewPointFrom(k.point)
}

// Equal returns whether `x` represents the same public key as `k`.
// This check is performed in constant time as long as the key types
// match.
func (k *PublicKey) Equal(x crypto.PublicKey) bool {
	other, ok := x.(*PublicKey)
	if !ok {
		return false
	}

	// Comparing the serialized form is faster than comparing points.
	return subtle.ConstantTimeCompare(k.pointBytes, other.pointBytes) == 1
}

// GenerateKey generates a new PrivateKey, using [crypto/rand.Reader]
// as the entropy source.
func GenerateKey() (*PrivateKey, error) {
	s, err := sampleRandomScalar(rand.Reader)
	if err != nil {
		return nil, err
	}

	return NewPrivateKeyFromScalar(s)
}

// NewPrivateKey checks that `key` is valid and returns a PrivateKey.
//
// This follows SEC 1, Version 2.0, Section 2.3.6, which amounts to
// decoding the bytes as a fixed length big endian integer and checking
// that the result is lower than the order of the curve. The zero
// private key is also rejected, as the encoding of the corresponding
// public key would be irregular.
func NewPrivateKey(key []byte) (*PrivateKey, error) {
	if len(key) != PrivateKeySize {
		return nil, errInvalidPrivateKey
	}

	s, didReduce := secp256k1.NewScalarFromBytes((*[secp256k1.ScalarSize]byte)(key))
	if didReduce != 0 {
		return nil, errInvalidPrivateKey
	}

	return newPrivateKeyFromScalar(s)
}

// NewPrivateKeyFromScalar checks that `s` is valid and returns a
// PrivateKey.
func NewPrivateKeyFromScalar(s *secp256k1.Scalar) (*PrivateKey, error) {
	return newPrivateKeyFromScalar(secp256k1.NewScalarFrom(s))
}

func newPrivateKeyFromScalar(s *secp256k1.Scalar) (*PrivateKey, error) {
	if s.IsZero() != 0 {
		return nil, errInvalidPrivateKey
	}

	// Note: Caller ensures that s is in the correct range.
	pt := secp256k1.NewIdentityPoint().ScalarBaseMult(s)
	publicKey, _ := newPublicKeyFromPoint(pt) // Can't fail, pt can NEVER be inf
	privateKey := &PrivateKey{
		scalar:    s,
		publicKey: publicKey,
	}

	return privateKey, nil
}

// NewPublicKey checks that `key` is valid and returns a PublicKey.
//
// This decodes an encoded point according to SEC 1, Version 2.0,
// Section 2.3.4. The point at infinity is rejected.
func NewPublicKey(key []byte) (*PublicKey, error) {
	// Note: crypto/ecdsa's version ONLY supports uncompressed points
	// but way too much of the shitcoin ecosystem supports compressed,
	// so might as well support all the formats, and explicitly just
	// reject the identity encoding.
	pt, err := secp256k1.NewPointFromBytes(key)
	if err != nil {
		return nil, fmt.Errorf("secp256k1/secec: invalid public key: %w", err)
	}

	return newPublicKeyFromPoint(pt)
}

// NewPublicKeyFromPoint checks that `point` is valid, and returns a PublicKey.
func NewPublicKeyFromPoint(point *secp256k1.Point) (*PublicKey, error) {
	return newPublicKeyFromPoint(secp256k1.NewPointFrom(point))
}

func newPublicKeyFromPoint(pt *secp256k1.Point) (*PublicKey, error) {
	if pt.IsIdentity() != 0 {
		return nil, errAIsInfinity
	}

	// Note: Caller ensures that pt is on the curve.
	return &PublicKey{
		point:      pt,
		pointBytes: pt.UncompressedBytes(),
	}, nil
}
