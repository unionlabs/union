// Copyright 2019 The Go Authors. All rights reserved.
// Copyright 2019 George Tankersley. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package ristretto255

import (
	"math/big"

	"github.com/gtank/ristretto255/internal/radix51"
)

// fePow22523 sets out to z^((p-5)/8). (p-5)/8 is 2^252-3.
func fePow22523(out, z *radix51.FieldElement) *radix51.FieldElement {
	// Refactored from golang.org/x/crypto/ed25519/internal/edwards25519.

	var t0, t1, t2 radix51.FieldElement

	t0.Square(z)
	for i := 1; i < 1; i++ {
		t0.Square(&t0)
	}
	t1.Square(&t0)
	for i := 1; i < 2; i++ {
		t1.Square(&t1)
	}
	t1.Mul(z, &t1)
	t0.Mul(&t0, &t1)
	t0.Square(&t0)
	for i := 1; i < 1; i++ {
		t0.Square(&t0)
	}
	t0.Mul(&t1, &t0)
	t1.Square(&t0)
	for i := 1; i < 5; i++ {
		t1.Square(&t1)
	}
	t0.Mul(&t1, &t0)
	t1.Square(&t0)
	for i := 1; i < 10; i++ {
		t1.Square(&t1)
	}
	t1.Mul(&t1, &t0)
	t2.Square(&t1)
	for i := 1; i < 20; i++ {
		t2.Square(&t2)
	}
	t1.Mul(&t2, &t1)
	t1.Square(&t1)
	for i := 1; i < 10; i++ {
		t1.Square(&t1)
	}
	t0.Mul(&t1, &t0)
	t1.Square(&t0)
	for i := 1; i < 50; i++ {
		t1.Square(&t1)
	}
	t1.Mul(&t1, &t0)
	t2.Square(&t1)
	for i := 1; i < 100; i++ {
		t2.Square(&t2)
	}
	t1.Mul(&t2, &t1)
	t1.Square(&t1)
	for i := 1; i < 50; i++ {
		t1.Square(&t1)
	}
	t0.Mul(&t1, &t0)
	t0.Square(&t0)
	for i := 1; i < 2; i++ {
		t0.Square(&t0)
	}
	return out.Mul(&t0, z)
}

// feSqrtRatio sets r to the square root of the ratio of u and v, according to
// Section 3.1.3 of draft-hdevalence-cfrg-ristretto-00.
func feSqrtRatio(r, u, v *radix51.FieldElement) (wasSquare int) {
	var a, b radix51.FieldElement

	v3 := a.Mul(a.Square(v), v)  // v^3 = v^2 * v
	v7 := b.Mul(b.Square(v3), v) // v^7 = (v^3)^2 * v

	// r = (u * v3) * (u * v7)^((p-5)/8)
	uv3 := a.Mul(u, v3) // (u * v3)
	uv7 := b.Mul(u, v7) // (u * v7)
	r.Mul(uv3, fePow22523(r, uv7))

	check := a.Mul(v, a.Square(r)) // check = v * r^2

	uNeg := b.Neg(u)
	correctSignSqrt := check.Equal(u)
	flippedSignSqrt := check.Equal(uNeg)
	flippedSignSqrtI := check.Equal(uNeg.Mul(uNeg, sqrtM1))

	rPrime := b.Mul(r, sqrtM1) // r_prime = SQRT_M1 * r
	// r = CT_SELECT(r_prime IF flipped_sign_sqrt | flipped_sign_sqrt_i ELSE r)
	r.Select(rPrime, r, flippedSignSqrt|flippedSignSqrtI)

	r.Abs(r) // Choose the nonnegative square root.
	return correctSignSqrt | flippedSignSqrt
}

func fieldElementFromDecimal(s string) *radix51.FieldElement {
	n, ok := new(big.Int).SetString(s, 10)
	if !ok {
		panic("ristretto255: not a valid decimal: " + s)
	}
	return new(radix51.FieldElement).FromBig(n)
}
