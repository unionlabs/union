// Copyright (c) 2016 The Go Authors. All rights reserved.
// Copyright (c) 2019-2021 Oasis Labs Inc. All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//   * Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//    * Redistributions in binary form must reproduce the above
// copyright notice, this list of conditions and the following disclaimer
// in the documentation and/or other materials provided with the
// distribution.
//    * Neither the name of Google Inc. nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

// Package ed25519 implements the Ed25519 signature algorithm. See
// https://ed25519.cr.yp.to/.
//
// These functions are mostly compatible with the “Ed25519” function defined in
// RFC 8032. However, unlike RFC 8032's formulation, this package's private key
// representation includes a public key suffix to make multiple signing
// operations with the same key more efficient. This package refers to the RFC
// 8032 private key as the “seed”.
//
// The default verification behavior is neither identical to that of the
// Go standard library, nor that as specified in FIPS 186-5.  If exact
// compatibility with either definition is required, use the appropriate
// VerifyOptions presets.
package ed25519

import (
	"bytes"
	"crypto"
	cryptorand "crypto/rand"
	"crypto/sha512"
	"fmt"
	"io"
	"strconv"

	"github.com/oasisprotocol/curve25519-voi/curve"
	"github.com/oasisprotocol/curve25519-voi/curve/scalar"
	"github.com/oasisprotocol/curve25519-voi/internal/subtle"
	_ "github.com/oasisprotocol/curve25519-voi/internal/toolchain"
)

const (
	// PublicKeySize is the size, in bytes, of public keys as used in this package.
	PublicKeySize = 32

	// PrivateKeySize is the size, in bytes, of private keys as used in this package.
	PrivateKeySize = 64

	// SignatureSize is the size, in bytes, of signatures generated and verified by this package.
	SignatureSize = 64

	// SeedSize is the size, in bytes, of private key seeds. These are the private key representations used by RFC 8032.
	SeedSize = 32

	// ContextMaxSize is the maximum allowed context length for Ed25519ctx.
	ContextMaxSize = 255

	addedRandomnessSize = 32
)

var (
	// VerifyOptionsDefault specifies verification behavior that is
	// used by this package by default.
	VerifyOptionsDefault = &VerifyOptions{
		AllowSmallOrderR: true,
	}

	// VerifyOptionsStdLib specifies verification behavior that is
	// compatible with that provided by the Go `crypto/ed25519` package.
	//
	// Note: This preset is incompatible with batch verification.
	VerifyOptionsStdLib = &VerifyOptions{
		AllowSmallOrderA:   true,
		AllowSmallOrderR:   true,
		AllowNonCanonicalA: true,
		CofactorlessVerify: true,
	}

	// VerifyOptionsFIPS_186_5 specifies verification behavior that is
	// compatible with FIPS 186-5.  The behavior provided by this preset
	// also matches RFC 8032 (with cofactored verification).
	VerifyOptionsFIPS_186_5 = &VerifyOptions{
		AllowSmallOrderA: true,
		AllowSmallOrderR: true,
	}

	// VerifyOptionsZIP_215 specifies verification behavior that is
	// compatible with ZIP-215.
	VerifyOptionsZIP_215 = &VerifyOptions{
		AllowSmallOrderA:   true,
		AllowSmallOrderR:   true,
		AllowNonCanonicalA: true,
		AllowNonCanonicalR: true,
	}

	optionsDefault = &Options{
		Verify: VerifyOptionsDefault,
	}

	addedRandomnessPadding [1024]byte

	_ crypto.Signer = (PrivateKey)(nil)
)

// Options can be used with PrivateKey.Sign or VerifyWithOptions
// to select Ed25519 variants.
type Options struct {
	// Hash can be crypto.Hash(0) for Ed25519/Ed25519ctx, or crypto.SHA512
	// for Ed25519ph.
	Hash crypto.Hash

	// Context is an optional domain separation context for Ed25519ph and
	// Ed25519ctx. It must be less than or equal to ContextMaxSize
	// in length.
	//
	// Warning: If Hash is crypto.Hash(0) and Context is a zero length
	// string, plain Ed25519 will be used instead of Ed25519ctx.
	Context string

	// AddedRandomness includes additional randomness when signing to
	// attempt to mitigate certain fault injection and side-channel
	// attacks.
	//
	// Warning: If this is set, signatures will be non-deterministic.
	AddedRandomness bool

	// SelfVerify will cause the signing process to verify the signature
	// after signing, to improve resilience against certain fault attacks.
	//
	// Warning: If this is set, signing will be significantly more
	// expensive.
	SelfVerify bool

	// Verify allows specifying verification behavior for compatibility
	// with other Ed25519 implementations.  If left unspecified, the
	// VerifyOptionsDefault will be used, which should be acceptable
	// for most use cases.
	Verify *VerifyOptions
}

// HashFunc returns an identifier for the hash function used to produce
// the message pased to Signer.Sign. For the Ed25519 family this must
// be crypto.Hash(0) for Ed25519/Ed25519ctx, or crypto.SHA512 for
// Ed25519ph.
func (opt *Options) HashFunc() crypto.Hash {
	return opt.Hash
}

func (opt *Options) verify() (dom2Flag, []byte, error) {
	var (
		context []byte
		f       dom2Flag = fPure
	)

	if vOpts := opt.Verify; vOpts != nil {
		if vOpts.AllowNonCanonicalR && vOpts.CofactorlessVerify {
			return f, nil, fmt.Errorf("ed25519: incompatible verification options")
		}
	}

	if l := len(opt.Context); l > 0 {
		if l > ContextMaxSize {
			return f, nil, fmt.Errorf("ed25519: bad context length: %d", l)
		}

		context = []byte(opt.Context)

		// This disallows Ed25519ctx with a 0 length context, which is
		// technically allowed by the RFC ("SHOULD NOT be empty"), but
		// is discouraged and somewhat nonsensical anyway.
		f = fCtx
	}

	return f, context, nil
}

func checkHash(f dom2Flag, message []byte, hashFunc crypto.Hash) (dom2Flag, error) {
	switch hashFunc {
	case crypto.SHA512:
		if l := len(message); l != sha512.Size {
			return f, fmt.Errorf("ed25519: bad message hash length: %d", l)
		}
		f = fPh
	case crypto.Hash(0):
	default:
		return f, fmt.Errorf("ed25519: expected opts HashFunc zero (unhashed message, for Ed25519/Ed25519ctx) or SHA-512 (for Ed25519ph)")
	}

	return f, nil
}

// VerifyOptions can be used to specify verification behavior for compatibility
// with other Ed25519 implementations.
type VerifyOptions struct {
	// AllowSmallOrderA allows signatures with a small order A.
	//
	// Note: This disables the check that makes the scheme strongly
	// binding.
	AllowSmallOrderA bool

	// AllowSmallOrder R allows signatures with a small order R.
	//
	// Note: Rejecting small order R is NOT required for binding.
	AllowSmallOrderR bool

	// AllowNonCanonicalA allows signatures with a non-canonical
	// encoding of A.
	AllowNonCanonicalA bool

	// AllowNonCanonicalR allows signatures with a non-canonical
	// encoding of R.
	//
	// Note: Setting this option is incompatible with CofactorlessVerify.
	AllowNonCanonicalR bool

	// CofactorlessVerify uses the cofactorless verification equation,
	// with the final comparison being done as a byte-compare between
	// the signature's R component and the canonical serialization of
	// the equation result (ref10 and derivative behavior).
	//
	// Note: Setting this option is incompatible with batch verification,
	// and is also incompatible with AllowNonCanonicalR.
	CofactorlessVerify bool
}

func (vOpts *VerifyOptions) unpackPublicKey(publicKey PublicKey, A *curve.EdwardsPoint) bool {
	// Unpack A.
	var aCompressed curve.CompressedEdwardsY
	if _, err := aCompressed.SetBytes(publicKey); err != nil {
		return false
	}
	if _, err := A.SetCompressedY(&aCompressed); err != nil {
		return false
	}

	// Check A order (required for strong binding).
	if !vOpts.AllowSmallOrderA && A.IsSmallOrder() {
		return false
	}

	// Check if A is canonical.
	if !vOpts.AllowNonCanonicalA && !aCompressed.IsCanonicalVartime() {
		return false
	}

	return true
}

func (vOpts *VerifyOptions) unpackSignature(sig []byte, R *curve.EdwardsPoint, S *scalar.Scalar) bool {
	if len(sig) != SignatureSize {
		return false
	}

	// https://tools.ietf.org/html/rfc8032#section-5.1.7 requires that s be in
	// the range [0, order) in order to prevent signature malleability.
	if !scalar.ScMinimalVartime(sig[32:]) {
		return false
	}

	// Unpack R.
	var rCompressed curve.CompressedEdwardsY
	if _, err := rCompressed.SetBytes(sig[:32]); err != nil {
		return false
	}
	if vOpts.verifyNeedsDecompressedR() {
		if _, err := R.SetCompressedY(&rCompressed); err != nil {
			return false
		}

		// Check R order.
		if !vOpts.AllowSmallOrderR && R.IsSmallOrder() {
			return false
		}
	} else {
		R.Identity()
		R = nil
	}

	// Check if R is canonical.
	if !vOpts.AllowNonCanonicalR && !rCompressed.IsCanonicalVartime() {
		return false
	}

	// Unpack S.
	if _, err := S.SetBytesModOrder(sig[32:]); err != nil {
		return false
	}

	return true
}

func (vOpts *VerifyOptions) verifyNeedsDecompressedR() bool {
	// The following conditions all require decompressed R.
	//  * Cofactorless verify
	//  * Batch verification
	//  * Reject small-order R
	//
	// As cofactorless verify is mutually exclusive with batch verification,
	// there is no need to know if we are doing batch verify to determine
	// if we need to do the decompression.

	if vOpts.CofactorlessVerify && vOpts.AllowSmallOrderR {
		return false
	}

	return true
}

// PrivateKey is the type of Ed25519 private keys. It implements crypto.Signer.
type PrivateKey []byte

// Public returns the PublicKey corresponding to priv.
func (priv PrivateKey) Public() crypto.PublicKey {
	pub := make([]byte, PublicKeySize)
	copy(pub, priv[SeedSize:])
	return PublicKey(pub)
}

// Equal reports whether priv and x have the same value. This function will
// execute in constant time.
func (priv PrivateKey) Equal(x crypto.PrivateKey) bool {
	xx, ok := x.(PrivateKey)
	if !ok {
		return false
	}
	return subtle.ConstantTimeCompareBytes(priv, xx) == 1
}

// Seed returns the private key seed corresponding to priv. It is provided for
// interoperability with RFC 8032. RFC 8032's private keys correspond to seeds
// in this package.
func (priv PrivateKey) Seed() []byte {
	s := make([]byte, SeedSize)
	copy(s, priv[:SeedSize])
	return s
}

// Sign signs the given message with priv.  If opts.HashFunc() is crypto.SHA512,
// the pre-hashed variant Ed25519ph is used and message is expected to be a
// SHA-512 hash, otherwise opts.HashFunc() must be crypto.Hash(0) and the
// message must not be hashed, as Ed25519 performs two passes over messages
// to be signed.
//
// If opts.AddRandomness is set, additional entropy from rand will be included.
// If rand is nil, crypto/rand.Reader will be used.
//
// Warning: This routine will panic if opts is nil.
func (priv PrivateKey) Sign(rand io.Reader, message []byte, opts crypto.SignerOpts) (signature []byte, err error) {
	var (
		context    []byte
		f          dom2Flag = fPure
		addedRand  bool
		selfVerify bool
	)
	if o, ok := opts.(*Options); ok {
		f, context, err = o.verify()
		if err != nil {
			return nil, err
		}

		addedRand = o.AddedRandomness
		if rand == nil {
			rand = cryptorand.Reader
		}

		selfVerify = o.SelfVerify
	}

	// Now that the Options specific validation is done, see if the caller
	// wants Ed25519ph instead.
	f, err = checkHash(f, message, opts.HashFunc())
	if err != nil {
		return nil, err
	}

	if l := len(priv); l != PrivateKeySize {
		return nil, fmt.Errorf("ed25519: bad private key length: %d", l)
	}

	var extsk [64]byte
	h := sha512.New()
	_, _ = h.Write(priv[:32])
	h.Sum(extsk[:0])
	extsk[0] &= 248
	extsk[31] &= 127
	extsk[31] |= 64

	// r = H(dom2, aExt[32..64], m)
	var (
		hashr [64]byte
		r     scalar.Scalar
	)
	h.Reset()
	dom2 := makeDom2(f, context)
	if dom2 != nil {
		_, _ = h.Write(dom2)
	}
	if addedRand {
		var entropy [addedRandomnessSize]byte
		if _, err := io.ReadFull(rand, entropy[:]); err != nil {
			return nil, fmt.Errorf("ed25519: failed to read Z: %w", err)
		}
		_, _ = h.Write(entropy[:])
	}
	_, _ = h.Write(extsk[32:])
	if addedRand {
		padSize := len(addedRandomnessPadding) - (len(dom2) + addedRandomnessSize + 32)
		_, _ = h.Write(addedRandomnessPadding[:padSize])
	}
	_, _ = h.Write(message)
	h.Sum(hashr[:0])
	if _, err = r.SetBytesModOrderWide(hashr[:]); err != nil {
		return nil, fmt.Errorf("ed25519: failed to deserialize r scalar: %w", err)
	}

	// R = rB
	var (
		R           curve.EdwardsPoint
		rCompressed curve.CompressedEdwardsY
	)
	rCompressed.SetEdwardsPoint(R.MulBasepoint(curve.ED25519_BASEPOINT_TABLE, &r))

	// S = H(R,A,m)
	var (
		hram [64]byte
		S    scalar.Scalar
	)
	h.Reset()
	if dom2 != nil {
		_, _ = h.Write(dom2)
	}
	_, _ = h.Write(rCompressed[:])
	_, _ = h.Write(priv[32:])
	_, _ = h.Write(message)
	h.Sum(hram[:0])
	if _, err = S.SetBytesModOrderWide(hram[:]); err != nil {
		return nil, fmt.Errorf("ed25519: failed to deserialize H(R,A,m) scalar: %w", err)
	}

	// S = H(R,A,m)a
	var a scalar.Scalar
	if _, err = a.SetBits(extsk[:32]); err != nil {
		return nil, fmt.Errorf("ed25519: failed to deserialize a scalar: %w", err)
	}
	S.Mul(&S, &a)

	// S = (r + H(R,A,m)a)
	S.Add(&S, &r)

	// S = (r + H(R,A,m)a) mod L
	var RS [SignatureSize]byte
	copy(RS[:32], rCompressed[:])
	if err = S.ToBytes(RS[32:]); err != nil {
		return nil, fmt.Errorf("ed25519: failed to serialize S scalar: %w", err)
	}

	// If opts.SelfVerify is set, verify the newly created signature.
	if selfVerify {
		if !VerifyWithOptions(PublicKey(priv[32:]), message, RS[:], opts.(*Options)) {
			return nil, fmt.Errorf("ed25519: failed to self-verify signature")
		}
	}

	return RS[:], nil
}

// PublicKey is the type of Ed25519 public keys.
type PublicKey []byte

// Any methods implemented on PublicKey might need to also be implemented on
// PrivateKey, as the latter embeds the former and will expose its methods.

// Equal reports whether pub and x have the same value, in variable-time.
func (pub PublicKey) Equal(x crypto.PublicKey) bool {
	xx, ok := x.(PublicKey)
	if !ok {
		return false
	}
	return bytes.Equal(pub, xx)
}

// Sign signs the message with privateKey and returns a signature. It will
// panic if len(privateKey) is not PrivateKeySize.
func Sign(privateKey PrivateKey, message []byte) []byte {
	// This would outline the function body to avoid a heap allocation,
	// but at least as of Go 1.16, something causes the escape analysis
	// to break.
	signature, err := privateKey.Sign(nil, message, optionsDefault)
	if err != nil {
		panic(err)
	}

	return signature
}

// Verify reports whether sig is a valid signature of message by publicKey. It
// will panic if len(publicKey) is not PublicKeySize.
func Verify(publicKey PublicKey, message, sig []byte) bool {
	return VerifyWithOptions(publicKey, message, sig, optionsDefault)
}

// VerifyWithOptions reports whether sig is a valid Ed25519 signature by
// publicKey with the extra Options to support Ed25519ph (pre-hashed by
// SHA-512) or Ed25519ctx (includes a domain separation context). It
// will panic if len(publicKey) is not PublicKeySize, len(message) is
// not sha512.Size (if pre-hashed), len(opts.Context) is greater than
// ContextMaxSize, or opts is nil.
func VerifyWithOptions(publicKey PublicKey, message, sig []byte, opts *Options) bool {
	// The standard library does this.
	if l := len(publicKey); l != PublicKeySize {
		panic("ed25519: bad public key length: " + strconv.Itoa(l))
	}

	ok, err := verifyWithOptionsNoPanic(publicKey, message, sig, opts)
	if err != nil {
		panic(err)
	}

	return ok
}

func verifyWithOptionsNoPanic(publicKey PublicKey, message, sig []byte, opts *Options) (bool, error) {
	f, context, err := opts.verify()
	if err != nil {
		return false, err
	}
	vOpts := opts.Verify
	if vOpts == nil {
		vOpts = VerifyOptionsDefault
	}

	// Now that the Options specific validation is done, see if the caller
	// wants Ed25519ph instead.
	f, err = checkHash(f, message, opts.HashFunc())
	if err != nil {
		return false, err
	}

	// Unpack and ensure the public key is well-formed (A).
	var A curve.EdwardsPoint
	if ok := vOpts.unpackPublicKey(publicKey, &A); !ok {
		return false, nil
	}

	// Unpack and ensure the signature is well-formed (R, S).
	var (
		checkR curve.EdwardsPoint
		S      scalar.Scalar
	)
	if ok := vOpts.unpackSignature(sig, &checkR, &S); !ok {
		return false, nil
	}

	// hram = H(R,A,m)
	var (
		hash [64]byte
		hram scalar.Scalar
	)
	h := sha512.New()
	if dom2 := makeDom2(f, context); dom2 != nil {
		_, _ = h.Write(dom2)
	}
	_, _ = h.Write(sig[:32])
	_, _ = h.Write(publicKey[:])
	_, _ = h.Write(message)
	h.Sum(hash[:0])
	if _, err = hram.SetBytesModOrderWide(hash[:]); err != nil {
		return false, fmt.Errorf("ed25519: failed to deserialize H(R,A,m) scalar: %w", err)
	}

	// A = -A (Since we want SB - H(R,A,m)A)
	A.Neg(&A)

	// There are 2 ways of verifying Ed25519 signatures in the wild, due
	// to the original paper/implementation, RFC, and FIPS draft.
	//
	// For the purpose of compatibility, support the old way of doing
	// things, though this is now considered unwise.
	if vOpts.CofactorlessVerify {
		// SB - H(R,A,m)A ?= R
		var R curve.EdwardsPoint
		R.DoubleScalarMulBasepointVartime(&hram, &A, &S)
		return cofactorlessVerify(&R, sig), nil
	}

	// Check that [8]R == [8](SB - H(R,A,m)A)), by computing
	// [delta S]B - [delta A]H(R,A,m) - [delta]R, multiplying the
	// result by the cofactor, and checking if the result is
	// small order.
	//
	// Note: IsSmallOrder includes a cofactor multiply.
	var rDiff curve.EdwardsPoint
	return rDiff.TripleScalarMulBasepointVartime(&hram, &A, &S, &checkR).IsSmallOrder(), nil
}

// NewKeyFromSeed calculates a private key from a seed. It will panic if
// len(seed) is not SeedSize. This function is provided for interoperability
// with RFC 8032. RFC 8032's private keys correspond to seeds in this
// package.
func NewKeyFromSeed(seed []byte) PrivateKey {
	// Outline the function body so that the returned key can be stack-allocated.
	privateKey := make([]byte, PrivateKeySize)
	newKeyFromSeed(privateKey, seed)
	return privateKey
}

func newKeyFromSeed(privateKey, seed []byte) {
	if l := len(seed); l != SeedSize {
		panic("ed25519: bad seed length: " + strconv.Itoa(l))
	}

	digest := sha512.Sum512(seed)
	digest[0] &= 248
	digest[31] &= 127
	digest[31] |= 64

	var a scalar.Scalar
	if _, err := a.SetBits(digest[:32]); err != nil {
		panic("ed25519: failed to deserialize scalar: " + err.Error())
	}

	var (
		A           curve.EdwardsPoint
		aCompressed curve.CompressedEdwardsY
	)
	aCompressed.SetEdwardsPoint(A.MulBasepoint(curve.ED25519_BASEPOINT_TABLE, &a))

	copy(privateKey, seed)
	copy(privateKey[32:], aCompressed[:])
}

// GenerateKey generates a public/private key pair using entropy from rand.
// If rand is nil, crypto/rand.Reader will be used.
func GenerateKey(rand io.Reader) (PublicKey, PrivateKey, error) {
	if rand == nil {
		rand = cryptorand.Reader
	}

	seed := make([]byte, SeedSize)
	if _, err := io.ReadFull(rand, seed); err != nil {
		return nil, nil, err
	}

	privateKey := NewKeyFromSeed(seed)
	publicKey := make([]byte, PublicKeySize)
	copy(publicKey, privateKey[32:])

	return publicKey, privateKey, nil
}

type dom2Flag byte

const (
	fCtx  dom2Flag = 0
	fPh   dom2Flag = 1
	fPure dom2Flag = 255 // Not in RFC, for implementation purposes.

	dom2Prefix = "SigEd25519 no Ed25519 collisions"
)

func makeDom2(f dom2Flag, c []byte) []byte {
	if f == fPure {
		return nil
	}

	cLen := len(c)
	if cLen > ContextMaxSize {
		panic("ed25519: bad context length: " + strconv.Itoa(cLen))
	}

	dLen := len(dom2Prefix) + 1 + 1 + cLen

	b := make([]byte, 0, dLen)
	b = append(b, dom2Prefix...)
	b = append(b, byte(f))
	b = append(b, byte(cLen))
	b = append(b, c...)

	return b
}

func cofactorlessVerify(R *curve.EdwardsPoint, sig []byte) bool {
	// This could instead do `R ?= canonicalized sig` to support
	// AllowNonCanonicalR with CofactorlessVerify.
	//
	// However, as far as I am aware, there is no commonly used
	// implementation in the wild that is both cofactor-less and
	// accepts non-canonical R (ref10 and derivatives will reject
	// the latter).
	//
	// For now just assume that anyone explicitly wanting to use
	// cofactorless verification wants interoperability with ref10.
	var RCompressed curve.CompressedEdwardsY
	RCompressed.SetEdwardsPoint(R)

	return bytes.Equal(RCompressed[:], sig[:32])
}
