package ed25519consensus

import (
	"crypto/ed25519"
	"crypto/rand"
	"crypto/sha512"

	"filippo.io/edwards25519"
)

// BatchVerifier accumulates batch entries with Add, before performing batch
// verification with Verify.
type BatchVerifier struct {
	entries []entry
}

// entry represents a batch entry with the public key, signature and scalar
// which the caller wants to verify.
type entry struct {
	good      bool // good is true if the Add inputs were valid
	pubkey    [ed25519.PublicKeySize]byte
	signature [ed25519.SignatureSize]byte
	digest    [64]byte
}

// NewBatchVerifier creates an empty BatchVerifier.
func NewBatchVerifier() BatchVerifier {
	return BatchVerifier{
		entries: []entry{},
	}
}

// NewPreallocatedBatchVerifier creates a new BatchVerifier with
// a preallocated capacity. If you know the size of the batch you plan
// to create ahead of time, this can prevent needless memory copies.
func NewPreallocatedBatchVerifier(size int) BatchVerifier {
	return BatchVerifier{
		entries: make([]entry, 0, size),
	}
}

// Add adds a (public key, message, sig) triple to the current batch. It retains
// no reference to the inputs.
func (v *BatchVerifier) Add(publicKey ed25519.PublicKey, message, sig []byte) {
	// Compute the challenge upfront to store it in the fixed-size entry
	// structure that can get allocated on the caller stack and avoid heap
	// allocations. Also, avoid holding any reference to the arguments.

	v.entries = append(v.entries, entry{})
	e := &v.entries[len(v.entries)-1]

	if len(publicKey) != ed25519.PublicKeySize || len(sig) != ed25519.SignatureSize {
		return
	}

	h := sha512.New()
	h.Write(sig[:32])
	h.Write(publicKey)
	h.Write(message)
	h.Sum(e.digest[:0])

	copy(e.pubkey[:], publicKey)
	copy(e.signature[:], sig)

	e.good = true
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
	Rcoeffs := scalars[1 : 1+vl]
	Acoeffs := scalars[1+vl:]

	pvals := make([]edwards25519.Point, 1+vl+vl)
	points := make([]*edwards25519.Point, 1+vl+vl)
	for i := range points {
		points[i] = &pvals[i]
	}
	B := points[0]
	Rs := points[1 : 1+vl]
	As := points[1+vl:]

	buf := make([]byte, 32)
	B.Set(edwards25519.NewGeneratorPoint())
	for i, entry := range v.entries {
		if !entry.good {
			return false
		}

		if _, err := Rs[i].SetBytes(entry.signature[:32]); err != nil {
			return false
		}

		if _, err := As[i].SetBytes(entry.pubkey[:]); err != nil {
			return false
		}

		if _, err := rand.Read(buf[:16]); err != nil {
			return false
		}
		if _, err := Rcoeffs[i].SetCanonicalBytes(buf); err != nil {
			return false
		}

		s, err := new(edwards25519.Scalar).SetCanonicalBytes(entry.signature[32:])
		if err != nil {
			return false
		}
		Bcoeff.MultiplyAdd(Rcoeffs[i], s, Bcoeff)

		k, err := new(edwards25519.Scalar).SetUniformBytes(entry.digest[:])
		if err != nil {
			return false
		}
		Acoeffs[i].Multiply(Rcoeffs[i], k)
	}
	Bcoeff.Negate(Bcoeff) // this term is subtracted in the summation

	check := new(edwards25519.Point).VarTimeMultiScalarMult(scalars, points)
	check.MultByCofactor(check)
	return check.Equal(edwards25519.NewIdentityPoint()) == 1
}
