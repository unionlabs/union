package schnorrkel

import (
	"github.com/gtank/merlin"
	r255 "github.com/gtank/ristretto255"
)

// Signature holds a schnorrkel signature
type Signature struct {
	R *r255.Element
	S *r255.Scalar
}

// NewSigningContext returns a new transcript initialized with the context for the signature
//.see: https://github.com/w3f/schnorrkel/blob/db61369a6e77f8074eb3247f9040ccde55697f20/src/context.rs#L183
func NewSigningContext(context, msg []byte) *merlin.Transcript {
	t := merlin.NewTranscript("SigningContext")
	t.AppendMessage([]byte(""), context)
	t.AppendMessage([]byte("sign-bytes"), msg)
	return t
}

// Sign uses the schnorr signature algorithm to sign a message
// See the following for the transcript message
// https://github.com/w3f/schnorrkel/blob/db61369a6e77f8074eb3247f9040ccde55697f20/src/sign.rs#L158
// Schnorr w/ transcript, secret key x:
// 1. choose random r from group
// 2. R = gr
// 3. k = scalar(transcript.extract_bytes())
// 4. s = kx + r
// signature: (R, s)
// public key used for verification: y = g^x
func (sk *SecretKey) Sign(t *merlin.Transcript) (*Signature, error) {
	t.AppendMessage([]byte("proto-name"), []byte("Schnorr-sig"))

	pub, err := sk.Public()
	if err != nil {
		return nil, err
	}
	pubc := pub.Compress()

	t.AppendMessage([]byte("sign:pk"), pubc[:])

	// note: TODO: merlin library doesn't have build_rng yet.
	// see https://github.com/w3f/schnorrkel/blob/798ab3e0813aa478b520c5cf6dc6e02fd4e07f0a/src/context.rs#L153
	// r := t.ExtractBytes([]byte("signing"), 32)

	// choose random r (nonce)
	r, err := NewRandomScalar()
	if err != nil {
		return nil, err
	}
	R := r255.NewElement().ScalarBaseMult(r)
	t.AppendMessage([]byte("sign:R"), R.Encode([]byte{}))

	// form k
	kb := t.ExtractBytes([]byte("sign:c"), 64)
	k := r255.NewScalar()
	k.FromUniformBytes(kb)

	// form scalar from secret key x
	x, err := ScalarFromBytes(sk.key)
	if err != nil {
		return nil, err
	}

	// s = kx + r
	s := x.Multiply(x, k).Add(x, r)

	return &Signature{R: R, S: s}, nil
}

// Verify verifies a schnorr signature with format: (R, s) where y is the public key
// 1. k = scalar(transcript.extract_bytes())
// 2. R' = -ky + gs
// 3. return R' == R
func (p *PublicKey) Verify(s *Signature, t *merlin.Transcript) bool {
	t.AppendMessage([]byte("proto-name"), []byte("Schnorr-sig"))
	pubc := p.Compress()
	t.AppendMessage([]byte("sign:pk"), pubc[:])
	t.AppendMessage([]byte("sign:R"), s.R.Encode([]byte{}))

	kb := t.ExtractBytes([]byte("sign:c"), 64)
	k := r255.NewScalar()
	k.FromUniformBytes(kb)

	Rp := r255.NewElement()
	Rp = Rp.ScalarBaseMult(s.S)
	ky := r255.NewElement().ScalarMult(k, p.key)
	Rp = Rp.Subtract(Rp, ky)

	return Rp.Equal(s.R) == 1
}

// Decode sets a Signature from bytes
// see: https://github.com/w3f/schnorrkel/blob/db61369a6e77f8074eb3247f9040ccde55697f20/src/sign.rs#L100
func (s *Signature) Decode(in [64]byte) error {
	s.R = r255.NewElement()
	err := s.R.Decode(in[:32])
	if err != nil {
		return err
	}
	in[63] &= 127
	s.S = r255.NewScalar()
	return s.S.Decode(in[32:])
}

// Encode turns a signature into a byte array
// see: https://github.com/w3f/schnorrkel/blob/db61369a6e77f8074eb3247f9040ccde55697f20/src/sign.rs#L77
func (s *Signature) Encode() [64]byte {
	out := [64]byte{}
	renc := s.R.Encode([]byte{})
	copy(out[:32], renc)
	senc := s.S.Encode([]byte{})
	copy(out[32:], senc)
	out[63] |= 128
	return out
}
