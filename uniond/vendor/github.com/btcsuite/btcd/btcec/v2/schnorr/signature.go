// Copyright (c) 2013-2022 The btcsuite developers

package schnorr

import (
	"fmt"

	"github.com/btcsuite/btcd/btcec/v2"
	"github.com/btcsuite/btcd/chaincfg/chainhash"
	secp "github.com/decred/dcrd/dcrec/secp256k1/v4"
	ecdsa_schnorr "github.com/decred/dcrd/dcrec/secp256k1/v4/schnorr"
)

const (
	// SignatureSize is the size of an encoded Schnorr signature.
	SignatureSize = 64

	// scalarSize is the size of an encoded big endian scalar.
	scalarSize = 32
)

var (
	// rfc6979ExtraDataV0 is the extra data to feed to RFC6979 when
	// generating the deterministic nonce for the BIP-340 scheme.  This
	// ensures the same nonce is not generated for the same message and key
	// as for other signing algorithms such as ECDSA.
	//
	// It is equal to SHA-256([]byte("BIP-340")).
	rfc6979ExtraDataV0 = [32]uint8{
		0xa3, 0xeb, 0x4c, 0x18, 0x2f, 0xae, 0x7e, 0xf4,
		0xe8, 0x10, 0xc6, 0xee, 0x13, 0xb0, 0xe9, 0x26,
		0x68, 0x6d, 0x71, 0xe8, 0x7f, 0x39, 0x4f, 0x79,
		0x9c, 0x00, 0xa5, 0x21, 0x03, 0xcb, 0x4e, 0x17,
	}
)

// Signature is a type representing a Schnorr signature.
type Signature struct {
	r btcec.FieldVal
	s btcec.ModNScalar
}

// NewSignature instantiates a new signature given some r and s values.
func NewSignature(r *btcec.FieldVal, s *btcec.ModNScalar) *Signature {
	var sig Signature
	sig.r.Set(r).Normalize()
	sig.s.Set(s)
	return &sig
}

// Serialize returns the Schnorr signature in the more strict format.
//
// The signatures are encoded as
//
//	sig[0:32]  x coordinate of the point R, encoded as a big-endian uint256
//	sig[32:64] s, encoded also as big-endian uint256
func (sig Signature) Serialize() []byte {
	// Total length of returned signature is the length of r and s.
	var b [SignatureSize]byte
	sig.r.PutBytesUnchecked(b[0:32])
	sig.s.PutBytesUnchecked(b[32:64])
	return b[:]
}

// ParseSignature parses a signature according to the BIP-340 specification and
// enforces the following additional restrictions specific to secp256k1:
//
// - The r component must be in the valid range for secp256k1 field elements
// - The s component must be in the valid range for secp256k1 scalars
func ParseSignature(sig []byte) (*Signature, error) {
	// The signature must be the correct length.
	sigLen := len(sig)
	if sigLen < SignatureSize {
		str := fmt.Sprintf("malformed signature: too short: %d < %d", sigLen,
			SignatureSize)
		return nil, signatureError(ecdsa_schnorr.ErrSigTooShort, str)
	}
	if sigLen > SignatureSize {
		str := fmt.Sprintf("malformed signature: too long: %d > %d", sigLen,
			SignatureSize)
		return nil, signatureError(ecdsa_schnorr.ErrSigTooLong, str)
	}

	// The signature is validly encoded at this point, however, enforce
	// additional restrictions to ensure r is in the range [0, p-1], and s is in
	// the range [0, n-1] since valid Schnorr signatures are required to be in
	// that range per spec.
	var r btcec.FieldVal
	if overflow := r.SetByteSlice(sig[0:32]); overflow {
		str := "invalid signature: r >= field prime"
		return nil, signatureError(ecdsa_schnorr.ErrSigRTooBig, str)
	}
	var s btcec.ModNScalar
	s.SetByteSlice(sig[32:64])

	// Return the signature.
	return NewSignature(&r, &s), nil
}

// IsEqual compares this Signature instance to the one passed, returning true
// if both Signatures are equivalent. A signature is equivalent to another, if
// they both have the same scalar value for R and S.
func (sig Signature) IsEqual(otherSig *Signature) bool {
	return sig.r.Equals(&otherSig.r) && sig.s.Equals(&otherSig.s)
}

// schnorrVerify attempt to verify the signature for the provided hash and
// secp256k1 public key and either returns nil if successful or a specific error
// indicating why it failed if not successful.
//
// This differs from the exported Verify method in that it returns a specific
// error to support better testing while the exported method simply returns a
// bool indicating success or failure.
func schnorrVerify(sig *Signature, hash []byte, pubKeyBytes []byte) error {
	// The algorithm for producing a BIP-340 signature is described in
	// README.md and is reproduced here for reference:
	//
	// 1. Fail if m is not 32 bytes
	// 2. P = lift_x(int(pk)).
	// 3. r = int(sig[0:32]); fail is r >= p.
	// 4. s = int(sig[32:64]); fail if s >= n.
	// 5. e = int(tagged_hash("BIP0340/challenge", bytes(r) || bytes(P) || M)) mod n.
	// 6. R = s*G - e*P
	// 7. Fail if is_infinite(R)
	// 8. Fail if not hash_even_y(R)
	// 9. Fail is x(R) != r.
	// 10. Return success iff failure did not occur before reaching this point.

	// Step 1.
	//
	// Fail if m is not 32 bytes
	if len(hash) != scalarSize {
		str := fmt.Sprintf("wrong size for message (got %v, want %v)",
			len(hash), scalarSize)
		return signatureError(ecdsa_schnorr.ErrInvalidHashLen, str)
	}

	// Step 2.
	//
	// P = lift_x(int(pk))
	//
	// Fail if P is not a point on the curve
	pubKey, err := ParsePubKey(pubKeyBytes)
	if err != nil {
		return err
	}
	if !pubKey.IsOnCurve() {
		str := "pubkey point is not on curve"
		return signatureError(ecdsa_schnorr.ErrPubKeyNotOnCurve, str)
	}

	// Step 3.
	//
	// Fail if r >= p
	//
	// Note this is already handled by the fact r is a field element.

	// Step 4.
	//
	// Fail if s >= n
	//
	// Note this is already handled by the fact s is a mod n scalar.

	// Step 5.
	//
	// e = int(tagged_hash("BIP0340/challenge", bytes(r) || bytes(P) || M)) mod n.
	var rBytes [32]byte
	sig.r.PutBytesUnchecked(rBytes[:])
	pBytes := SerializePubKey(pubKey)

	commitment := chainhash.TaggedHash(
		chainhash.TagBIP0340Challenge, rBytes[:], pBytes, hash,
	)

	var e btcec.ModNScalar
	e.SetBytes((*[32]byte)(commitment))

	// Negate e here so we can use AddNonConst below to subtract the s*G
	// point from e*P.
	e.Negate()

	// Step 6.
	//
	// R = s*G - e*P
	var P, R, sG, eP btcec.JacobianPoint
	pubKey.AsJacobian(&P)
	btcec.ScalarBaseMultNonConst(&sig.s, &sG)
	btcec.ScalarMultNonConst(&e, &P, &eP)
	btcec.AddNonConst(&sG, &eP, &R)

	// Step 7.
	//
	// Fail if R is the point at infinity
	if (R.X.IsZero() && R.Y.IsZero()) || R.Z.IsZero() {
		str := "calculated R point is the point at infinity"
		return signatureError(ecdsa_schnorr.ErrSigRNotOnCurve, str)
	}

	// Step 8.
	//
	// Fail if R.y is odd
	//
	// Note that R must be in affine coordinates for this check.
	R.ToAffine()
	if R.Y.IsOdd() {
		str := "calculated R y-value is odd"
		return signatureError(ecdsa_schnorr.ErrSigRYIsOdd, str)
	}

	// Step 9.
	//
	// Verified if R.x == r
	//
	// Note that R must be in affine coordinates for this check.
	if !sig.r.Equals(&R.X) {
		str := "calculated R point was not given R"
		return signatureError(ecdsa_schnorr.ErrUnequalRValues, str)
	}

	// Step 10.
	//
	// Return success iff failure did not occur before reaching this point.
	return nil
}

// Verify returns whether or not the signature is valid for the provided hash
// and secp256k1 public key.
func (sig *Signature) Verify(hash []byte, pubKey *btcec.PublicKey) bool {
	pubkeyBytes := SerializePubKey(pubKey)
	return schnorrVerify(sig, hash, pubkeyBytes) == nil
}

// zeroArray zeroes the memory of a scalar array.
func zeroArray(a *[scalarSize]byte) {
	for i := 0; i < scalarSize; i++ {
		a[i] = 0x00
	}
}

// schnorrSign generates a BIP-340 signature over the secp256k1 curve for the
// provided hash (which should be the result of hashing a larger message) using
// the given nonce and private key.  The produced signature is deterministic
// (same message, nonce, and key yield the same signature) and canonical.
//
// WARNING: The hash MUST be 32 bytes and both the nonce and private keys must
// NOT be 0.  Since this is an internal use function, these preconditions MUST
// be satisfied by the caller.
func schnorrSign(privKey, nonce *btcec.ModNScalar, pubKey *btcec.PublicKey, hash []byte,
	opts *signOptions) (*Signature, error) {

	// The algorithm for producing a BIP-340 signature is described in
	// README.md and is reproduced here for reference:
	//
	// G = curve generator
	// n = curve order
	// d = private key
	// m = message
	// a = input randomness
	// r, s = signature
	//
	// 1. d' = int(d)
	// 2. Fail if m is not 32 bytes
	// 3. Fail if d = 0 or d >= n
	// 4. P = d'*G
	// 5. Negate d if P.y is odd
	// 6. t = bytes(d) xor tagged_hash("BIP0340/aux", t || bytes(P) || m)
	// 7. rand = tagged_hash("BIP0340/nonce", a)
	// 8. k' = int(rand) mod n
	// 9. Fail if k' = 0
	// 10. R = 'k*G
	// 11. Negate k if R.y id odd
	// 12. e = tagged_hash("BIP0340/challenge", bytes(R) || bytes(P) || m) mod n
	// 13. sig = bytes(R) || bytes((k + e*d)) mod n
	// 14. If Verify(bytes(P), m, sig) fails, abort.
	// 15. return sig.
	//
	// Note that the set of functional options passed in may modify the
	// above algorithm. Namely if CustomNonce is used, then steps 6-8 are
	// replaced with a process that generates the nonce using rfc6979. If
	// FastSign is passed, then we skip set 14.

	// NOTE: Steps 1-9 are performed by the caller.

	//
	// Step 10.
	//
	// R = kG
	var R btcec.JacobianPoint
	k := *nonce
	btcec.ScalarBaseMultNonConst(&k, &R)

	// Step 11.
	//
	// Negate nonce k if R.y is odd (R.y is the y coordinate of the point R)
	//
	// Note that R must be in affine coordinates for this check.
	R.ToAffine()
	if R.Y.IsOdd() {
		k.Negate()
	}

	// Step 12.
	//
	// e = tagged_hash("BIP0340/challenge", bytes(R) || bytes(P) || m) mod n
	pBytes := SerializePubKey(pubKey)
	commitment := chainhash.TaggedHash(
		chainhash.TagBIP0340Challenge, R.X.Bytes()[:], pBytes, hash,
	)

	var e btcec.ModNScalar
	if overflow := e.SetBytes((*[32]byte)(commitment)); overflow != 0 {
		k.Zero()
		str := "hash of (r || P || m) too big"
		return nil, signatureError(ecdsa_schnorr.ErrSchnorrHashValue, str)
	}

	// Step 13.
	//
	// s = k + e*d mod n
	s := new(btcec.ModNScalar).Mul2(&e, privKey).Add(&k)
	k.Zero()

	sig := NewSignature(&R.X, s)

	// Step 14.
	//
	// If Verify(bytes(P), m, sig) fails, abort.
	if !opts.fastSign {
		if err := schnorrVerify(sig, hash, pBytes); err != nil {
			return nil, err
		}
	}

	// Step 15.
	//
	// Return (r, s)
	return sig, nil
}

// SignOption is a functional option argument that allows callers to modify the
// way we generate BIP-340 schnorr signatures.
type SignOption func(*signOptions)

// signOptions houses the set of functional options that can be used to modify
// the method used to generate the BIP-340 signature.
type signOptions struct {
	// fastSign determines if we'll skip the check at the end of the routine
	// where we attempt to verify the produced signature.
	fastSign bool

	// authNonce allows the user to pass in their own nonce information, which
	// is useful for schemes like mu-sig.
	authNonce *[32]byte
}

// defaultSignOptions returns the default set of signing operations.
func defaultSignOptions() *signOptions {
	return &signOptions{}
}

// FastSign forces signing to skip the extra verification step at the end.
// Performance sensitive applications may opt to use this option to speed up the
// signing operation.
func FastSign() SignOption {
	return func(o *signOptions) {
		o.fastSign = true
	}
}

// CustomNonce allows users to pass in a custom set of auxData that's used as
// input randomness to generate the nonce used during signing. Users may want
// to specify this custom value when using multi-signatures schemes such as
// Mu-Sig2. If this option isn't set, then rfc6979 will be used to generate the
// nonce material.
func CustomNonce(auxData [32]byte) SignOption {
	return func(o *signOptions) {
		o.authNonce = &auxData
	}
}

// Sign generates an BIP-340 signature over the secp256k1 curve for the
// provided hash (which should be the result of hashing a larger message) using
// the given private key.  The produced signature is deterministic (same
// message and same key yield the same signature) and canonical.
//
// Note that the current signing implementation has a few remaining variable
// time aspects which make use of the private key and the generated nonce,
// which can expose the signer to constant time attacks.  As a result, this
// function should not be used in situations where there is the possibility of
// someone having EM field/cache/etc access.
func Sign(privKey *btcec.PrivateKey, hash []byte,
	signOpts ...SignOption) (*Signature, error) {

	// First, parse the set of optional signing options.
	opts := defaultSignOptions()
	for _, option := range signOpts {
		option(opts)
	}

	// The algorithm for producing a BIP-340 signature is described in
	// README.md and is reproduced here for reference:
	//
	// G = curve generator
	// n = curve order
	// d = private key
	// m = message
	// a = input randomness
	// r, s = signature
	//
	// 1. d' = int(d)
	// 2. Fail if m is not 32 bytes
	// 3. Fail if d = 0 or d >= n
	// 4. P = d'*G
	// 5. Negate d if P.y is odd
	// 6. t = bytes(d) xor tagged_hash("BIP0340/aux", t || bytes(P) || m)
	// 7. rand = tagged_hash("BIP0340/nonce", a)
	// 8. k' = int(rand) mod n
	// 9. Fail if k' = 0
	// 10. R = 'k*G
	// 11. Negate k if R.y id odd
	// 12. e = tagged_hash("BIP0340/challenge", bytes(R) || bytes(P) || mod) mod n
	// 13. sig = bytes(R) || bytes((k + e*d)) mod n
	// 14. If Verify(bytes(P), m, sig) fails, abort.
	// 15. return sig.
	//
	// Note that the set of functional options passed in may modify the
	// above algorithm. Namely if CustomNonce is used, then steps 6-8 are
	// replaced with a process that generates the nonce using rfc6979. If
	// FastSign is passed, then we skip set 14.

	// Step 1.
	//
	// d' = int(d)
	var privKeyScalar btcec.ModNScalar
	privKeyScalar.Set(&privKey.Key)

	// Step 2.
	//
	// Fail if m is not 32 bytes
	if len(hash) != scalarSize {
		str := fmt.Sprintf("wrong size for message hash (got %v, want %v)",
			len(hash), scalarSize)
		return nil, signatureError(ecdsa_schnorr.ErrInvalidHashLen, str)
	}

	// Step 3.
	//
	// Fail if d = 0 or d >= n
	if privKeyScalar.IsZero() {
		str := "private key is zero"
		return nil, signatureError(ecdsa_schnorr.ErrPrivateKeyIsZero, str)
	}

	// Step 4.
	//
	// P = 'd*G
	pub := privKey.PubKey()

	// Step 5.
	//
	// Negate d if P.y is odd.
	pubKeyBytes := pub.SerializeCompressed()
	if pubKeyBytes[0] == secp.PubKeyFormatCompressedOdd {
		privKeyScalar.Negate()
	}

	// At this point, we check to see if a CustomNonce has been passed in,
	// and if so, then we'll deviate from the main routine here by
	// generating the nonce value as specified by BIP-0340.
	if opts.authNonce != nil {
		// Step 6.
		//
		// t = bytes(d) xor tagged_hash("BIP0340/aux", a)
		privBytes := privKeyScalar.Bytes()
		t := chainhash.TaggedHash(
			chainhash.TagBIP0340Aux, (*opts.authNonce)[:],
		)
		for i := 0; i < len(t); i++ {
			t[i] ^= privBytes[i]
		}

		// Step 7.
		//
		// rand = tagged_hash("BIP0340/nonce", t || bytes(P) || m)
		//
		// We snip off the first byte of the serialized pubkey, as we
		// only need the x coordinate and not the market byte.
		rand := chainhash.TaggedHash(
			chainhash.TagBIP0340Nonce, t[:], pubKeyBytes[1:], hash,
		)

		// Step 8.
		//
		// k'= int(rand) mod n
		var kPrime btcec.ModNScalar
		kPrime.SetBytes((*[32]byte)(rand))

		// Step 9.
		//
		// Fail if k' = 0
		if kPrime.IsZero() {
			str := fmt.Sprintf("generated nonce is zero")
			return nil, signatureError(ecdsa_schnorr.ErrSchnorrHashValue, str)
		}

		sig, err := schnorrSign(&privKeyScalar, &kPrime, pub, hash, opts)
		kPrime.Zero()
		if err != nil {
			return nil, err
		}

		return sig, nil
	}

	var privKeyBytes [scalarSize]byte
	privKeyScalar.PutBytes(&privKeyBytes)
	defer zeroArray(&privKeyBytes)
	for iteration := uint32(0); ; iteration++ {
		// Step 6-9.
		//
		// Use RFC6979 to generate a deterministic nonce k in [1, n-1]
		// parameterized by the private key, message being signed, extra data
		// that identifies the scheme, and an iteration count
		k := btcec.NonceRFC6979(
			privKeyBytes[:], hash, rfc6979ExtraDataV0[:], nil, iteration,
		)

		// Steps 10-15.
		sig, err := schnorrSign(&privKeyScalar, k, pub, hash, opts)
		k.Zero()
		if err != nil {
			// Try again with a new nonce.
			continue
		}

		return sig, nil
	}
}
