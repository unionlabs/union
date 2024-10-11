// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

package bitcoin

import (
	"bytes"
	"crypto"
	csrand "crypto/rand"
	"crypto/sha256"
	"crypto/subtle"
	"errors"
	"fmt"
	"io"
	"strings"

	"gitlab.com/yawning/secp256k1-voi"
	"gitlab.com/yawning/secp256k1-voi/internal/disalloweq"
	"gitlab.com/yawning/secp256k1-voi/internal/field"
	"gitlab.com/yawning/secp256k1-voi/secec"
)

const (
	// SchnorrPublicKeySize is the size of a BIP-0340 Schnorr public key
	// in bytes.
	SchnorrPublicKeySize = 32
	// SchnorrSignatureSize is the size of a BIP-0340 Schnorr signature
	// in bytes.
	SchnorrSignatureSize = 64

	schnorrEntropySize = 32

	schnorrTagAux       = "BIP0340/aux"
	schnorrTagNonce     = "BIP0340/nonce"
	schnorrTagChallenge = "BIP0340/challenge"
)

var (
	errAIsInfinity      = errors.New("secp256k1/secec/bitcoin: public key is the point at infinity")
	errAIsUninitialized = errors.New("secp256k1/secec/bitcoin: uninitialized public key")
	errEntropySource    = errors.New("secp256k1/secec/bitcoin: entropy source failure")
	errInvalidDomainSep = errors.New("secp256k1/secec/bitcoin: invalid domain separator")
	errInvalidPublicKey = errors.New("secp256k1/secec/bitcoin: invalid public key")
	errKPrimeIsZero     = errors.New("secp256k1/secec/bitcoin: k' = 0")
	errSigCheckFailed   = errors.New("secp256k1/secec/bitcoin: failed to verify new sig")
)

// PreHashSchnorrMessage pre-hashes the message `msg`, with the
// domain-separator `name`, as suggested in BIP-0340.  It returns
// the byte-encoded pre-hashed message.
//
// Note: The spec is silent regarding 0-length `name` values.  This
// implementation rejects them with an error, given that the main
// motivation for pre-hashing in the first place is domain-separation.
func PreHashSchnorrMessage(name string, msg []byte) ([]byte, error) {
	// Go strings are UTF-8 by default, but this accepts user input.
	if n := strings.ToValidUTF8(name, ""); n != name || len(name) == 0 {
		return nil, errInvalidDomainSep
	}

	return schnorrTaggedHash(name, msg), nil
}

// SchnorrPrivateKey is a private key for sigining BIP-0340 Schnorr signatures.
type SchnorrPrivateKey struct {
	_ disalloweq.DisallowEqual

	dPrime *secp256k1.Scalar // The raw scalar
	d      *secp256k1.Scalar // dPrime negated as required

	publicKey *SchnorrPublicKey
}

// Bytes returns a copy of the encoding of the private key.
func (k *SchnorrPrivateKey) Bytes() []byte {
	return k.dPrime.Bytes()
}

// Scalar returns a copy of the scalar underlying `k`.
func (k *SchnorrPrivateKey) Scalar() *secp256k1.Scalar {
	return secp256k1.NewScalarFrom(k.dPrime)
}

// Equal returns whether `x` represents the same private key as `k`.
// This check is performed in constant time as long as the key types
// match.
func (k *SchnorrPrivateKey) Equal(x crypto.PrivateKey) bool {
	other, ok := x.(*SchnorrPrivateKey)
	if !ok {
		return false
	}

	return other.dPrime.Equal(k.dPrime) == 1
}

func (k *SchnorrPrivateKey) Public() crypto.PublicKey {
	return k.publicKey
}

// PublicKey returns the Schnorr public key corresponding to `k`.
func (k *SchnorrPrivateKey) PublicKey() *SchnorrPublicKey {
	return k.publicKey
}

// Sign signs `msg` using the SchnorrPrivateKey `k`, using the signing
// procedure as specified in BIP-0340.  It returns the byte-encoded
// signature.
//
// Note: If `rand` is nil, [crypto/rand.Reader] will be used.
func (k *SchnorrPrivateKey) Sign(rand io.Reader, msg []byte, _ crypto.SignerOpts) ([]byte, error) {
	// BIP-0340 cautions about how deterministic nonce creation a la
	// RFC6979 can lead to key compromise if the same key is shared
	// between ECDSA and Schnorr signatures due to nonce reuse.
	//
	// We implement the nonce generation as per the BIP, so the
	// tagged hashing mechanism is sufficient to handle this case,
	// and will do the "right" thing when the entropy source is
	// broken by incorporating k and msg into the nonce.
	//
	// In theory, the cSHAKE based algorithm that is used by default
	// in our ECDSA algorithm also will do the right thing, but
	// following the standard lets us test against their test
	// vectors.

	if rand == nil {
		rand = csrand.Reader
	}

	var auxEntropy [schnorrEntropySize]byte
	if _, err := io.ReadFull(rand, auxEntropy[:]); err != nil {
		return nil, fmt.Errorf("%w: %w", errEntropySource, err)
	}

	return signSchnorr(&auxEntropy, k, msg)
}

// NewSchnorrPrivateKey checks that `key` is valid, and returns a
// SchnorrPrivateKey.
func NewSchnorrPrivateKey(key []byte) (*SchnorrPrivateKey, error) {
	ecdsaPriv, err := secec.NewPrivateKey(key)
	if err != nil {
		return nil, err
	}

	return NewSchnorrPrivateKeyFromECDSA(ecdsaPriv), nil
}

// GenerateSchnorrKey generates a new SchnorrPrivateKey,
// using [crypto/rand.Reader] as the entropy source.
func GenerateSchnorrKey() (*SchnorrPrivateKey, error) {
	ecdsaPriv, err := secec.GenerateKey()
	if err != nil {
		return nil, err
	}

	return NewSchnorrPrivateKeyFromECDSA(ecdsaPriv), nil
}

// NewSchnorrPrivateKeyFromECDSA returns the SchnorrPrivateKey corresponding
// to the ECDSA PrivateKey `sk`.
func NewSchnorrPrivateKeyFromECDSA(sk *secec.PrivateKey) *SchnorrPrivateKey {
	priv := &SchnorrPrivateKey{
		dPrime: sk.Scalar(),
	}

	// Precompute P and d for signing.
	pXBytes, negateD := secp256k1.SplitUncompressedPoint(sk.PublicKey().Bytes())
	priv.d = secp256k1.NewScalar().ConditionalNegate(priv.dPrime, negateD)

	pt := sk.PublicKey().Point() // Copies.
	pt.ConditionalNegate(pt, negateD)

	priv.publicKey = &SchnorrPublicKey{
		point:  pt,
		xBytes: pXBytes,
	}

	return priv
}

// SchnorrPublicKey is a public key for verifying BIP-0340 Schnorr signatures.
type SchnorrPublicKey struct {
	_ disalloweq.DisallowEqual

	point  *secp256k1.Point // INVARIANT: Never identity, lift_x applied
	xBytes []byte           // SEC 1 X-coordinate
}

// Bytes returns a copy of the byte encoding of the public key.
func (k *SchnorrPublicKey) Bytes() []byte {
	if k.xBytes == nil {
		panic(errAIsUninitialized)
	}

	return bytes.Clone(k.xBytes)
}

// Point returns a copy of the point underlying `k`.
func (k *SchnorrPublicKey) Point() *secp256k1.Point {
	return secp256k1.NewPointFrom(k.point)
}

// Equal returns whether `x` represents the same public key as `k`.
// This check is performed in constant time as long as the key types
// match.
func (k *SchnorrPublicKey) Equal(x crypto.PublicKey) bool {
	other, ok := x.(*SchnorrPublicKey)
	if !ok {
		return false
	}

	// Comparing the serialized form is faster than comparing points.
	return subtle.ConstantTimeCompare(k.xBytes, other.xBytes) == 1
}

// Verify verifies the Schnorr signature `sig` of `msg`, using the
// SchnorrPublicKey `k`, using the verification procedure as specified
// in BIP-0340.  Its return value records whether the signature is
// valid.
func (k *SchnorrPublicKey) Verify(msg, sig []byte) bool {
	if len(sig) != SchnorrSignatureSize {
		return false
	}

	// The algorithm Verify(pk, m, sig) is defined as:

	// Let P = lift_x(int(pk)); fail if that fails.
	//
	// Note/yawning: k contains a pre-deserialized point, deserialization
	// process is equivalent to lift_x.

	// Let r = int(sig[0:32]); fail if r >= p.
	// Let s = int(sig[32:64]); fail if s >= n.
	// Let e = int(hashBIP0340/challenge(bytes(r) || bytes(P) || m)) mod n.

	ok, s, e, sigRXBytes := parseSchnorrSignature(k.xBytes, msg, sig)
	if !ok {
		return false
	}

	// Let R = s*G - e*P.

	e.Negate(e)
	R := secp256k1.NewIdentityPoint().DoubleScalarMultBasepointVartime(s, e, k.point)

	// Fail if is_infinite(R).
	// Fail if not has_even_y(R).
	// Fail if x(R) != r.
	// Return success iff no failure occurred before reaching this point.

	return verifySchnorrSignatureR(sigRXBytes, R)
}

// NewSchnorrPublicKey checks that `key` is valid, and returns a
// SchnorrPublicKey.
func NewSchnorrPublicKey(key []byte) (*SchnorrPublicKey, error) {
	if len(key) != SchnorrPublicKeySize {
		return nil, errInvalidPublicKey
	}

	var ptBytes [secp256k1.CompressedPointSize]byte
	ptBytes[0] = 0x02
	copy(ptBytes[1:], key)

	pt, err := secp256k1.NewPointFromBytes(ptBytes[:])
	if err != nil {
		return nil, fmt.Errorf("secp256k1/secec/bitcoin: failed to decompress public key: %w", err)
	}

	return &SchnorrPublicKey{
		point:  pt,
		xBytes: bytes.Clone(key),
	}, nil
}

// NewSchnorrPublicKeyFromPoint checks that `point` is valid, and returns
// a SchnorrPublicKey.
//
// Note: This routine accepts any point on the curve, and will fixup the
// Y-coordinate if required.
func NewSchnorrPublicKeyFromPoint(point *secp256k1.Point) (*SchnorrPublicKey, error) {
	pt := secp256k1.NewPointFrom(point)
	if pt.IsIdentity() != 0 {
		// This could check before to save the copy on the fail case,
		// but why would anyone pass in the point at infinity?
		return nil, errAIsInfinity
	}

	// "Implicitly choosing the Y coordinate that is even"
	pt.ConditionalNegate(pt, pt.IsYOdd())

	ptX, _ := pt.XBytes() // Can't fail, pt != Inf

	return &SchnorrPublicKey{
		point:  pt,
		xBytes: ptX,
	}, nil
}

// NewSchnorrPublicKeyFromECDSA returns the SchnorrPublicKey corresponding
// to the ECDSA PrivateKey `sk`.
func NewSchnorrPublicKeyFromECDSA(pk *secec.PublicKey) *SchnorrPublicKey {
	// Can't fail, pk.Point is never identity.
	pub, _ := NewSchnorrPublicKeyFromPoint(pk.Point()) // An extra copy :(
	return pub
}

func schnorrTaggedHash(tag string, vals ...[]byte) []byte {
	hashedTag := sha256.Sum256([]byte(tag))

	h := sha256.New()
	_, _ = h.Write(hashedTag[:])
	_, _ = h.Write(hashedTag[:])
	for _, v := range vals {
		_, _ = h.Write(v)
	}

	return h.Sum(nil)
}

func signSchnorr(auxRand *[schnorrEntropySize]byte, sk *SchnorrPrivateKey, msg []byte) ([]byte, error) {
	// The algorithm Sign(sk, m) is defined as:

	// Let d' = int(sk)
	// Fail if d' = 0 or d' >= n
	// Let P = d'*G
	// Let d = d' if has_even_y(P), otherwise let d = n - d' .
	//
	// Note/yawning: sk is a pre-deserialized private key, that is
	// guaranteed to be valid.  There is no reason not to pre-compute
	// P and d so we do.

	pBytes, d := sk.publicKey.xBytes, sk.d

	// Let t be the byte-wise xor of bytes(d) and hashBIP0340/aux(a)[11].

	var t [schnorrEntropySize]byte
	subtle.XORBytes(t[:], schnorrTaggedHash(schnorrTagAux, auxRand[:]), d.Bytes())

	// Let rand = hashBIP0340/nonce(t || bytes(P) || m)[12].

	rand := schnorrTaggedHash(schnorrTagNonce, t[:], pBytes, msg)

	// Let k' = int(rand) mod n[13].

	kPrime, _ := secp256k1.NewScalarFromBytes((*[secp256k1.ScalarSize]byte)(rand)) //nolint:revive

	// Fail if k' = 0.

	if kPrime.IsZero() != 0 {
		// In theory this is a probabalistic failure, however the odds
		// of this happening are basically non-existent.
		return nil, errKPrimeIsZero
	}

	// Let R = k'*G.

	R := secp256k1.NewIdentityPoint().ScalarBaseMult(kPrime)
	rXBytes, rYIsOdd := secp256k1.SplitUncompressedPoint(R.UncompressedBytes())

	// Let k = k' if has_even_y(R), otherwise let k = n - k' .

	k := secp256k1.NewScalar().ConditionalNegate(kPrime, rYIsOdd)

	// Let e = int(hashBIP0340/challenge(bytes(R) || bytes(P) || m)) mod n.

	eBytes := schnorrTaggedHash(schnorrTagChallenge, rXBytes, pBytes, msg)
	e, _ := secp256k1.NewScalarFromBytes((*[secp256k1.ScalarSize]byte)(eBytes))

	// Let sig = bytes(R) || bytes((k + ed) mod n).

	sum := secp256k1.NewScalar().Multiply(e, d) // ed
	sum.Add(k, sum)                             // k + ed
	sig := make([]byte, 0, SchnorrSignatureSize)
	sig = append(sig, rXBytes...)
	sig = append(sig, sum.Bytes()...)

	// If Verify(bytes(P), m, sig) (see below) returns failure, abort[14].
	//
	// Note/yawning: "It is recommended, but can be omitted if the
	// computation cost is prohibitive.".  Doing this check with the
	// standard verification routine, triples the time it takes to sign.
	//
	// There is a trivial optimization that replaces the
	// DoubleScalarMultBasepointVartime with a ScalarBaseMult when
	// verifying signatures signed by your own private key, so do that
	// instead.
	//
	// Note: Apart from the faster calculation of R, the verification
	// process is identical to the normal verify.

	if !verifySchnorrSelf(d, sk.PublicKey().Bytes(), msg, sig) {
		// This is likely totally untestable, since it requires
		// generating a signature that doesn't verify.
		return nil, errSigCheckFailed
	}

	return sig, nil
}

func verifySchnorrSelf(d *secp256k1.Scalar, pkXBytes, msg, sig []byte) bool {
	ok, s, e, sigRXBytes := parseSchnorrSignature(pkXBytes, msg, sig)
	if !ok {
		return false
	}

	// Let R = (s - d*e)*G.
	//
	// Note/yawning: d is the private key (or it's negation), so deriving
	// R needs to be done in constant-time.

	factor := secp256k1.NewScalar().Multiply(d, e)
	factor.Subtract(s, factor)
	R := secp256k1.NewIdentityPoint().ScalarBaseMult(factor)

	return verifySchnorrSignatureR(sigRXBytes, R)
}

func parseSchnorrSignature(pkXBytes, msg, sig []byte) (bool, *secp256k1.Scalar, *secp256k1.Scalar, []byte) {
	if len(sig) != SchnorrSignatureSize {
		return false, nil, nil, nil
	}

	// Let r = int(sig[0:32]); fail if r >= p.
	//
	// Note/yawning: If one were to want to do this without using the
	// internal field package, the point decompression routine also
	// would work, but would be slower.

	sigRXBytes := sig[0:32]
	if !field.BytesAreCanonical((*[field.ElementSize]byte)(sigRXBytes)) {
		return false, nil, nil, nil
	}

	// Let s = int(sig[32:64]); fail if s >= n.

	s, err := secp256k1.NewScalarFromCanonicalBytes((*[secp256k1.ScalarSize]byte)(sig[32:64]))
	if err != nil {
		return false, nil, nil, nil
	}

	// Let e = int(hashBIP0340/challenge(bytes(r) || bytes(P) || m)) mod n.

	eBytes := schnorrTaggedHash(schnorrTagChallenge, sigRXBytes, pkXBytes, msg)
	e, _ := secp256k1.NewScalarFromBytes((*[secp256k1.ScalarSize]byte)(eBytes))

	return true, s, e, sigRXBytes
}

func verifySchnorrSignatureR(sigRXBytes []byte, R *secp256k1.Point) bool { //nolint:gocritic
	// Fail if is_infinite(R).

	if R.IsIdentity() != 0 {
		return false
	}

	// Note/yawning: Doing it this way saves repeated rescaling, since
	// the curve implementation always does the inversion.

	rXBytes, rYIsOdd := secp256k1.SplitUncompressedPoint(R.UncompressedBytes())

	// Fail if not has_even_y(R).

	if rYIsOdd != 0 {
		return false
	}

	// Fail if x(R) != r.
	//
	// Note/yawning: Vartime compare, because this is verification.

	if !bytes.Equal(rXBytes, sigRXBytes) {
		return false
	}

	return true
}
