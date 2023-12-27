// Copyright (c) 2017-2019 isis agora lovecruft. All rights reserved.
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
	"fmt"
	"io"

	"github.com/oasisprotocol/curve25519-voi/curve"
	"github.com/oasisprotocol/curve25519-voi/curve/scalar"
)

const (
	// SignatureSize is the size of a sr25519 signature in bytes.
	SignatureSize = 64

	protoLabel = "Schnorr-sig"
	aLabel     = "sign:pk"
	rLabel     = "sign:R"
	cLabel     = "sign:c"

	witnessScalarLabel = "signing"
)

var errSignatureNotMarkedSchnorrkel = fmt.Errorf("sr25519: not a sr25519 signature")

func markSignatureSchnorrkel(b []byte) {
	b[63] |= 128
}

type Signature struct {
	rCompressed curve.CompressedRistretto
	s           *scalar.Scalar
}

// UnmarshalBinary decodes a binary marshaled Signature.
func (sig *Signature) UnmarshalBinary(data []byte) error {
	sig.rCompressed.Identity()
	sig.s = nil

	if l := len(data); l != SignatureSize {
		return fmt.Errorf("sr25519: bad Signature size: %v", l)
	}

	// Check that the upper-most bit is set (to distinguish sr25519
	// signatures from Ed25519 signatures).
	if data[63]&128 == 0 {
		return errSignatureNotMarkedSchnorrkel
	}

	// Copy and clear the upper-most bit.
	var upper [scalar.ScalarSize]byte
	copy(upper[:], data[32:])
	upper[31] &= 127

	// Check that the scalar is encoded in canonical form.
	if !scalar.ScMinimalVartime(upper[:]) {
		return fmt.Errorf("sr25519: non-canonical signature scalar")
	}
	sigScalar, err := scalar.NewFromCanonicalBytes(upper[:])
	if err != nil {
		return fmt.Errorf("sr25519: failed to deserialize signature scalar: %v", err)
	}

	// Copy (but do not decompress) the point.
	if _, err := sig.rCompressed.SetBytes(data[:32]); err != nil {
		return fmt.Errorf("sr25519: failed to deserialize signature point: %v", err)
	}

	sig.s = sigScalar

	return nil
}

// MarshalBinary encodes a Signature into binary form.
func (sig *Signature) MarshalBinary() ([]byte, error) {
	var b []byte

	switch sig.s {
	case nil:
		b = make([]byte, SignatureSize)
	default:
		b = make([]byte, 0, SignatureSize)
		b = append(b, sig.rCompressed[:]...)

		scalarBytes, err := sig.s.MarshalBinary()
		if err != nil {
			return nil, fmt.Errorf("sr25519: failed to serialize signature scalar: %v", err)
		}

		b = append(b, scalarBytes...)
	}

	markSignatureSchnorrkel(b)

	return b, nil
}

// NewSignatureFromBytes constructs a Signature from the byte representation.
func NewSignatureFromBytes(b []byte) (*Signature, error) {
	var s Signature
	if err := s.UnmarshalBinary(b); err != nil {
		return nil, err
	}
	return &s, nil
}

func deriveVerifyChallengeScalar(publicKey *PublicKey, transcript *SigningTranscript, signature *Signature) *scalar.Scalar {
	t := transcript.clone()
	t.protoName(protoLabel)
	t.commitPoint(aLabel, &publicKey.compressed)
	t.commitPoint(rLabel, &signature.rCompressed)
	return t.challengeScalar(cLabel)
}

// Verify verifies a signature by a public key on a transcript.
func (pk *PublicKey) Verify(transcript *SigningTranscript, signature *Signature) bool {
	if pk.point == nil || signature.s == nil {
		return false
	}

	var r curve.RistrettoPoint
	if _, err := r.SetCompressed(&signature.rCompressed); err != nil {
		return false
	}

	var negA curve.RistrettoPoint
	negA.Neg(pk.point)

	k := deriveVerifyChallengeScalar(pk, transcript, signature)

	var rDiff curve.RistrettoPoint
	return rDiff.TripleScalarMulBasepointVartime(k, &negA, signature.s, &r).IsIdentity()
}

// Sign signs a transcript with a key pair, and provided entropy source.
// If rng is nil, crypto/rand.Reader will be used.
func (kp *KeyPair) Sign(rng io.Reader, transcript *SigningTranscript) (*Signature, error) {
	t := transcript.clone()
	t.protoName(protoLabel)
	t.commitPoint(aLabel, &kp.pk.compressed)

	rScalar, err := t.witnessScalar(witnessScalarLabel, [][]byte{kp.sk.nonce[:]}, rng)
	if err != nil {
		return nil, fmt.Errorf("sr25519: failed to generate witness scalar: %w", err)
	}

	var (
		sig Signature
		r   curve.RistrettoPoint
	)
	r.MulBasepoint(curve.RISTRETTO_BASEPOINT_TABLE, rScalar)
	sig.rCompressed.SetRistrettoPoint(&r)

	t.commitPoint(rLabel, &sig.rCompressed)

	k := t.challengeScalar(cLabel)

	sig.s = scalar.New().Mul(k, kp.sk.key)
	sig.s.Add(sig.s, rScalar)

	return &sig, nil
}
