// Copyright (c) 2020 Jack Grigg. All rights reserved.
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

package curve

import (
	"github.com/oasisprotocol/curve25519-voi/curve/scalar"
	"github.com/oasisprotocol/curve25519-voi/internal/lattice"
)

func edwardsMulAbglsvPorninVartime(out *EdwardsPoint, a *scalar.Scalar, A *EdwardsPoint, b *scalar.Scalar, C *EdwardsPoint) *EdwardsPoint {
	switch supportsVectorizedEdwards {
	case true:
		return edwardsMulAbglsvPorninVartimeVector(out, a, A, b, C)
	default:
		return edwardsMulAbglsvPorninVartimeGeneric(out, a, A, b, C)
	}
}

func expandedEdwardsMulAbglsvPorninVartime(out *EdwardsPoint, a *scalar.Scalar, A *ExpandedEdwardsPoint, b *scalar.Scalar, C *EdwardsPoint) *EdwardsPoint {
	switch supportsVectorizedEdwards {
	case true:
		return expandedEdwardsMulAbglsvPorninVartimeVector(out, a, A, b, C)
	default:
		return expandedEdwardsMulAbglsvPorninVartimeGeneric(out, a, A, b, C)
	}
}

func edwardsMulAbglsvPorninVartimeGeneric(out *EdwardsPoint, a *scalar.Scalar, A *EdwardsPoint, b *scalar.Scalar, C *EdwardsPoint) *EdwardsPoint {
	// Starting with the target equation:
	//
	//     [(delta_a mod l)]A + [(delta_b mod l)]B - [delta]C
	//
	// We can split delta_b mod l into two halves e_0 (128 bits) and e_1 (125 bits),
	// and rewrite the equation as:
	//
	//     [(delta_a mod l)]A + [e_0]B + [e_1 2^128]B - [delta]C
	//
	// B and [2^128]B are precomputed, and their resulting scalar multiplications each
	// have half as many doublings. We therefore want to find a pair of signed integers
	//
	//     (d_0, d_1) = (delta_a mod l, delta)
	//
	// that both have as few bits as possible, similarly reducing the number of doublings
	// in the scalar multiplications [d_0]A and [d_1]C. This is equivalent to finding a
	// short vector in a lattice of dimension 2.

	// Find a short vector.
	d0, d1 := lattice.FindShortVector(a)

	// Save the sign of d_0, and move the sign of d_1 into it's corresponding base and scalar.
	var (
		negC          EdwardsPoint
		s_b, d_0, d_1 scalar.Scalar
	)
	d0IsNeg := d0.IsNegative()
	if d1.IsNegative() {
		// (-b, C)
		s_b.Neg(b)
		negC.Set(C)
	} else {
		// (b, -C)
		s_b.Set(b)
		negC.Neg(C)
	}
	d0.Abs().ToScalar(&d_0)
	d1.Abs().ToScalar(&d_1)

	tableA := newProjectiveNielsPointNafLookupTable(A)
	tableNegC := newProjectiveNielsPointNafLookupTable(&negC)

	return edwardsMulAbglsvPorninVartimeGenericInner(out, d0IsNeg, &tableA, &d_0, &d_1, &s_b, &tableNegC)
}

func expandedEdwardsMulAbglsvPorninVartimeGeneric(out *EdwardsPoint, a *scalar.Scalar, A *ExpandedEdwardsPoint, b *scalar.Scalar, C *EdwardsPoint) *EdwardsPoint {
	d0, d1 := lattice.FindShortVector(a)

	var (
		negC          EdwardsPoint
		s_b, d_0, d_1 scalar.Scalar
	)
	d0IsNeg := d0.IsNegative()
	if d1.IsNegative() {
		s_b.Neg(b)
		negC.Set(C)
	} else {
		s_b.Set(b)
		negC.Neg(C)
	}
	d0.Abs().ToScalar(&d_0)
	d1.Abs().ToScalar(&d_1)

	tableA := A.inner
	tableNegC := newProjectiveNielsPointNafLookupTable(&negC)

	return edwardsMulAbglsvPorninVartimeGenericInner(out, d0IsNeg, tableA, &d_0, &d_1, &s_b, &tableNegC)
}

func edwardsMulAbglsvPorninVartimeGenericInner(out *EdwardsPoint, d0IsNeg bool, tableA *projectiveNielsPointNafLookupTable, d_0, d_1, s_b *scalar.Scalar, tableNegC *projectiveNielsPointNafLookupTable) *EdwardsPoint {
	// Calculate the remaining scalars.
	var (
		db, e_0, e_1 scalar.Scalar
		dbBuf, eBuf  [scalar.ScalarSize]byte
	)
	db.Mul(s_b, d_1)
	if err := db.ToBytes(dbBuf[:]); err != nil {
		panic("curve: failed to serialize db scalar")
	}
	copy(eBuf[:16], dbBuf[:16])
	if _, err := e_0.SetBits(eBuf[:]); err != nil {
		panic("curve: failed to unpack e_0 scalar")
	}
	copy(eBuf[:16], dbBuf[16:])
	if _, err := e_1.SetBits(eBuf[:]); err != nil {
		panic("curve: failed to unpack e_1 scalar")
	}

	// Now we can compute the following using Straus's method:
	//     [d_0]A + [e_0]B + [e_1][2^128]B + [d_1][-C]
	//
	// We inline it here so we can use precomputed multiples of [2^128]B.

	d_0_naf := d_0.NonAdjacentForm(5)
	e_0_naf := e_0.NonAdjacentForm(8)
	e_1_naf := e_1.NonAdjacentForm(8)
	d_1_naf := d_1.NonAdjacentForm(5)

	// Find the starting index.
	var i int
	for j := 255; j >= 0; j-- {
		i = j
		if d_0_naf[i] != 0 || e_0_naf[i] != 0 || e_1_naf[i] != 0 || d_1_naf[i] != 0 {
			break
		}
	}

	tableB := &constAFFINE_ODD_MULTIPLES_OF_BASEPOINT
	tableB_SHL_128 := &constAFFINE_ODD_MULTIPLES_OF_B_SHL_128

	var r projectivePoint
	r.Identity()

	var (
		tEp EdwardsPoint
		t   completedPoint
	)
	for {
		t.Double(&r)

		// If d0 was negative, instead of using a table generated off -A,
		// just reverse Add/Sub instead, with a easily predicted branch
		// since that beats storing another table when we are doing
		// precomputation.
		if !d0IsNeg {
			if d_0_naf[i] > 0 {
				t.AddEdwardsProjectiveNiels(tEp.setCompleted(&t), tableA.Lookup(uint8(d_0_naf[i])))
			} else if d_0_naf[i] < 0 {
				t.SubEdwardsProjectiveNiels(tEp.setCompleted(&t), tableA.Lookup(uint8(-d_0_naf[i])))
			}
		} else {
			if d_0_naf[i] > 0 {
				t.SubEdwardsProjectiveNiels(tEp.setCompleted(&t), tableA.Lookup(uint8(d_0_naf[i])))
			} else if d_0_naf[i] < 0 {
				t.AddEdwardsProjectiveNiels(tEp.setCompleted(&t), tableA.Lookup(uint8(-d_0_naf[i])))
			}
		}

		if e_0_naf[i] > 0 {
			t.AddCompletedAffineNiels(&t, tableB.Lookup(uint8(e_0_naf[i])))
		} else if e_0_naf[i] < 0 {
			t.SubCompletedAffineNiels(&t, tableB.Lookup(uint8(-e_0_naf[i])))
		}

		if e_1_naf[i] > 0 {
			t.AddCompletedAffineNiels(&t, tableB_SHL_128.Lookup(uint8(e_1_naf[i])))
		} else if e_1_naf[i] < 0 {
			t.SubCompletedAffineNiels(&t, tableB_SHL_128.Lookup(uint8(-e_1_naf[i])))
		}

		if d_1_naf[i] > 0 {
			t.AddEdwardsProjectiveNiels(tEp.setCompleted(&t), tableNegC.Lookup(uint8(d_1_naf[i])))
		} else if d_1_naf[i] < 0 {
			t.SubEdwardsProjectiveNiels(tEp.setCompleted(&t), tableNegC.Lookup(uint8(-d_1_naf[i])))
		}

		r.SetCompleted(&t)

		if i == 0 {
			break
		}
		i--
	}

	return out.setProjective(&r)
}

func edwardsMulAbglsvPorninVartimeVector(out *EdwardsPoint, a *scalar.Scalar, A *EdwardsPoint, b *scalar.Scalar, C *EdwardsPoint) *EdwardsPoint {
	d0, d1 := lattice.FindShortVector(a)

	var (
		negC          EdwardsPoint
		s_b, d_0, d_1 scalar.Scalar
	)
	d0IsNeg := d0.IsNegative()
	if d1.IsNegative() {
		s_b.Neg(b)
		negC.Set(C)
	} else {
		s_b.Set(b)
		negC.Neg(C)
	}
	d0.Abs().ToScalar(&d_0)
	d1.Abs().ToScalar(&d_1)

	tableA := newCachedPointNafLookupTable(A)
	tableNegC := newCachedPointNafLookupTable(&negC)

	return edwardsMulAbglsvPorninVartimeVectorInner(out, d0IsNeg, &tableA, &d_0, &d_1, &s_b, &tableNegC)
}

func expandedEdwardsMulAbglsvPorninVartimeVector(out *EdwardsPoint, a *scalar.Scalar, A *ExpandedEdwardsPoint, b *scalar.Scalar, C *EdwardsPoint) *EdwardsPoint {
	d0, d1 := lattice.FindShortVector(a)

	var (
		negC          EdwardsPoint
		s_b, d_0, d_1 scalar.Scalar
	)
	d0IsNeg := d0.IsNegative()
	if d1.IsNegative() {
		s_b.Neg(b)
		negC.Set(C)
	} else {
		s_b.Set(b)
		negC.Neg(C)
	}
	d0.Abs().ToScalar(&d_0)
	d1.Abs().ToScalar(&d_1)

	tableA := A.innerVector
	tableNegC := newCachedPointNafLookupTable(&negC)

	return edwardsMulAbglsvPorninVartimeVectorInner(out, d0IsNeg, tableA, &d_0, &d_1, &s_b, &tableNegC)
}

func edwardsMulAbglsvPorninVartimeVectorInner(out *EdwardsPoint, d0IsNeg bool, tableA *cachedPointNafLookupTable, d_0, d_1, s_b *scalar.Scalar, tableNegC *cachedPointNafLookupTable) *EdwardsPoint {
	var (
		db, e_0, e_1 scalar.Scalar
		dbBuf, eBuf  [scalar.ScalarSize]byte
	)
	db.Mul(s_b, d_1)
	if err := db.ToBytes(dbBuf[:]); err != nil {
		panic("curve: failed to serialize db scalar")
	}
	copy(eBuf[:16], dbBuf[:16])
	if _, err := e_0.SetBits(eBuf[:]); err != nil {
		panic("curve: failed to unpack e_0 scalar")
	}
	copy(eBuf[:16], dbBuf[16:])
	if _, err := e_1.SetBits(eBuf[:]); err != nil {
		panic("curve: failed to unpack e_1 scalar")
	}

	d_0_naf := d_0.NonAdjacentForm(5)
	e_0_naf := e_0.NonAdjacentForm(8)
	e_1_naf := e_1.NonAdjacentForm(8)
	d_1_naf := d_1.NonAdjacentForm(5)

	var i int
	for j := 255; j >= 0; j-- {
		i = j
		if d_0_naf[i] != 0 || e_0_naf[i] != 0 || e_1_naf[i] != 0 || d_1_naf[i] != 0 {
			break
		}
	}

	tableB := constVECTOR_ODD_MULTIPLES_OF_BASEPOINT
	tableB_SHL_128 := constVECTOR_ODD_MULTIPLES_OF_B_SHL_128

	var q extendedPoint
	q.Identity()

	for {
		q.Double(&q)

		if !d0IsNeg {
			if d_0_naf[i] > 0 {
				q.AddExtendedCached(&q, tableA.Lookup(uint8(d_0_naf[i])))
			} else if d_0_naf[i] < 0 {
				q.SubExtendedCached(&q, tableA.Lookup(uint8(-d_0_naf[i])))
			}
		} else {
			if d_0_naf[i] > 0 {
				q.SubExtendedCached(&q, tableA.Lookup(uint8(d_0_naf[i])))
			} else if d_0_naf[i] < 0 {
				q.AddExtendedCached(&q, tableA.Lookup(uint8(-d_0_naf[i])))
			}
		}

		if e_0_naf[i] > 0 {
			q.AddExtendedCached(&q, tableB.Lookup(uint8(e_0_naf[i])))
		} else if e_0_naf[i] < 0 {
			q.SubExtendedCached(&q, tableB.Lookup(uint8(-e_0_naf[i])))
		}

		if e_1_naf[i] > 0 {
			q.AddExtendedCached(&q, tableB_SHL_128.Lookup(uint8(e_1_naf[i])))
		} else if e_1_naf[i] < 0 {
			q.SubExtendedCached(&q, tableB_SHL_128.Lookup(uint8(-e_1_naf[i])))
		}

		if d_1_naf[i] > 0 {
			q.AddExtendedCached(&q, tableNegC.Lookup(uint8(d_1_naf[i])))
		} else if d_1_naf[i] < 0 {
			q.SubExtendedCached(&q, tableNegC.Lookup(uint8(-d_1_naf[i])))
		}

		if i == 0 {
			break
		}
		i--
	}

	return out.setExtended(&q)
}
