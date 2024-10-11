// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

package secec

import (
	"crypto"
	csrand "crypto/rand"
	"errors"
	"fmt"
	"io"

	"gitlab.com/yawning/tuplehash"

	"gitlab.com/yawning/secp256k1-voi"
)

const (
	wantedEntropyBytes = 256 / 8
	maxScalarResamples = 8
	domainSepECDSA     = "ECDSA-Sign"
)

var (
	errInvalidEncoding = errors.New("secp256k1/secec: invalid signature encoding")
	errInvalidScalar   = errors.New("secp256k1/secec: invalid scalar")
	errInvalidDigest   = errors.New("secp256k1/secec: invalid digest")
	errInvalidRorS     = errors.New("secp256k1/secec: r or s is zero")
	errRIsInfinity     = errors.New("secp256k1/secec: R is the point at infinity")
	errVNeqR           = errors.New("secp256k1/secec: v does not equal r")
	errSigCheckFailed  = errors.New("secp256k1/secec: failed to verify new sig")

	errEntropySource     = errors.New("secp256k1/secec: entropy source failure")
	errRejectionSampling = errors.New("secp256k1/secec: failed rejection sampling")
)

// SignatureEncoding is a ECDSA signature encoding method.
type SignatureEncoding int

const (
	// EncodingASN1 is an ASN.1 `SEQUENCE { r INTEGER, s INTEGER }`.
	EncodingASN1 SignatureEncoding = iota
	// EncodingCompact is `[R | S]`, with the scalars encoded as
	// 32-byte big-endian integers.
	EncodingCompact
	// EncodingCompactRecoverable is `[R | S | V]`, with the scalars
	// encoded as 32-byte big-endian integers, and `V` being in the
	// range `[0,3]`.
	EncodingCompactRecoverable
)

// ECDSAOptions can be used with `PrivateKey.Sign` or `PublicKey.Verify`
// to select ECDSA options.
type ECDSAOptions struct {
	// Hash is the digest algorithm used to hash the digest.  It is
	// used only for the purpose of validating input parameters.  If
	// unspecified, [crypto.SHA256] will be assumed.
	Hash crypto.Hash

	// Encoding selects the signature encoding format.
	Encoding SignatureEncoding

	// SelfVerify will cause the signing process to verify the
	// signature after signing, to improve resilience against certain
	// fault attacks.
	//
	// WARNING: If this is set, signing will be significantly more
	// expensive.
	SelfVerify bool

	// RejectMalleable will cause the verification process to
	// reject signatures where `s > n / 2`.
	RejectMalleable bool
}

// HashFunc returns an identifier for the hash function used to produce
// the message passed to [crypto.Signer.Sign], or else zero to indicate
// that no hashing was done.
func (opt *ECDSAOptions) HashFunc() crypto.Hash {
	return opt.Hash
}

// Sign signs `digest` (which should be the result of hashing a larger
// message) using the PrivateKey `k`, using the signing procedure
// as specified in SEC 1, Version 2.0, Section 4.1.3.  It returns the
// byte-encoded signature.  If `opts` is not a `*ECDSAOptions` the
// output encoding will default to `EncodingASN1`.
//
// Notes: If `rand` is nil, [crypto/rand.Reader] will be used.
// `s` will always be less than or equal to `n / 2`.
func (k *PrivateKey) Sign(rand io.Reader, digest []byte, opts crypto.SignerOpts) ([]byte, error) {
	// Assume default parameters.
	sigEncoding := EncodingASN1
	selfVerify := false // XXX: Should this default to true?

	if opts != nil {
		hashFn := opts.HashFunc()
		// Override the defaults.
		if o, ok := opts.(*ECDSAOptions); ok {
			sigEncoding = o.Encoding
			selfVerify = o.SelfVerify
			if hashFn == crypto.Hash(0) {
				hashFn = crypto.SHA256
			}
		}

		// Check that the digest is sized correctly.
		expectedLen := hashFn.Size()
		if len(digest) != expectedLen {
			return nil, errInvalidDigest
		}
	}

	r, s, v, err := k.SignRaw(rand, digest)
	if err != nil {
		return nil, err
	}

	if selfVerify {
		// Failures here are really hard to test since it's hard to
		// force faults.
		if err = verify(k, nil, digest, r, s); err != nil {
			return nil, errSigCheckFailed
		}

		// It is annoying/expensive to check that `v` is correct because
		// we do not know if we negated s when we signed.  Just do the
		// light-weight thing and check that it is within the correct
		// range.
		const vMask = 0x03
		if v&vMask != v {
			return nil, errSigCheckFailed
		}
	}

	var sig []byte
	switch sigEncoding {
	case EncodingASN1:
		sig = BuildASN1Signature(r, s)
	case EncodingCompact:
		sig = BuildCompactSignature(r, s)
	case EncodingCompactRecoverable:
		sig = BuildCompactRecoverableSignature(r, s, v)
	default:
		// "Why, yes, this is after SignRaw. Don't do that then."
		return nil, errInvalidEncoding
	}

	return sig, nil
}

// SignRaw signs `digest` (which should be the result of hashing a larger
// message) using the PrivateKey `k`, using the signing procedure
// as specified in SEC 1, Version 2.0, Section 4.1.3.  It returns the
// tuple `(r, s, recovery_id)`.
//
// Notes: If `rand` is nil, [crypto/rand.Reader] will be used.
// `s` will always be less than or equal to `n / 2`.  `recovery_id`
// will always be in the range `[0, 3]`.
func (k *PrivateKey) SignRaw(rand io.Reader, digest []byte) (*secp256k1.Scalar, *secp256k1.Scalar, byte, error) {
	return sign(rand, k, digest)
}

// Verify verifies the byte encoded signature `sig` of `digest`,
// using the PublicKey `k`, using the verification procedure as specified
// in SEC 1, Version 2.0, Section 4.1.4.  Its return value records
// whether the signature is valid.  If `opts` is nil, the input encoding
// will default to `EncodingASN1`, and `s` in the range `[1,n)` will
// be accepted.
func (k *PublicKey) Verify(digest, sig []byte, opts *ECDSAOptions) bool {
	// Assume default parameters.
	sigEncoding := EncodingASN1
	rejectMalleable := false

	if opts != nil {
		hashFn := opts.Hash
		sigEncoding = opts.Encoding
		rejectMalleable = opts.RejectMalleable
		if hashFn == crypto.Hash(0) {
			hashFn = crypto.SHA256
		}

		// Check that the digest is sized correctly.
		expectedLen := hashFn.Size()
		if len(digest) != expectedLen {
			return false
		}
	}

	var (
		r, s *secp256k1.Scalar
		v    byte

		err error
	)

	switch sigEncoding {
	case EncodingASN1:
		r, s, err = ParseASN1Signature(sig)
	case EncodingCompact:
		r, s, err = ParseCompactSignature(sig)
	case EncodingCompactRecoverable:
		r, s, v, err = ParseCompactRecoverableSignature(sig)
	default:
		err = errInvalidEncoding
	}
	if err != nil {
		return false
	}

	if rejectMalleable && s.IsGreaterThanHalfN() != 0 {
		return false
	}

	switch sigEncoding {
	case EncodingASN1, EncodingCompact:
		return k.VerifyRaw(digest, r, s)
	case EncodingCompactRecoverable:
		q, err := RecoverPublicKey(digest, r, s, v)
		if err != nil {
			return false
		}
		return k.Equal(q)
	}

	panic("secp256k1/secec: BUG: NOT REACHED")
}

// VerifyRaw verifies the `(r, s)` signature of `digest`, using the
// PublicKey `k`, using the verification procedure as specified in
// SEC 1, Version 2.0, Section 4.1.4.  Its return value records
// whether the signature is valid.
func (k *PublicKey) VerifyRaw(digest []byte, r, s *secp256k1.Scalar) bool {
	return nil == verify(nil, k, digest, r, s)
}

// RecoverPublicKey recovers the public key from the signature
// `(r, s, recoveryID)` over `digest`.  `recoverID` MUST be in the range
// `[0,3]`.
//
// Note: `s` in the range `[1, n)` is considered valid here.  It is the
// caller's responsibility to check `s.IsGreaterThanHalfN()` as required.
func RecoverPublicKey(digest []byte, r, s *secp256k1.Scalar, recoveryID byte) (*PublicKey, error) {
	if r.IsZero() != 0 || s.IsZero() != 0 {
		return nil, errInvalidRorS
	}

	// This roughly follows SEC 1, Version 2.0, Section 4.1.6, except
	// that instead of computing all possible R candidates from r,
	// the recoveryID explicitly encodes which point to use.

	R, err := secp256k1.RecoverPoint(r, recoveryID)
	if err != nil {
		return nil, err
	}
	if R.IsIdentity() != 0 {
		// This can NEVER happen as secp256k1.RecoverPoint always
		// returns a point that is on the curve or an error.
		panic(errRIsInfinity)
	}

	// 1.5. Compute e from M using Steps 2 and 3 of ECDSA signature verification.

	e, err := hashToScalar(digest)
	if err != nil {
		return nil, err
	}
	negE := secp256k1.NewScalar().Negate(e)

	// 1.6.1 Compute a candidate public key as: Q = r^(−1)(sR − eG).
	//
	// Rewriting this to be nicer, (-e)*r^(-1) * G + s*r^(-1) * R.

	rInv := secp256k1.NewScalar().Invert(r)
	u1 := secp256k1.NewScalar().Multiply(negE, rInv)
	u2 := secp256k1.NewScalar().Multiply(s, rInv)

	Q := secp256k1.NewIdentityPoint().DoubleScalarMultBasepointVartime(u1, u2, R)

	return NewPublicKeyFromPoint(Q)
}

func sign(rand io.Reader, d *PrivateKey, hBytes []byte) (*secp256k1.Scalar, *secp256k1.Scalar, byte, error) {
	var recoveryID byte

	// Note/yawning: `e` (derived from `hash`) in steps 4 and 5, is
	// unchanged throughout the process even if a different `k`
	// needs to be selected, thus, the value is derived first
	// before the rejection sampling loop.

	// 4. Use the hash function selected during the setup procedure
	// to compute the hash value:
	//   H = Hash(M)
	// of length hashlen octets as specified in Section 3.5. If the
	// hash function outputs “invalid”, output “invalid” and stop.

	// 5. Derive an integer e from H as follows:
	// 5.1. Convert the octet string H to a bit string H using the
	// conversion routine specified in Section 2.3.2.
	// 5.2. Set E = H if ceil(log2(n)) >= 8(hashlen), and set E equal
	// to the leftmost ceil(log2(n)) bits of H if ceil(log2(n)) <
	// 8(hashlen).
	// 5.3. Convert the bit string E to an octet string E using the
	// conversion routine specified in Section 2.3.1.
	// 5.4. Convert the octet string E to an integer e using the
	// conversion routine specified in Section 2.3.8.

	e, err := hashToScalar(hBytes)
	if err != nil {
		return nil, nil, 0, err
	}

	// While I normally will be content to let idiots compromise
	// their signing keys, past precident (eg: Sony Computer
	// Entertainment America, Inc v. Hotz) shows that "idiots"
	// are also litigatious asshats.
	//
	// Hardening the user-provided RNG is a sensible thing
	// to do, even if this wasn't something that has historically
	// been a large problem.

	fixedRng, err := mitigateDebianAndSony(rand, domainSepECDSA, d, e)
	if err != nil {
		return nil, nil, 0, err
	}

	var r, s *secp256k1.Scalar
	for {
		// 1. Select an ephemeral elliptic curve key pair (k, R) with
		// R = (xR, yR) associated with the elliptic curve domain parameters
		// T established during the setup procedure using the key pair
		// generation primitive specified in Section 3.2.1.

		k, err := sampleRandomScalar(fixedRng)
		if err != nil {
			// This is essentially totally untestable, as:
			// - This is astronomically unlikely to begin with.
			// - `fixedRng` is cSHAKE or HMAC_DRBG, so it is hard to
			//   force it to generate pathologically bad output.
			return nil, nil, 0, fmt.Errorf("secp256k1/secec/ecdsa: failed to generate k: %w", err)
		}
		R := secp256k1.NewIdentityPoint().ScalarBaseMult(k)

		// 2. Convert the field element xR to an integer xR using the
		// conversion routine specified in Section 2.3.9.

		rXBytes, rYIsOdd := secp256k1.SplitUncompressedPoint(R.UncompressedBytes())

		// 3. Set r = xR mod n. If r = 0, or optionally r fails to meet
		// other publicly verifiable criteria (see below), return to Step 1.

		var didReduce uint64
		r, didReduce = secp256k1.NewScalarFromBytes((*[secp256k1.ScalarSize]byte)(rXBytes))
		if r.IsZero() != 0 {
			// This is essentially totally untestable since the odds
			// of generating `r = 0` is astronomically unlikely.
			continue
		}

		// (Steps 4/5 done prior to loop.)

		// 6. Compute: s = k^−1 (e + r * dU) mod n.
		// If s = 0, return to Step 1.

		kInv := secp256k1.NewScalar().Invert(k) //nolint:revive
		s = secp256k1.NewScalar()
		s.Multiply(r, d.scalar).Add(s, e).Multiply(s, kInv)
		if s.IsZero() == 0 {
			recoveryID = (byte(didReduce) << 1) | byte(rYIsOdd)
			break
		}
	}

	// 7. Output S = (r, s). Optionally, output additional
	// information needed to recover R efficiently from r (see below).

	// The signer may replace (r, s) with (r, −s mod n), because this
	// is an equivalent signature.
	//
	// Note/yawning: To prevent mallability, Shitcoin enforces s <= n/2.
	// As either is valid in any other context, always produce
	// signatures of that form.

	negateS := s.IsGreaterThanHalfN()
	s.ConditionalNegate(s, negateS)
	recoveryID ^= byte(negateS)

	return r, s, recoveryID, nil
}

func verify(d *PrivateKey, q *PublicKey, hBytes []byte, r, s *secp256k1.Scalar) error {
	// 1. If r and s are not both integers in the interval [1, n − 1],
	// output “invalid” and stop.
	//
	// Note/yawning: This is somewhat redundant because the various
	// ASN.1 parsing routines reject these, but we also support
	// verifying user supplied (r, s), so just check again.

	if r.IsZero() != 0 || s.IsZero() != 0 {
		return errInvalidRorS
	}

	// 2. Use the hash function established during the setup procedure
	// to compute the hash value:
	//   H = Hash(M)
	// of length hashlen octets as specified in Section 3.5. If the
	// hash function outputs “invalid”, output “invalid” and stop.

	// 3. Derive an integer e from H as follows:
	// 3.1. Convert the octet string H to a bit string H using the
	// conversion routine specified in Section 2.3.2.
	// 3.2. Set E = H if ceil(log2(n)) >= 8(hashlen), and set E equal
	// to the leftmost ceil(log2(n)) bits of H if ceil(log2(n)) <
	// 8(hashlen).
	// 3.3. Convert the bit string E to an octet string E using the
	// conversion routine specified in Section 2.3.1.
	// 3.4. Convert the octet string E to an integer e using the
	// conversion routine specified in Section 2.3.8.

	e, err := hashToScalar(hBytes)
	if err != nil {
		return err
	}

	// 4. Compute: u1 = e(s^−1) mod n and u2 = r(s^-1) mod n.

	sInv := secp256k1.NewScalar().Invert(s)
	u1 := secp256k1.NewScalar().Multiply(e, sInv)
	u2 := secp256k1.NewScalar().Multiply(r, sInv)

	R := secp256k1.NewIdentityPoint()
	switch d {
	case nil:
		// 5. Compute: R = (xR, yR) = u1 * G + u2 * QU.
		R.DoubleScalarMultBasepointVartime(u1, u2, q.point)
	default:
		// 4.1.5 Alternative Verifying Operation
		//
		// All verification steps are the same, except that in Step 5,
		// the verifier instead computes
		//
		// R = (xR, yR) = (u1 + u2 * d) * G
		u2.Multiply(u2, d.scalar)
		u1.Add(u1, u2)
		R.ScalarBaseMult(u1)
	}

	// If R = O, output “invalid” and stop.
	if R.IsIdentity() != 0 {
		return errRIsInfinity
	}

	// 6. Convert the field element xR to an integer xR using the
	// conversion routine specified in Section 2.3.9.
	//
	// 7. Set v = xR mod n.

	xRBytes, _ := R.XBytes() // Can't fail, R != Inf.
	v, _ := secp256k1.NewScalarFromBytes((*[secp256k1.ScalarSize]byte)(xRBytes))

	// 8. Compare v and r — if v = r, output “valid”, and if
	// v != r, output “invalid”.

	if v.Equal(r) != 1 {
		return errVNeqR
	}

	return nil
}

// hashToScalar converts a hash to a scalar per SEC 1, Version 2.0,
// Section 4.1.3, Step 5 (and Section 4.1.4, Step 3).
//
// Note: This also will reduce the resulting scalar such that it is
// in the range [0, n), which is fine for ECDSA.
func hashToScalar(hash []byte) (*secp256k1.Scalar, error) {
	if len(hash) < secp256k1.ScalarSize {
		return nil, errInvalidDigest
	}

	// TLDR; The left-most Ln-bits of hash.
	tmp := (*[secp256k1.ScalarSize]byte)(hash[:secp256k1.ScalarSize])
	s, _ := secp256k1.NewScalarFromBytes(tmp) // Reduction info unneeded.
	return s, nil
}

func mitigateDebianAndSony(rand io.Reader, ctx string, k *PrivateKey, e *secp256k1.Scalar) (io.Reader, error) {
	// There are documented attacks that can exploit even the
	// most subtle amounts of bias (< 1-bit) in the generation
	// of the ECDSA nonce.
	//
	// RFC 6979 proposes to use HMAC_DRBG instantiated
	// with the private key and message digest, and making
	// signatures fully deterministic.
	//
	// We go one step further, and use TupleHashXOF128 to mix
	// the private key, 256-bits of entropy, and the message
	// digest.
	//
	// See:
	// - https://eprint.iacr.org/2020/615.pdf
	// - https://eprint.iacr.org/2019/1155.pdf

	switch rand {
	case readerRFC6979SHA256:
		return newDrbgRFC6979(k.scalar, e), nil
	case nil:
		rand = csrand.Reader
	}

	var tmp [wantedEntropyBytes]byte
	if _, err := io.ReadFull(rand, tmp[:]); err != nil {
		return nil, fmt.Errorf("%w: %w", errEntropySource, err)
	}

	xof := tuplehash.NewTupleHashXOF128([]byte("Honorary Debian/Sony RNG mitigation:" + ctx))
	_, _ = xof.Write(k.scalar.Bytes())
	_, _ = xof.Write(tmp[:])
	_, _ = xof.Write(e.Bytes())
	return xof, nil
}

func sampleRandomScalar(rand io.Reader) (*secp256k1.Scalar, error) {
	// Do rejection sampling to ensure that there is no bias in the
	// scalar values.  Note that the odds of a single failure are
	// approximately p = 3.73 * 10^-39, so even requiring a single
	// retry is unlikely unless the entropy source is broken.
	var (
		tmp [secp256k1.ScalarSize]byte
		s   = secp256k1.NewScalar()
	)
	for i := 0; i < maxScalarResamples; i++ {
		if _, err := io.ReadFull(rand, tmp[:]); err != nil {
			return nil, fmt.Errorf("%w: %w", errEntropySource, err)
		}

		_, didReduce := s.SetBytes(&tmp)
		if didReduce == 0 && s.IsZero() == 0 { // Short circuit reject is ok.
			return s, nil
		}
	}

	return nil, errRejectionSampling
}
