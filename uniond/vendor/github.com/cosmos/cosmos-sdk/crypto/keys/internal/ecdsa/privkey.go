package ecdsa

import (
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/rand"
	"crypto/sha256"
	"fmt"
	"math/big"
)

// p256Order returns the curve order for the secp256r1 curve
// NOTE: this is specific to the secp256r1/P256 curve,
// and not taken from the domain params for the key itself
// (which would be a more generic approach for all EC).
var p256Order = elliptic.P256().Params().N

// p256HalfOrder returns half the curve order
// a bit shift of 1 to the right (Rsh) is equivalent
// to division by 2, only faster.
var p256HalfOrder = new(big.Int).Rsh(p256Order, 1)

// IsSNormalized returns true for the integer sigS if sigS falls in
// lower half of the curve order
func IsSNormalized(sigS *big.Int) bool {
	return sigS.Cmp(p256HalfOrder) != 1
}

// NormalizeS will invert the s value if not already in the lower half
// of curve order value
func NormalizeS(sigS *big.Int) *big.Int {
	if IsSNormalized(sigS) {
		return sigS
	}

	return new(big.Int).Sub(p256Order, sigS)
}

// signatureRaw will serialize signature to R || S.
// R, S are padded to 32 bytes respectively.
// code roughly copied from secp256k1_nocgo.go
func signatureRaw(r *big.Int, s *big.Int) []byte {
	rBytes := r.Bytes()
	sBytes := s.Bytes()
	sigBytes := make([]byte, 64)
	// 0 pad the byte arrays from the left if they aren't big enough.
	copy(sigBytes[32-len(rBytes):32], rBytes)
	copy(sigBytes[64-len(sBytes):64], sBytes)
	return sigBytes
}

// GenPrivKey generates a new secp256r1 private key. It uses operating
// system randomness.
func GenPrivKey(curve elliptic.Curve) (PrivKey, error) {
	key, err := ecdsa.GenerateKey(curve, rand.Reader)
	if err != nil {
		return PrivKey{}, err
	}
	return PrivKey{*key}, nil
}

type PrivKey struct {
	ecdsa.PrivateKey
}

// PubKey returns ECDSA public key associated with this private key.
func (sk *PrivKey) PubKey() PubKey {
	return PubKey{sk.PublicKey, nil}
}

// Bytes serialize the private key using big-endian.
func (sk *PrivKey) Bytes() []byte {
	if sk == nil {
		return nil
	}
	fieldSize := (sk.Curve.Params().BitSize + 7) / 8
	bz := make([]byte, fieldSize)
	sk.D.FillBytes(bz)
	return bz
}

// Sign hashes and signs the message using ECDSA. Implements SDK
// PrivKey interface.
// NOTE: this now calls the ecdsa Sign function
// (not method!) directly as the s value of the signature is needed to
// low-s normalize the signature value
// See issue: https://github.com/cosmos/cosmos-sdk/issues/9723
// It then raw encodes the signature as two fixed width 32-byte values
// concatenated, reusing the code copied from secp256k1_nocgo.go
func (sk *PrivKey) Sign(msg []byte) ([]byte, error) {
	digest := sha256.Sum256(msg)
	r, s, err := ecdsa.Sign(rand.Reader, &sk.PrivateKey, digest[:])
	if err != nil {
		return nil, err
	}

	normS := NormalizeS(s)
	return signatureRaw(r, normS), nil
}

// String returns a string representation of the public key based on the curveName.
func (sk *PrivKey) String(name string) string {
	return name + "{-}"
}

// MarshalTo implements proto.Marshaler interface.
func (sk *PrivKey) MarshalTo(dAtA []byte) (int, error) {
	bz := sk.Bytes()
	copy(dAtA, bz)
	return len(bz), nil
}

// Unmarshal implements proto.Marshaler interface.
func (sk *PrivKey) Unmarshal(bz []byte, curve elliptic.Curve, expectedSize int) error {
	if len(bz) != expectedSize {
		return fmt.Errorf("wrong ECDSA SK bytes, expecting %d bytes", expectedSize)
	}

	sk.Curve = curve
	sk.D = new(big.Int).SetBytes(bz)
	sk.X, sk.Y = curve.ScalarBaseMult(bz)
	return nil
}
