// Copyright 2019 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package ristretto255

import (
	"encoding/base64"
	"github.com/gtank/ristretto255/internal/scalar"
)

// A Scalar is an element of the ristretto255 scalar field, as specified in
// draft-hdevalence-cfrg-ristretto-01, Section 3.4. That is, an integer modulo
//
//     l = 2^252 + 27742317777372353535851937790883648493
type Scalar struct {
	s scalar.Scalar
}

// NewScalar returns a Scalar set to the value 0.
func NewScalar() *Scalar {
	return (&Scalar{}).Zero()
}

// Add sets s = x + y mod l and returns s.
func (s *Scalar) Add(x, y *Scalar) *Scalar {
	s.s.Add(&x.s, &y.s)
	return s
}

// Subtract sets s = x - y mod l and returns s.
func (s *Scalar) Subtract(x, y *Scalar) *Scalar {
	s.s.Sub(&x.s, &y.s)
	return s
}

// Negate sets s = -x mod l and returns s.
func (s *Scalar) Negate(x *Scalar) *Scalar {
	s.s.Neg(&x.s)
	return s
}

// Multiply sets s = x * y mod l and returns s.
func (s *Scalar) Multiply(x, y *Scalar) *Scalar {
	s.s.Mul(&x.s, &y.s)
	return s
}

// Invert sets s = 1 / x such that s * x = 1 mod l and returns s.
//
// If x is 0, the result is undefined.
func (s *Scalar) Invert(x *Scalar) *Scalar {
	s.s.Inv(&x.s)
	return s
}

// FromUniformBytes sets s to an uniformly distributed value given 64 uniformly
// distributed random bytes.
func (s *Scalar) FromUniformBytes(x []byte) *Scalar {
	s.s.FromUniformBytes(x)
	return s
}

// Decode sets s = x, where x is a 32 bytes little-endian encoding of s. If x is
// not a canonical encoding of s, Decode returns an error and the receiver is
// unchanged.
func (s *Scalar) Decode(x []byte) error {
	return s.s.FromCanonicalBytes(x)
}

// Encode appends a 32 bytes little-endian encoding of s to b.
func (s *Scalar) Encode(b []byte) []byte {
	return s.s.Bytes(b)
}

// Equal returns 1 if v and u are equal, and 0 otherwise.
func (s *Scalar) Equal(u *Scalar) int {
	return s.s.Equal(&u.s)
}

// Zero sets s = 0 and returns s.
func (s *Scalar) Zero() *Scalar {
	s.s = scalar.Scalar{}
	return s
}

// MarshalText implements encoding/TextMarshaler interface
func (s *Scalar) MarshalText() (text []byte, err error) {
	b := s.Encode([]byte{})
	return []byte(base64.StdEncoding.EncodeToString(b)), nil
}

// UnmarshalText implements encoding/TextMarshaler interface
func (s *Scalar) UnmarshalText(text []byte) error {
	sb, err := base64.StdEncoding.DecodeString(string(text))
	if err == nil {
		return s.Decode(sb)
	}
	return err
}

// String implements the Stringer interface
func (s *Scalar) String() string {
	result, _ := s.MarshalText()
	return string(result)
}
