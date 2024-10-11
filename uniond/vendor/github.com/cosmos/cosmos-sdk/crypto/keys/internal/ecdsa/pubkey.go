package ecdsa

import (
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/sha256"
	"fmt"
	"math/big"

	cmtcrypto "github.com/cometbft/cometbft/crypto"

	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/types/address"
	"github.com/cosmos/cosmos-sdk/types/errors"
)

// signatureFromBytes function roughly copied from secp256k1_nocgo.go
// Read Signature struct from R || S. Caller needs to ensure that
// len(sigStr) == 64.
func signatureFromBytes(sigStr []byte) *signature {
	return &signature{
		R: new(big.Int).SetBytes(sigStr[:32]),
		S: new(big.Int).SetBytes(sigStr[32:64]),
	}
}

// signature holds the r and s values of an ECDSA signature.
type signature struct {
	R, S *big.Int
}

type PubKey struct {
	ecdsa.PublicKey

	// cache
	address cmtcrypto.Address
}

// Address gets the address associated with a pubkey. If no address exists, it returns a newly created ADR-28 address
// for ECDSA keys.
// protoName is a concrete proto structure id.
func (pk *PubKey) Address(protoName string) cmtcrypto.Address {
	if pk.address == nil {
		pk.address = address.Hash(protoName, pk.Bytes())
	}
	return pk.address
}

// Bytes returns the byte representation of the public key using a compressed form
// specified in section 4.3.6 of ANSI X9.62 with first byte being the curve type.
func (pk *PubKey) Bytes() []byte {
	if pk == nil {
		return nil
	}
	return elliptic.MarshalCompressed(pk.Curve, pk.X, pk.Y)
}

// VerifySignature checks if sig is a valid ECDSA signature for msg.
// This includes checking for low-s normalized signatures
// where the s integer component of the signature is in the
// lower half of the curve order
// 7/21/21 - expects raw encoded signature (fixed-width 64-bytes, R || S)
func (pk *PubKey) VerifySignature(msg, sig []byte) bool {
	// check length for raw signature
	// which is two 32-byte padded big.Ints
	// concatenated
	// NOT DER!

	if len(sig) != 64 {
		return false
	}

	s := signatureFromBytes(sig)
	if !IsSNormalized(s.S) {
		return false
	}

	h := sha256.Sum256(msg)
	return ecdsa.Verify(&pk.PublicKey, h[:], s.R, s.S)
}

// String returns a string representation of the public key based on the curveName.
func (pk *PubKey) String(curveName string) string {
	return fmt.Sprintf("%s{%X}", curveName, pk.Bytes())
}

// **** Proto Marshaler ****

// MarshalTo implements proto.Marshaler interface.
func (pk *PubKey) MarshalTo(dAtA []byte) (int, error) {
	bz := pk.Bytes()
	copy(dAtA, bz)
	return len(bz), nil
}

// Unmarshal implements proto.Marshaler interface.
func (pk *PubKey) Unmarshal(bz []byte, curve elliptic.Curve, expectedSize int) error {
	if len(bz) != expectedSize {
		return errorsmod.Wrapf(errors.ErrInvalidPubKey, "wrong ECDSA PK bytes, expecting %d bytes, got %d", expectedSize, len(bz))
	}
	cpk := ecdsa.PublicKey{Curve: curve}
	cpk.X, cpk.Y = elliptic.UnmarshalCompressed(curve, bz)
	if cpk.X == nil || cpk.Y == nil {
		return errorsmod.Wrapf(errors.ErrInvalidPubKey, "wrong ECDSA PK bytes, unknown curve type: %d", bz[0])
	}
	pk.PublicKey = cpk
	return nil
}
