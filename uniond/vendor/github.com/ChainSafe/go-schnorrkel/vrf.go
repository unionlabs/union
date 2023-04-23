package schnorrkel

import (
	"github.com/gtank/merlin"
	r255 "github.com/gtank/ristretto255"
)

type VrfInOut struct {
	input  *r255.Element
	output *r255.Element
}

type VrfOutput struct {
	output *r255.Element
}

type VrfProof struct {
	c *r255.Scalar
	s *r255.Scalar
}

// Output returns a VrfOutput from a VrfInOut
func (io *VrfInOut) Output() *VrfOutput {
	return &VrfOutput{
		output: io.output,
	}
}

// EncodeOutput returns the 64-byte encoding of the input and output concatenated
func (io *VrfInOut) Encode() []byte {
	outbytes := [32]byte{}
	copy(outbytes[:], io.output.Encode([]byte{}))
	inbytes := [32]byte{}
	copy(inbytes[:], io.input.Encode([]byte{}))
	return append(inbytes[:], outbytes[:]...)
}

// NewOutput creates a new VRF output from a 64-byte element
func NewOutput(in [32]byte) *VrfOutput {
	output := r255.NewElement()
	output.Decode(in[:])
	return &VrfOutput{
		output: output,
	}
}

// AttachInput returns a VrfInOut pair from an output
func (out *VrfOutput) AttachInput(pub *PublicKey, t *merlin.Transcript) *VrfInOut {
	input := pub.vrfHash(t)
	return &VrfInOut{
		input:  input,
		output: out.output,
	}
}

// Encode returns the 32-byte encoding of the output
func (out *VrfOutput) Encode() [32]byte {
	outbytes := [32]byte{}
	copy(outbytes[:], out.output.Encode([]byte{}))
	return outbytes
}

// Decode sets the VrfOutput to the decoded input
func (out *VrfOutput) Decode(in [32]byte) error {
	output := r255.NewElement()
	err := output.Decode(in[:])
	if err != nil {
		return err
	}
	out.output = output
	return nil
}

// Encode returns a 64-byte encoded VrfProof
func (p *VrfProof) Encode() [64]byte {
	cbytes := [32]byte{}
	copy(cbytes[:], p.c.Encode([]byte{}))
	sbytes := [32]byte{}
	copy(sbytes[:], p.s.Encode([]byte{}))
	enc := [64]byte{}
	copy(enc[:32], cbytes[:])
	copy(enc[32:], sbytes[:])
	return enc
}

// Decode sets the VrfProof to the decoded input
func (p *VrfProof) Decode(in [64]byte) error {
	c := r255.NewScalar()
	err := c.Decode(in[:32])
	if err != nil {
		return err
	}
	p.c = c

	s := r255.NewScalar()
	err = s.Decode(in[32:])
	if err != nil {
		return err
	}
	p.s = s

	return nil
}

// VrfSign returns a vrf output and proof given a secret key and transcript.
func (sk *SecretKey) VrfSign(t *merlin.Transcript) (*VrfInOut, *VrfProof, error) {
	p, err := sk.vrfCreateHash(t)
	if err != nil {
		return nil, nil, err
	}

	t0 := merlin.NewTranscript("VRF")
	proof, err := sk.dleqProve(t0, p)
	if err != nil {
		return nil, nil, err
	}
	return p, proof, nil
}

// dleqProve creates a VRF proof for the transcript and input with this secret key.
// see: https://github.com/w3f/schnorrkel/blob/798ab3e0813aa478b520c5cf6dc6e02fd4e07f0a/src/vrf.rs#L604
func (sk *SecretKey) dleqProve(t *merlin.Transcript, p *VrfInOut) (*VrfProof, error) {
	t.AppendMessage([]byte("proto-name"), []byte("DLEQProof"))
	t.AppendMessage([]byte("vrf:h"), p.input.Encode([]byte{}))

	// create random element R = g^r
	r, err := NewRandomScalar()
	if err != nil {
		return nil, err
	}
	R := r255.NewElement()
	R.ScalarBaseMult(r)
	t.AppendMessage([]byte("vrf:R=g^r"), R.Encode([]byte{}))

	// create hr := HashToElement(input)
	hr := r255.NewElement().ScalarMult(r, p.input).Encode([]byte{})
	t.AppendMessage([]byte("vrf:h^r"), hr)

	pub, err := sk.Public()
	if err != nil {
		return nil, err
	}
	pubenc := pub.Encode()
	t.AppendMessage([]byte("vrf:pk"), pubenc[:])
	t.AppendMessage([]byte("vrf:h^sk"), p.output.Encode([]byte{}))

	c := challengeScalar(t, []byte("prove"))
	s := r255.NewScalar()
	sc, err := ScalarFromBytes(sk.key)
	if err != nil {
		return nil, err
	}
	s.Subtract(r, r255.NewScalar().Multiply(c, sc))

	return &VrfProof{
		c: c,
		s: s,
	}, nil
}

// vrfCreateHash creates a VRF input/output pair on the given transcript.
func (sk *SecretKey) vrfCreateHash(t *merlin.Transcript) (*VrfInOut, error) {
	pub, err := sk.Public()
	if err != nil {
		return nil, err
	}
	input := pub.vrfHash(t)

	output := r255.NewElement()
	sc := r255.NewScalar()
	err = sc.Decode(sk.key[:])
	if err != nil {
		return nil, err
	}
	output.ScalarMult(sc, input)

	return &VrfInOut{
		input:  input,
		output: output,
	}, nil
}

// VrfVerify verifies that the proof and output created are valid given the public key and transcript.
func (pk *PublicKey) VrfVerify(t *merlin.Transcript, inout *VrfInOut, proof *VrfProof) (bool, error) {
	t0 := merlin.NewTranscript("VRF")
	return pk.dleqVerify(t0, inout, proof)
}

// dleqVerify verifies the corresponding dleq proof.
func (pk *PublicKey) dleqVerify(t *merlin.Transcript, p *VrfInOut, proof *VrfProof) (bool, error) {
	t.AppendMessage([]byte("proto-name"), []byte("DLEQProof"))
	t.AppendMessage([]byte("vrf:h"), p.input.Encode([]byte{}))

	// R = proof.c*pk + proof.s*g
	R := r255.NewElement()
	R.VarTimeDoubleScalarBaseMult(proof.c, pk.key, proof.s)
	t.AppendMessage([]byte("vrf:R=g^r"), R.Encode([]byte{}))

	// hr = proof.c * p.output + proof.s * p.input
	hr := r255.NewElement().VarTimeMultiScalarMult([]*r255.Scalar{proof.c, proof.s}, []*r255.Element{p.output, p.input})
	t.AppendMessage([]byte("vrf:h^r"), hr.Encode([]byte{}))
	t.AppendMessage([]byte("vrf:pk"), pk.key.Encode([]byte{}))
	t.AppendMessage([]byte("vrf:h^sk"), p.output.Encode([]byte{}))

	cexpected := challengeScalar(t, []byte("prove"))
	if cexpected.Equal(proof.c) == 1 {
		return true, nil
	}

	return false, nil
}

// vrfHash hashes the transcript to a point.
func (pk *PublicKey) vrfHash(t *merlin.Transcript) *r255.Element {
	mt := TranscriptWithMalleabilityAddressed(t, pk)
	hash := mt.ExtractBytes([]byte("VRFHash"), 64)
	point := r255.NewElement()
	point.FromUniformBytes(hash)
	return point
}

// TranscriptWithMalleabilityAddressed returns the input transcript with the public key commited to it,
// addressing VRF output malleability.
func TranscriptWithMalleabilityAddressed(t *merlin.Transcript, pk *PublicKey) *merlin.Transcript {
	enc := pk.Encode()
	t.AppendMessage([]byte("vrf-nm-pk"), enc[:])
	return t
}
