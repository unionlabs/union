package schnorrkel

import (
	"crypto/rand"

	"github.com/gtank/merlin"
	r255 "github.com/gtank/ristretto255"
)

func challengeScalar(t *merlin.Transcript, msg []byte) *r255.Scalar {
	scb := t.ExtractBytes(msg, 64)
	sc := r255.NewScalar()
	sc.FromUniformBytes(scb)
	return sc
}

// https://github.com/w3f/schnorrkel/blob/718678e51006d84c7d8e4b6cde758906172e74f8/src/scalars.rs#L18
func divideScalarByCofactor(s []byte) []byte {
	l := len(s) - 1
	low := byte(0)
	for i := range s {
		r := s[l-i] & 0b00000111 // remainder
		s[l-i] >>= 3
		s[l-i] += low
		low = r << 5
	}

	return s
}

// NewRandomElement returns a random ristretto element
func NewRandomElement() (*r255.Element, error) {
	e := r255.NewElement()
	s := [64]byte{}
	_, err := rand.Read(s[:])
	if err != nil {
		return nil, err
	}

	return e.FromUniformBytes(s[:]), nil
}

// NewRandomScalar returns a random ristretto scalar
func NewRandomScalar() (*r255.Scalar, error) {
	s := [64]byte{}
	_, err := rand.Read(s[:])
	if err != nil {
		return nil, err
	}

	ss := r255.NewScalar()
	return ss.FromUniformBytes(s[:]), nil
}

// ScalarFromBytes returns a ristretto scalar from the input bytes
// performs input mod l where l is the group order
func ScalarFromBytes(b [32]byte) (*r255.Scalar, error) {
	s := r255.NewScalar()
	err := s.Decode(b[:])
	if err != nil {
		return nil, err
	}

	return s, nil
}
