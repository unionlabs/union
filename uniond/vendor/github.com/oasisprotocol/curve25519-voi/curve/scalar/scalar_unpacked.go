// Copyright (c) 2016-2019 isis agora lovecruft. All rights reserved.
// Copyright (c) 2016-2019 Henry de Valence. All rights reserved.
// Copyright (c) 2020-2021 Oasis Labs Inc. All rights reserved.
// Portions Copyright 2017 Brian Smith.
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

package scalar

func newUnpackedScalar() *unpackedScalar {
	return &unpackedScalar{}
}

// Mul sets `s = a * b (mod l)`, and returns s.
func (s *unpackedScalar) Mul(a, b *unpackedScalar) *unpackedScalar {
	limbs := scalarMulInternal(a, b)
	s.MontgomeryReduce(&limbs)
	limbs = scalarMulInternal(s, &constRR)
	return s.MontgomeryReduce(&limbs)
}

// Square sets `s = a^2` (mod l)`, and returns s.
func (s *unpackedScalar) Square(a *unpackedScalar) *unpackedScalar {
	limbs := a.squareInternal()
	s.MontgomeryReduce(&limbs)
	limbs = scalarMulInternal(s, &constRR)
	return s.MontgomeryReduce(&limbs)
}

// MontgomeryMul sets `s = (a * b) / R (mod l)`, where R is the Montgomery
// modulus, and returns s.
func (s *unpackedScalar) MontgomeryMul(a, b *unpackedScalar) *unpackedScalar {
	limbs := scalarMulInternal(a, b)
	return s.MontgomeryReduce(&limbs)
}

// MontgomerySquare sets `s = (a^2) / R` (mod l), where R is the Montgomery
// modulus, and returns s.
func (s *unpackedScalar) MontgomerySquare(a *unpackedScalar) *unpackedScalar {
	limbs := a.squareInternal()
	return s.MontgomeryReduce(&limbs)
}

// ToMontgomery puts the scalar in to Montgomery form, i.e. computes `a*R (mod l)`,
// and returns s.
func (s *unpackedScalar) ToMontgomery(a *unpackedScalar) *unpackedScalar {
	return s.MontgomeryMul(a, &constRR)
}

// MontgomeryInvert inverts an unpackedScalar in Montgomery form.
func (s *unpackedScalar) MontgomeryInvert() {
	// Uses the addition chain from
	// https://briansmith.org/ecc-inversion-addition-chains-01#curve25519_scalar_inversion
	var c1, c10, c100, c11, c101, c111, c1001, c1011, c1111 unpackedScalar
	c1 = *s
	c10.MontgomerySquare(&c1)
	c100.MontgomerySquare(&c10)
	c11.MontgomeryMul(&c10, &c1)
	c101.MontgomeryMul(&c10, &c11)
	c111.MontgomeryMul(&c10, &c101)
	c1001.MontgomeryMul(&c10, &c111)
	c1011.MontgomeryMul(&c10, &c1001)
	c1111.MontgomeryMul(&c100, &c1011)

	// _10000
	y := s
	y.MontgomeryMul(&c1111, &c1)

	// montgomerySquareMultiply used to be just a function local to
	// montgomeryInvert, but Go's overly primitive escape analysis
	// starts moving things to the heap.

	y.montgomerySquareMultiply(123+3, &c101)
	y.montgomerySquareMultiply(2+2, &c11)
	y.montgomerySquareMultiply(1+4, &c1111)
	y.montgomerySquareMultiply(1+4, &c1111)
	y.montgomerySquareMultiply(4, &c1001)
	y.montgomerySquareMultiply(2, &c11)
	y.montgomerySquareMultiply(1+4, &c1111)
	y.montgomerySquareMultiply(1+3, &c101)
	y.montgomerySquareMultiply(3+3, &c101)
	y.montgomerySquareMultiply(3, &c111)
	y.montgomerySquareMultiply(1+4, &c1111)
	y.montgomerySquareMultiply(2+3, &c111)
	y.montgomerySquareMultiply(2+2, &c11)
	y.montgomerySquareMultiply(1+4, &c1011)
	y.montgomerySquareMultiply(2+4, &c1011)
	y.montgomerySquareMultiply(6+4, &c1001)
	y.montgomerySquareMultiply(2+2, &c11)
	y.montgomerySquareMultiply(3+2, &c11)
	y.montgomerySquareMultiply(3+2, &c11)
	y.montgomerySquareMultiply(1+4, &c1001)
	y.montgomerySquareMultiply(1+3, &c111)
	y.montgomerySquareMultiply(2+4, &c1111)
	y.montgomerySquareMultiply(1+4, &c1011)
	y.montgomerySquareMultiply(3, &c101)
	y.montgomerySquareMultiply(2+4, &c1111)
	y.montgomerySquareMultiply(3, &c101)
	y.montgomerySquareMultiply(1+2, &c11)
}

func (s *unpackedScalar) montgomerySquareMultiply(squarings uint, x *unpackedScalar) {
	for i := uint(0); i < squarings; i++ {
		s.MontgomerySquare(s)
	}
	s.MontgomeryMul(s, x)
}

// Invert sets the s to the multiplicative inverse of the nonzero scalar, and
// returns s.
func (s *unpackedScalar) Invert(a *unpackedScalar) *unpackedScalar {
	aMont := newUnpackedScalar().ToMontgomery(a)
	aMont.MontgomeryInvert()
	return s.FromMontgomery(aMont)
}
