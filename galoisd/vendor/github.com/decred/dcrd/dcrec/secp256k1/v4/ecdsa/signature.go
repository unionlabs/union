// Copyright (c) 2013-2014 The btcsuite developers
// Copyright (c) 2015-2022 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package ecdsa

import (
	"fmt"

	"github.com/decred/dcrd/dcrec/secp256k1/v4"
)

// References:
//   [GECC]: Guide to Elliptic Curve Cryptography (Hankerson, Menezes, Vanstone)
//
//   [ISO/IEC 8825-1]: Information technology — ASN.1 encoding rules:
//     Specification of Basic Encoding Rules (BER), Canonical Encoding Rules
//     (CER) and Distinguished Encoding Rules (DER)
//
//   [SEC1]: Elliptic Curve Cryptography (May 31, 2009, Version 2.0)
//     https://www.secg.org/sec1-v2.pdf

var (
	// zero32 is an array of 32 bytes used for the purposes of zeroing and is
	// defined here to avoid extra allocations.
	zero32 = [32]byte{}

	// orderAsFieldVal is the order of the secp256k1 curve group stored as a
	// field value.  It is provided here to avoid the need to create it multiple
	// times.
	orderAsFieldVal = func() secp256k1.FieldVal {
		var f secp256k1.FieldVal
		f.SetByteSlice(secp256k1.Params().N.Bytes())
		return f
	}()
)

const (
	// asn1SequenceID is the ASN.1 identifier for a sequence and is used when
	// parsing and serializing signatures encoded with the Distinguished
	// Encoding Rules (DER) format per section 10 of [ISO/IEC 8825-1].
	asn1SequenceID = 0x30

	// asn1IntegerID is the ASN.1 identifier for an integer and is used when
	// parsing and serializing signatures encoded with the Distinguished
	// Encoding Rules (DER) format per section 10 of [ISO/IEC 8825-1].
	asn1IntegerID = 0x02
)

// Signature is a type representing an ECDSA signature.
type Signature struct {
	r secp256k1.ModNScalar
	s secp256k1.ModNScalar
}

// NewSignature instantiates a new signature given some r and s values.
func NewSignature(r, s *secp256k1.ModNScalar) *Signature {
	return &Signature{*r, *s}
}

// Serialize returns the ECDSA signature in the Distinguished Encoding Rules
// (DER) format per section 10 of [ISO/IEC 8825-1] and such that the S component
// of the signature is less than or equal to the half order of the group.
//
// Note that the serialized bytes returned do not include the appended hash type
// used in Decred signature scripts.
func (sig *Signature) Serialize() []byte {
	// The format of a DER encoded signature is as follows:
	//
	// 0x30 <total length> 0x02 <length of R> <R> 0x02 <length of S> <S>
	//   - 0x30 is the ASN.1 identifier for a sequence.
	//   - Total length is 1 byte and specifies length of all remaining data.
	//   - 0x02 is the ASN.1 identifier that specifies an integer follows.
	//   - Length of R is 1 byte and specifies how many bytes R occupies.
	//   - R is the arbitrary length big-endian encoded number which
	//     represents the R value of the signature.  DER encoding dictates
	//     that the value must be encoded using the minimum possible number
	//     of bytes.  This implies the first byte can only be null if the
	//     highest bit of the next byte is set in order to prevent it from
	//     being interpreted as a negative number.
	//   - 0x02 is once again the ASN.1 integer identifier.
	//   - Length of S is 1 byte and specifies how many bytes S occupies.
	//   - S is the arbitrary length big-endian encoded number which
	//     represents the S value of the signature.  The encoding rules are
	//     identical as those for R.

	// Ensure the S component of the signature is less than or equal to the half
	// order of the group because both S and its negation are valid signatures
	// modulo the order, so this forces a consistent choice to reduce signature
	// malleability.
	sigS := new(secp256k1.ModNScalar).Set(&sig.s)
	if sigS.IsOverHalfOrder() {
		sigS.Negate()
	}

	// Serialize the R and S components of the signature into their fixed
	// 32-byte big-endian encoding.  Note that the extra leading zero byte is
	// used to ensure it is canonical per DER and will be stripped if needed
	// below.
	var rBuf, sBuf [33]byte
	sig.r.PutBytesUnchecked(rBuf[1:33])
	sigS.PutBytesUnchecked(sBuf[1:33])

	// Ensure the encoded bytes for the R and S components are canonical per DER
	// by trimming all leading zero bytes so long as the next byte does not have
	// the high bit set and it's not the final byte.
	canonR, canonS := rBuf[:], sBuf[:]
	for len(canonR) > 1 && canonR[0] == 0x00 && canonR[1]&0x80 == 0 {
		canonR = canonR[1:]
	}
	for len(canonS) > 1 && canonS[0] == 0x00 && canonS[1]&0x80 == 0 {
		canonS = canonS[1:]
	}

	// Total length of returned signature is 1 byte for each magic and length
	// (6 total), plus lengths of R and S.
	totalLen := 6 + len(canonR) + len(canonS)
	b := make([]byte, 0, totalLen)
	b = append(b, asn1SequenceID)
	b = append(b, byte(totalLen-2))
	b = append(b, asn1IntegerID)
	b = append(b, byte(len(canonR)))
	b = append(b, canonR...)
	b = append(b, asn1IntegerID)
	b = append(b, byte(len(canonS)))
	b = append(b, canonS...)
	return b
}

// zeroArray32 zeroes the provided 32-byte buffer.
func zeroArray32(b *[32]byte) {
	copy(b[:], zero32[:])
}

// fieldToModNScalar converts a field value to scalar modulo the group order and
// returns the scalar along with either 1 if it was reduced (aka it overflowed)
// or 0 otherwise.
//
// Note that a bool is not used here because it is not possible in Go to convert
// from a bool to numeric value in constant time and many constant-time
// operations require a numeric value.
func fieldToModNScalar(v *secp256k1.FieldVal) (secp256k1.ModNScalar, uint32) {
	var buf [32]byte
	v.PutBytes(&buf)
	var s secp256k1.ModNScalar
	overflow := s.SetBytes(&buf)
	zeroArray32(&buf)
	return s, overflow
}

// modNScalarToField converts a scalar modulo the group order to a field value.
func modNScalarToField(v *secp256k1.ModNScalar) secp256k1.FieldVal {
	var buf [32]byte
	v.PutBytes(&buf)
	var fv secp256k1.FieldVal
	fv.SetBytes(&buf)
	return fv
}

// Verify returns whether or not the signature is valid for the provided hash
// and secp256k1 public key.
func (sig *Signature) Verify(hash []byte, pubKey *secp256k1.PublicKey) bool {
	// The algorithm for verifying an ECDSA signature is given as algorithm 4.30
	// in [GECC].
	//
	// The following is a paraphrased version for reference:
	//
	// G = curve generator
	// N = curve order
	// Q = public key
	// m = message
	// R, S = signature
	//
	// 1. Fail if R and S are not in [1, N-1]
	// 2. e = H(m)
	// 3. w = S^-1 mod N
	// 4. u1 = e * w mod N
	//    u2 = R * w mod N
	// 5. X = u1G + u2Q
	// 6. Fail if X is the point at infinity
	// 7. x = X.x mod N (X.x is the x coordinate of X)
	// 8. Verified if x == R
	//
	// However, since all group operations are done internally in Jacobian
	// projective space, the algorithm is modified slightly here in order to
	// avoid an expensive inversion back into affine coordinates at step 7.
	// Credits to Greg Maxwell for originally suggesting this optimization.
	//
	// Ordinarily, step 7 involves converting the x coordinate to affine by
	// calculating x = x / z^2 (mod P) and then calculating the remainder as
	// x = x (mod N).  Then step 8 compares it to R.
	//
	// Note that since R is the x coordinate mod N from a random point that was
	// originally mod P, and the cofactor of the secp256k1 curve is 1, there are
	// only two possible x coordinates that the original random point could have
	// been to produce R: x, where x < N, and x+N, where x+N < P.
	//
	// This implies that the signature is valid if either:
	// a) R == X.x / X.z^2 (mod P)
	//    => R * X.z^2 == X.x (mod P)
	// --or--
	// b) R + N < P && R + N == X.x / X.z^2 (mod P)
	//    => R + N < P && (R + N) * X.z^2 == X.x (mod P)
	//
	// Therefore the following modified algorithm is used:
	//
	// 1. Fail if R and S are not in [1, N-1]
	// 2. e = H(m)
	// 3. w = S^-1 mod N
	// 4. u1 = e * w mod N
	//    u2 = R * w mod N
	// 5. X = u1G + u2Q
	// 6. Fail if X is the point at infinity
	// 7. z = (X.z)^2 mod P (X.z is the z coordinate of X)
	// 8. Verified if R * z == X.x (mod P)
	// 9. Fail if R + N >= P
	// 10. Verified if (R + N) * z == X.x (mod P)

	// Step 1.
	//
	// Fail if R and S are not in [1, N-1].
	if sig.r.IsZero() || sig.s.IsZero() {
		return false
	}

	// Step 2.
	//
	// e = H(m)
	var e secp256k1.ModNScalar
	e.SetByteSlice(hash)

	// Step 3.
	//
	// w = S^-1 mod N
	w := new(secp256k1.ModNScalar).InverseValNonConst(&sig.s)

	// Step 4.
	//
	// u1 = e * w mod N
	// u2 = R * w mod N
	u1 := new(secp256k1.ModNScalar).Mul2(&e, w)
	u2 := new(secp256k1.ModNScalar).Mul2(&sig.r, w)

	// Step 5.
	//
	// X = u1G + u2Q
	var X, Q, u1G, u2Q secp256k1.JacobianPoint
	pubKey.AsJacobian(&Q)
	secp256k1.ScalarBaseMultNonConst(u1, &u1G)
	secp256k1.ScalarMultNonConst(u2, &Q, &u2Q)
	secp256k1.AddNonConst(&u1G, &u2Q, &X)

	// Step 6.
	//
	// Fail if X is the point at infinity
	if (X.X.IsZero() && X.Y.IsZero()) || X.Z.IsZero() {
		return false
	}

	// Step 7.
	//
	// z = (X.z)^2 mod P (X.z is the z coordinate of X)
	z := new(secp256k1.FieldVal).SquareVal(&X.Z)

	// Step 8.
	//
	// Verified if R * z == X.x (mod P)
	sigRModP := modNScalarToField(&sig.r)
	result := new(secp256k1.FieldVal).Mul2(&sigRModP, z).Normalize()
	if result.Equals(&X.X) {
		return true
	}

	// Step 9.
	//
	// Fail if R + N >= P
	if sigRModP.IsGtOrEqPrimeMinusOrder() {
		return false
	}

	// Step 10.
	//
	// Verified if (R + N) * z == X.x (mod P)
	sigRModP.Add(&orderAsFieldVal)
	result.Mul2(&sigRModP, z).Normalize()
	return result.Equals(&X.X)
}

// IsEqual compares this Signature instance to the one passed, returning true if
// both Signatures are equivalent.  A signature is equivalent to another, if
// they both have the same scalar value for R and S.
func (sig *Signature) IsEqual(otherSig *Signature) bool {
	return sig.r.Equals(&otherSig.r) && sig.s.Equals(&otherSig.s)
}

// ParseDERSignature parses a signature in the Distinguished Encoding Rules
// (DER) format per section 10 of [ISO/IEC 8825-1] and enforces the following
// additional restrictions specific to secp256k1:
//
// - The R and S values must be in the valid range for secp256k1 scalars:
//   - Negative values are rejected
//   - Zero is rejected
//   - Values greater than or equal to the secp256k1 group order are rejected
func ParseDERSignature(sig []byte) (*Signature, error) {
	// The format of a DER encoded signature for secp256k1 is as follows:
	//
	// 0x30 <total length> 0x02 <length of R> <R> 0x02 <length of S> <S>
	//   - 0x30 is the ASN.1 identifier for a sequence
	//   - Total length is 1 byte and specifies length of all remaining data
	//   - 0x02 is the ASN.1 identifier that specifies an integer follows
	//   - Length of R is 1 byte and specifies how many bytes R occupies
	//   - R is the arbitrary length big-endian encoded number which
	//     represents the R value of the signature.  DER encoding dictates
	//     that the value must be encoded using the minimum possible number
	//     of bytes.  This implies the first byte can only be null if the
	//     highest bit of the next byte is set in order to prevent it from
	//     being interpreted as a negative number.
	//   - 0x02 is once again the ASN.1 integer identifier
	//   - Length of S is 1 byte and specifies how many bytes S occupies
	//   - S is the arbitrary length big-endian encoded number which
	//     represents the S value of the signature.  The encoding rules are
	//     identical as those for R.
	//
	// NOTE: The DER specification supports specifying lengths that can occupy
	// more than 1 byte, however, since this is specific to secp256k1
	// signatures, all lengths will be a single byte.
	const (
		// minSigLen is the minimum length of a DER encoded signature and is
		// when both R and S are 1 byte each.
		//
		// 0x30 + <1-byte> + 0x02 + 0x01 + <byte> + 0x2 + 0x01 + <byte>
		minSigLen = 8

		// maxSigLen is the maximum length of a DER encoded signature and is
		// when both R and S are 33 bytes each.  It is 33 bytes because a
		// 256-bit integer requires 32 bytes and an additional leading null byte
		// might be required if the high bit is set in the value.
		//
		// 0x30 + <1-byte> + 0x02 + 0x21 + <33 bytes> + 0x2 + 0x21 + <33 bytes>
		maxSigLen = 72

		// sequenceOffset is the byte offset within the signature of the
		// expected ASN.1 sequence identifier.
		sequenceOffset = 0

		// dataLenOffset is the byte offset within the signature of the expected
		// total length of all remaining data in the signature.
		dataLenOffset = 1

		// rTypeOffset is the byte offset within the signature of the ASN.1
		// identifier for R and is expected to indicate an ASN.1 integer.
		rTypeOffset = 2

		// rLenOffset is the byte offset within the signature of the length of
		// R.
		rLenOffset = 3

		// rOffset is the byte offset within the signature of R.
		rOffset = 4
	)

	// The signature must adhere to the minimum and maximum allowed length.
	sigLen := len(sig)
	if sigLen < minSigLen {
		str := fmt.Sprintf("malformed signature: too short: %d < %d", sigLen,
			minSigLen)
		return nil, signatureError(ErrSigTooShort, str)
	}
	if sigLen > maxSigLen {
		str := fmt.Sprintf("malformed signature: too long: %d > %d", sigLen,
			maxSigLen)
		return nil, signatureError(ErrSigTooLong, str)
	}

	// The signature must start with the ASN.1 sequence identifier.
	if sig[sequenceOffset] != asn1SequenceID {
		str := fmt.Sprintf("malformed signature: format has wrong type: %#x",
			sig[sequenceOffset])
		return nil, signatureError(ErrSigInvalidSeqID, str)
	}

	// The signature must indicate the correct amount of data for all elements
	// related to R and S.
	if int(sig[dataLenOffset]) != sigLen-2 {
		str := fmt.Sprintf("malformed signature: bad length: %d != %d",
			sig[dataLenOffset], sigLen-2)
		return nil, signatureError(ErrSigInvalidDataLen, str)
	}

	// Calculate the offsets of the elements related to S and ensure S is inside
	// the signature.
	//
	// rLen specifies the length of the big-endian encoded number which
	// represents the R value of the signature.
	//
	// sTypeOffset is the offset of the ASN.1 identifier for S and, like its R
	// counterpart, is expected to indicate an ASN.1 integer.
	//
	// sLenOffset and sOffset are the byte offsets within the signature of the
	// length of S and S itself, respectively.
	rLen := int(sig[rLenOffset])
	sTypeOffset := rOffset + rLen
	sLenOffset := sTypeOffset + 1
	if sTypeOffset >= sigLen {
		str := "malformed signature: S type indicator missing"
		return nil, signatureError(ErrSigMissingSTypeID, str)
	}
	if sLenOffset >= sigLen {
		str := "malformed signature: S length missing"
		return nil, signatureError(ErrSigMissingSLen, str)
	}

	// The lengths of R and S must match the overall length of the signature.
	//
	// sLen specifies the length of the big-endian encoded number which
	// represents the S value of the signature.
	sOffset := sLenOffset + 1
	sLen := int(sig[sLenOffset])
	if sOffset+sLen != sigLen {
		str := "malformed signature: invalid S length"
		return nil, signatureError(ErrSigInvalidSLen, str)
	}

	// R elements must be ASN.1 integers.
	if sig[rTypeOffset] != asn1IntegerID {
		str := fmt.Sprintf("malformed signature: R integer marker: %#x != %#x",
			sig[rTypeOffset], asn1IntegerID)
		return nil, signatureError(ErrSigInvalidRIntID, str)
	}

	// Zero-length integers are not allowed for R.
	if rLen == 0 {
		str := "malformed signature: R length is zero"
		return nil, signatureError(ErrSigZeroRLen, str)
	}

	// R must not be negative.
	if sig[rOffset]&0x80 != 0 {
		str := "malformed signature: R is negative"
		return nil, signatureError(ErrSigNegativeR, str)
	}

	// Null bytes at the start of R are not allowed, unless R would otherwise be
	// interpreted as a negative number.
	if rLen > 1 && sig[rOffset] == 0x00 && sig[rOffset+1]&0x80 == 0 {
		str := "malformed signature: R value has too much padding"
		return nil, signatureError(ErrSigTooMuchRPadding, str)
	}

	// S elements must be ASN.1 integers.
	if sig[sTypeOffset] != asn1IntegerID {
		str := fmt.Sprintf("malformed signature: S integer marker: %#x != %#x",
			sig[sTypeOffset], asn1IntegerID)
		return nil, signatureError(ErrSigInvalidSIntID, str)
	}

	// Zero-length integers are not allowed for S.
	if sLen == 0 {
		str := "malformed signature: S length is zero"
		return nil, signatureError(ErrSigZeroSLen, str)
	}

	// S must not be negative.
	if sig[sOffset]&0x80 != 0 {
		str := "malformed signature: S is negative"
		return nil, signatureError(ErrSigNegativeS, str)
	}

	// Null bytes at the start of S are not allowed, unless S would otherwise be
	// interpreted as a negative number.
	if sLen > 1 && sig[sOffset] == 0x00 && sig[sOffset+1]&0x80 == 0 {
		str := "malformed signature: S value has too much padding"
		return nil, signatureError(ErrSigTooMuchSPadding, str)
	}

	// The signature is validly encoded per DER at this point, however, enforce
	// additional restrictions to ensure R and S are in the range [1, N-1] since
	// valid ECDSA signatures are required to be in that range per spec.
	//
	// Also note that while the overflow checks are required to make use of the
	// specialized mod N scalar type, rejecting zero here is not strictly
	// required because it is also checked when verifying the signature, but
	// there really isn't a good reason not to fail early here on signatures
	// that do not conform to the ECDSA spec.

	// Strip leading zeroes from R.
	rBytes := sig[rOffset : rOffset+rLen]
	for len(rBytes) > 0 && rBytes[0] == 0x00 {
		rBytes = rBytes[1:]
	}

	// R must be in the range [1, N-1].  Notice the check for the maximum number
	// of bytes is required because SetByteSlice truncates as noted in its
	// comment so it could otherwise fail to detect the overflow.
	var r secp256k1.ModNScalar
	if len(rBytes) > 32 {
		str := "invalid signature: R is larger than 256 bits"
		return nil, signatureError(ErrSigRTooBig, str)
	}
	if overflow := r.SetByteSlice(rBytes); overflow {
		str := "invalid signature: R >= group order"
		return nil, signatureError(ErrSigRTooBig, str)
	}
	if r.IsZero() {
		str := "invalid signature: R is 0"
		return nil, signatureError(ErrSigRIsZero, str)
	}

	// Strip leading zeroes from S.
	sBytes := sig[sOffset : sOffset+sLen]
	for len(sBytes) > 0 && sBytes[0] == 0x00 {
		sBytes = sBytes[1:]
	}

	// S must be in the range [1, N-1].  Notice the check for the maximum number
	// of bytes is required because SetByteSlice truncates as noted in its
	// comment so it could otherwise fail to detect the overflow.
	var s secp256k1.ModNScalar
	if len(sBytes) > 32 {
		str := "invalid signature: S is larger than 256 bits"
		return nil, signatureError(ErrSigSTooBig, str)
	}
	if overflow := s.SetByteSlice(sBytes); overflow {
		str := "invalid signature: S >= group order"
		return nil, signatureError(ErrSigSTooBig, str)
	}
	if s.IsZero() {
		str := "invalid signature: S is 0"
		return nil, signatureError(ErrSigSIsZero, str)
	}

	// Create and return the signature.
	return NewSignature(&r, &s), nil
}

// sign generates an ECDSA signature over the secp256k1 curve for the provided
// hash (which should be the result of hashing a larger message) using the given
// nonce and private key and returns it along with an additional public key
// recovery code and success indicator.  Upon success, the produced signature is
// deterministic (same message, nonce, and key yield the same signature) and
// canonical in accordance with BIP0062.
//
// Note that signRFC6979 makes use of this function as it is the primary ECDSA
// signing logic.  It differs in that it accepts a nonce to use when signing and
// may not successfully produce a valid signature for the given nonce.  It is
// primarily separated for testing purposes.
func sign(privKey, nonce *secp256k1.ModNScalar, hash []byte) (*Signature, byte, bool) {
	// The algorithm for producing an ECDSA signature is given as algorithm 4.29
	// in [GECC].
	//
	// The following is a paraphrased version for reference:
	//
	// G = curve generator
	// N = curve order
	// d = private key
	// m = message
	// r, s = signature
	//
	// 1. Select random nonce k in [1, N-1]
	// 2. Compute kG
	// 3. r = kG.x mod N (kG.x is the x coordinate of the point kG)
	//    Repeat from step 1 if r = 0
	// 4. e = H(m)
	// 5. s = k^-1(e + dr) mod N
	//    Repeat from step 1 if s = 0
	// 6. Return (r,s)
	//
	// This is slightly modified here to conform to RFC6979 and BIP 62 as
	// follows:
	//
	// A. Instead of selecting a random nonce in step 1, use RFC6979 to generate
	//    a deterministic nonce in [1, N-1] parameterized by the private key,
	//    message being signed, and an iteration count for the repeat cases
	// B. Negate s calculated in step 5 if it is > N/2
	//    This is done because both s and its negation are valid signatures
	//    modulo the curve order N, so it forces a consistent choice to reduce
	//    signature malleability

	// NOTE: Step 1 is performed by the caller.
	//
	// Step 2.
	//
	// Compute kG
	//
	// Note that the point must be in affine coordinates.
	k := nonce
	var kG secp256k1.JacobianPoint
	secp256k1.ScalarBaseMultNonConst(k, &kG)
	kG.ToAffine()

	// Step 3.
	//
	// r = kG.x mod N
	// Repeat from step 1 if r = 0
	r, overflow := fieldToModNScalar(&kG.X)
	if r.IsZero() {
		return nil, 0, false
	}

	// Since the secp256k1 curve has a cofactor of 1, when recovering a
	// public key from an ECDSA signature over it, there are four possible
	// candidates corresponding to the following cases:
	//
	// 1) The X coord of the random point is < N and its Y coord even
	// 2) The X coord of the random point is < N and its Y coord is odd
	// 3) The X coord of the random point is >= N and its Y coord is even
	// 4) The X coord of the random point is >= N and its Y coord is odd
	//
	// Rather than forcing the recovery procedure to check all possible
	// cases, this creates a recovery code that uniquely identifies which of
	// the cases apply by making use of 2 bits.  Bit 0 identifies the
	// oddness case and Bit 1 identifies the overflow case (aka when the X
	// coord >= N).
	//
	// It is also worth noting that making use of Hasse's theorem shows
	// there are around log_2((p-n)/p) ~= -127.65 ~= 1 in 2^127 points where
	// the X coordinate is >= N.  It is not possible to calculate these
	// points since that would require breaking the ECDLP, but, in practice
	// this strongly implies with extremely high probability that there are
	// only a few actual points for which this case is true.
	pubKeyRecoveryCode := byte(overflow<<1) | byte(kG.Y.IsOddBit())

	// Step 4.
	//
	// e = H(m)
	//
	// Note that this actually sets e = H(m) mod N which is correct since
	// it is only used in step 5 which itself is mod N.
	var e secp256k1.ModNScalar
	e.SetByteSlice(hash)

	// Step 5 with modification B.
	//
	// s = k^-1(e + dr) mod N
	// Repeat from step 1 if s = 0
	// s = -s if s > N/2
	kinv := new(secp256k1.ModNScalar).InverseValNonConst(k)
	s := new(secp256k1.ModNScalar).Mul2(privKey, &r).Add(&e).Mul(kinv)
	if s.IsZero() {
		return nil, 0, false
	}
	if s.IsOverHalfOrder() {
		s.Negate()

		// Negating s corresponds to the random point that would have been
		// generated by -k (mod N), which necessarily has the opposite
		// oddness since N is prime, thus flip the pubkey recovery code
		// oddness bit accordingly.
		pubKeyRecoveryCode ^= 0x01
	}

	// Step 6.
	//
	// Return (r,s)
	return NewSignature(&r, s), pubKeyRecoveryCode, true
}

// signRFC6979 generates a deterministic ECDSA signature according to RFC 6979
// and BIP0062 and returns it along with an additional public key recovery code
// for efficiently recovering the public key from the signature.
func signRFC6979(privKey *secp256k1.PrivateKey, hash []byte) (*Signature, byte) {
	// The algorithm for producing an ECDSA signature is given as algorithm 4.29
	// in [GECC].
	//
	// The following is a paraphrased version for reference:
	//
	// G = curve generator
	// N = curve order
	// d = private key
	// m = message
	// r, s = signature
	//
	// 1. Select random nonce k in [1, N-1]
	// 2. Compute kG
	// 3. r = kG.x mod N (kG.x is the x coordinate of the point kG)
	//    Repeat from step 1 if r = 0
	// 4. e = H(m)
	// 5. s = k^-1(e + dr) mod N
	//    Repeat from step 1 if s = 0
	// 6. Return (r,s)
	//
	// This is slightly modified here to conform to RFC6979 and BIP 62 as
	// follows:
	//
	// A. Instead of selecting a random nonce in step 1, use RFC6979 to generate
	//    a deterministic nonce in [1, N-1] parameterized by the private key,
	//    message being signed, and an iteration count for the repeat cases
	// B. Negate s calculated in step 5 if it is > N/2
	//    This is done because both s and its negation are valid signatures
	//    modulo the curve order N, so it forces a consistent choice to reduce
	//    signature malleability

	privKeyScalar := &privKey.Key
	var privKeyBytes [32]byte
	privKeyScalar.PutBytes(&privKeyBytes)
	defer zeroArray32(&privKeyBytes)
	for iteration := uint32(0); ; iteration++ {
		// Step 1 with modification A.
		//
		// Generate a deterministic nonce in [1, N-1] parameterized by the
		// private key, message being signed, and iteration count.
		k := secp256k1.NonceRFC6979(privKeyBytes[:], hash, nil, nil, iteration)

		// Steps 2-6.
		sig, pubKeyRecoveryCode, success := sign(privKeyScalar, k, hash)
		k.Zero()
		if !success {
			continue
		}

		return sig, pubKeyRecoveryCode
	}
}

// Sign generates an ECDSA signature over the secp256k1 curve for the provided
// hash (which should be the result of hashing a larger message) using the given
// private key.  The produced signature is deterministic (same message and same
// key yield the same signature) and canonical in accordance with RFC6979 and
// BIP0062.
func Sign(key *secp256k1.PrivateKey, hash []byte) *Signature {
	signature, _ := signRFC6979(key, hash)
	return signature
}

const (
	// compactSigSize is the size of a compact signature.  It consists of a
	// compact signature recovery code byte followed by the R and S components
	// serialized as 32-byte big-endian values. 1+32*2 = 65.
	// for the R and S components. 1+32+32=65.
	compactSigSize = 65

	// compactSigMagicOffset is a value used when creating the compact signature
	// recovery code inherited from Bitcoin and has no meaning, but has been
	// retained for compatibility.  For historical purposes, it was originally
	// picked to avoid a binary representation that would allow compact
	// signatures to be mistaken for other components.
	compactSigMagicOffset = 27

	// compactSigCompPubKey is a value used when creating the compact signature
	// recovery code to indicate the original public key was compressed.
	compactSigCompPubKey = 4

	// pubKeyRecoveryCodeOddnessBit specifies the bit that indicates the oddess
	// of the Y coordinate of the random point calculated when creating a
	// signature.
	pubKeyRecoveryCodeOddnessBit = 1 << 0

	// pubKeyRecoveryCodeOverflowBit specifies the bit that indicates the X
	// coordinate of the random point calculated when creating a signature was
	// >= N, where N is the order of the group.
	pubKeyRecoveryCodeOverflowBit = 1 << 1
)

// SignCompact produces a compact ECDSA signature over the secp256k1 curve for
// the provided hash (which should be the result of hashing a larger message)
// using the given private key.  The isCompressedKey parameter specifies if the
// produced signature should reference a compressed public key or not.
//
// Compact signature format:
// <1-byte compact sig recovery code><32-byte R><32-byte S>
//
// The compact sig recovery code is the value 27 + public key recovery code + 4
// if the compact signature was created with a compressed public key.
func SignCompact(key *secp256k1.PrivateKey, hash []byte, isCompressedKey bool) []byte {
	// Create the signature and associated pubkey recovery code and calculate
	// the compact signature recovery code.
	sig, pubKeyRecoveryCode := signRFC6979(key, hash)
	compactSigRecoveryCode := compactSigMagicOffset + pubKeyRecoveryCode
	if isCompressedKey {
		compactSigRecoveryCode += compactSigCompPubKey
	}

	// Output <compactSigRecoveryCode><32-byte R><32-byte S>.
	var b [compactSigSize]byte
	b[0] = compactSigRecoveryCode
	sig.r.PutBytesUnchecked(b[1:33])
	sig.s.PutBytesUnchecked(b[33:65])
	return b[:]
}

// RecoverCompact attempts to recover the secp256k1 public key from the provided
// compact signature and message hash.  It first verifies the signature, and, if
// the signature matches then the recovered public key will be returned as well
// as a boolean indicating whether or not the original key was compressed.
func RecoverCompact(signature, hash []byte) (*secp256k1.PublicKey, bool, error) {
	// The following is very loosely based on the information and algorithm that
	// describes recovering a public key from and ECDSA signature in section
	// 4.1.6 of [SEC1].
	//
	// Given the following parameters:
	//
	// G = curve generator
	// N = group order
	// P = field prime
	// Q = public key
	// m = message
	// e = hash of the message
	// r, s = signature
	// X = random point used when creating signature whose x coordinate is r
	//
	// The equation to recover a public key candidate from an ECDSA signature
	// is:
	// Q = r^-1(sX - eG).
	//
	// This can be verified by plugging it in for Q in the sig verification
	// equation:
	// X = s^-1(eG + rQ) (mod N)
	//  => s^-1(eG + r(r^-1(sX - eG))) (mod N)
	//  => s^-1(eG + sX - eG) (mod N)
	//  => s^-1(sX) (mod N)
	//  => X (mod N)
	//
	// However, note that since r is the x coordinate mod N from a random point
	// that was originally mod P, and the cofactor of the secp256k1 curve is 1,
	// there are four possible points that the original random point could have
	// been to produce r: (r,y), (r,-y), (r+N,y), and (r+N,-y).  At least 2 of
	// those points will successfully verify, and all 4 will successfully verify
	// when the original x coordinate was in the range [N+1, P-1], but in any
	// case, only one of them corresponds to the original private key used.
	//
	// The method described by section 4.1.6 of [SEC1] to determine which one is
	// the correct one involves calculating each possibility as a candidate
	// public key and comparing the candidate to the authentic public key.  It
	// also hints that it is possible to generate the signature in a such a
	// way that only one of the candidate public keys is viable.
	//
	// A more efficient approach that is specific to the secp256k1 curve is used
	// here instead which is to produce a "pubkey recovery code" when signing
	// that uniquely identifies which of the 4 possibilities is correct for the
	// original random point and using that to recover the pubkey directly as
	// follows:
	//
	// 1. Fail if r and s are not in [1, N-1]
	// 2. Convert r to integer mod P
	// 3. If pubkey recovery code overflow bit is set:
	//    3.1 Fail if r + N >= P
	//    3.2 r = r + N (mod P)
	// 4. y = +sqrt(r^3 + 7) (mod P)
	//    4.1 Fail if y does not exist
	//    4.2 y = -y if needed to match pubkey recovery code oddness bit
	// 5. X = (r, y)
	// 6. e = H(m) mod N
	// 7. w = r^-1 mod N
	// 8. u1 = -(e * w) mod N
	//    u2 = s * w mod N
	// 9. Q = u1G + u2X
	// 10. Fail if Q is the point at infinity

	// A compact signature consists of a recovery byte followed by the R and
	// S components serialized as 32-byte big-endian values.
	if len(signature) != compactSigSize {
		str := fmt.Sprintf("malformed signature: wrong size: %d != %d",
			len(signature), compactSigSize)
		return nil, false, signatureError(ErrSigInvalidLen, str)
	}

	// Parse and validate the compact signature recovery code.
	const (
		minValidCode = compactSigMagicOffset
		maxValidCode = compactSigMagicOffset + compactSigCompPubKey + 3
	)
	sigRecoveryCode := signature[0]
	if sigRecoveryCode < minValidCode || sigRecoveryCode > maxValidCode {
		str := fmt.Sprintf("invalid signature: public key recovery code %d is "+
			"not in the valid range [%d, %d]", sigRecoveryCode, minValidCode,
			maxValidCode)
		return nil, false, signatureError(ErrSigInvalidRecoveryCode, str)
	}
	sigRecoveryCode -= compactSigMagicOffset
	wasCompressed := sigRecoveryCode&compactSigCompPubKey != 0
	pubKeyRecoveryCode := sigRecoveryCode & 3

	// Step 1.
	//
	// Parse and validate the R and S signature components.
	//
	// Fail if r and s are not in [1, N-1].
	var r, s secp256k1.ModNScalar
	if overflow := r.SetByteSlice(signature[1:33]); overflow {
		str := "invalid signature: R >= group order"
		return nil, false, signatureError(ErrSigRTooBig, str)
	}
	if r.IsZero() {
		str := "invalid signature: R is 0"
		return nil, false, signatureError(ErrSigRIsZero, str)
	}
	if overflow := s.SetByteSlice(signature[33:]); overflow {
		str := "invalid signature: S >= group order"
		return nil, false, signatureError(ErrSigSTooBig, str)
	}
	if s.IsZero() {
		str := "invalid signature: S is 0"
		return nil, false, signatureError(ErrSigSIsZero, str)
	}

	// Step 2.
	//
	// Convert r to integer mod P.
	fieldR := modNScalarToField(&r)

	// Step 3.
	//
	// If pubkey recovery code overflow bit is set:
	if pubKeyRecoveryCode&pubKeyRecoveryCodeOverflowBit != 0 {
		// Step 3.1.
		//
		// Fail if r + N >= P
		//
		// Either the signature or the recovery code must be invalid if the
		// recovery code overflow bit is set and adding N to the R component
		// would exceed the field prime since R originally came from the X
		// coordinate of a random point on the curve.
		if fieldR.IsGtOrEqPrimeMinusOrder() {
			str := "invalid signature: signature R + N >= P"
			return nil, false, signatureError(ErrSigOverflowsPrime, str)
		}

		// Step 3.2.
		//
		// r = r + N (mod P)
		fieldR.Add(&orderAsFieldVal)
	}

	// Step 4.
	//
	// y = +sqrt(r^3 + 7) (mod P)
	// Fail if y does not exist.
	// y = -y if needed to match pubkey recovery code oddness bit
	//
	// The signature must be invalid if the calculation fails because the X
	// coord originally came from a random point on the curve which means there
	// must be a Y coord that satisfies the equation for a valid signature.
	oddY := pubKeyRecoveryCode&pubKeyRecoveryCodeOddnessBit != 0
	var y secp256k1.FieldVal
	if valid := secp256k1.DecompressY(&fieldR, oddY, &y); !valid {
		str := "invalid signature: not for a valid curve point"
		return nil, false, signatureError(ErrPointNotOnCurve, str)
	}

	// Step 5.
	//
	// X = (r, y)
	var X secp256k1.JacobianPoint
	X.X.Set(fieldR.Normalize())
	X.Y.Set(y.Normalize())
	X.Z.SetInt(1)

	// Step 6.
	//
	// e = H(m) mod N
	var e secp256k1.ModNScalar
	e.SetByteSlice(hash)

	// Step 7.
	//
	// w = r^-1 mod N
	w := new(secp256k1.ModNScalar).InverseValNonConst(&r)

	// Step 8.
	//
	// u1 = -(e * w) mod N
	// u2 = s * w mod N
	u1 := new(secp256k1.ModNScalar).Mul2(&e, w).Negate()
	u2 := new(secp256k1.ModNScalar).Mul2(&s, w)

	// Step 9.
	//
	// Q = u1G + u2X
	var Q, u1G, u2X secp256k1.JacobianPoint
	secp256k1.ScalarBaseMultNonConst(u1, &u1G)
	secp256k1.ScalarMultNonConst(u2, &X, &u2X)
	secp256k1.AddNonConst(&u1G, &u2X, &Q)

	// Step 10.
	//
	// Fail if Q is the point at infinity.
	//
	// Either the signature or the pubkey recovery code must be invalid if the
	// recovered pubkey is the point at infinity.
	if (Q.X.IsZero() && Q.Y.IsZero()) || Q.Z.IsZero() {
		str := "invalid signature: recovered pubkey is the point at infinity"
		return nil, false, signatureError(ErrPointNotOnCurve, str)
	}

	// Notice that the public key is in affine coordinates.
	Q.ToAffine()
	pubKey := secp256k1.NewPublicKey(&Q.X, &Q.Y)
	return pubKey, wasCompressed, nil
}
