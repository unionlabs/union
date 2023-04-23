package ed25519consensus

import (
	"crypto/ed25519"
	"crypto/rand"
	"crypto/sha512"

	"filippo.io/edwards25519"
)

// BatchVerifier accumulates batch entries with Add, before performing batch verification with Verify.
type BatchVerifier struct {
	entries []entry
}

// entry represents a batch entry with the public key, signature and scalar which the caller wants to verify
type entry struct {
	pubkey    ed25519.PublicKey
	signature []byte
	k         *edwards25519.Scalar
}

// NewBatchVerifier creates an empty BatchVerifier.
func NewBatchVerifier() BatchVerifier {
	return BatchVerifier{
		entries: []entry{},
	}
}

// Add adds a (public key, message, sig) triple to the current batch.
func (v *BatchVerifier) Add(publicKey ed25519.PublicKey, message, sig []byte) {
	// Compute the challenge scalar for this entry upfront, so that we don't
	// introduce a dependency on the lifetime of the message array. This doesn't
	// matter so much for Go, which has garbage collection, but did matter for
	// the Rust implementation this was ported from, but not keeping buffers
	// alive for longer than they have to is nice to do anyways.

	h := sha512.New()

	// R_bytes is the first 32 bytes of the signature, but because the signature
	// is passed as a variable-length array it could be too short. In that case
	// we'll fail in Verify, so just avoid a panic here.
	n := 32
	if len(sig) < n {
		n = len(sig)
	}
	h.Write(sig[:n])

	h.Write(publicKey)
	h.Write(message)
	var digest [64]byte
	h.Sum(digest[:0])

	k, err := new(edwards25519.Scalar).SetUniformBytes(digest[:])
	if err != nil {
		panic(err)
	}

	e := entry{
		pubkey:    publicKey,
		signature: sig,
		k:         k,
	}

	v.entries = append(v.entries, e)
}

// Verify checks all entries in the current batch, returning true if all entries
// are valid and false if any one entry is invalid.
//
// If a failure arises it is unknown which entry failed, the caller must verify
// each entry individually.
//
// Calling Verify on an empty batch returns false.
func (v *BatchVerifier) Verify() bool {
	vl := len(v.entries)
	// Abort early on an empty batch, which probably indicates a bug
	if vl == 0 {
		return false
	}

	// The batch verification equation is
	//
	// [-sum(z_i * s_i)]B + sum([z_i]R_i) + sum([z_i * k_i]A_i) = 0.
	// where for each signature i,
	// - A_i is the verification key;
	// - R_i is the signature's R value;
	// - s_i is the signature's s value;
	// - k_i is the hash of the message and other data;
	// - z_i is a random 128-bit Scalar.
	svals := make([]edwards25519.Scalar, 1+vl+vl)
	scalars := make([]*edwards25519.Scalar, 1+vl+vl)

	// Populate scalars variable with concrete scalars to reduce heap allocation
	for i := range scalars {
		scalars[i] = &svals[i]
	}

	Bcoeff := scalars[0]
	Rcoeffs := scalars[1:][:int(vl)]
	Acoeffs := scalars[1+vl:]

	pvals := make([]edwards25519.Point, 1+vl+vl)
	points := make([]*edwards25519.Point, 1+vl+vl)
	for i := range points {
		points[i] = &pvals[i]
	}
	B := points[0]
	Rs := points[1:][:vl]
	As := points[1+vl:]

	B.Set(edwards25519.NewGeneratorPoint())
	for i, entry := range v.entries {
		// Check that the signature is exactly 64 bytes upfront,
		// so that we can slice it later without potential panics
		if len(entry.signature) != 64 {
			return false
		}

		if _, err := Rs[i].SetBytes(entry.signature[:32]); err != nil {
			return false
		}

		if _, err := As[i].SetBytes(entry.pubkey); err != nil {
			return false
		}

		buf := make([]byte, 32)
		rand.Read(buf[:16])
		_, err := Rcoeffs[i].SetCanonicalBytes(buf)
		if err != nil {
			return false
		}

		s, err := new(edwards25519.Scalar).SetCanonicalBytes(entry.signature[32:])
		if err != nil {
			return false
		}
		Bcoeff.MultiplyAdd(Rcoeffs[i], s, Bcoeff)

		Acoeffs[i].Multiply(Rcoeffs[i], entry.k)
	}
	Bcoeff.Negate(Bcoeff) // this term is subtracted in the summation

	check := new(edwards25519.Point).VarTimeMultiScalarMult(scalars, points)
	check.MultByCofactor(check)
	return check.Equal(edwards25519.NewIdentityPoint()) == 1
}
