// Copyright (c) 2016-2019 isis agora lovecruft. All rights reserved.
// Copyright (c) 2016-2019 Henry de Valence. All rights reserved.
// Copyright (c) 2020-2021 Oasis Labs Inc. All rights reserved.
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

import "github.com/oasisprotocol/curve25519-voi/curve/scalar"

func edwardsDoubleScalarMulBasepointVartime(out *EdwardsPoint, a *scalar.Scalar, A *EdwardsPoint, b *scalar.Scalar) *EdwardsPoint {
	switch supportsVectorizedEdwards {
	case true:
		return edwardsDoubleScalarMulBasepointVartimeVector(out, a, A, b)
	default:
		return edwardsDoubleScalarMulBasepointVartimeGeneric(out, a, A, b)
	}
}

func expandedEdwardsDoubleScalarMulBasepointVartime(out *EdwardsPoint, a *scalar.Scalar, A *ExpandedEdwardsPoint, b *scalar.Scalar) *EdwardsPoint {
	switch supportsVectorizedEdwards {
	case true:
		return edwardsDoubleScalarMulBasepointVartimeVectorInner(out, a, A.innerVector, b)
	default:
		return edwardsDoubleScalarMulBasepointVartimeGenericInner(out, a, A.inner, b)
	}
}

func edwardsDoubleScalarMulBasepointVartimeGeneric(out *EdwardsPoint, a *scalar.Scalar, A *EdwardsPoint, b *scalar.Scalar) *EdwardsPoint {
	tableA := newProjectiveNielsPointNafLookupTable(A)

	return edwardsDoubleScalarMulBasepointVartimeGenericInner(out, a, &tableA, b)
}

func edwardsDoubleScalarMulBasepointVartimeGenericInner(out *EdwardsPoint, a *scalar.Scalar, tableA *projectiveNielsPointNafLookupTable, b *scalar.Scalar) *EdwardsPoint {
	aNaf := a.NonAdjacentForm(5)
	bNaf := b.NonAdjacentForm(8)

	// Find the starting index.
	var i int
	for j := 255; j >= 0; j-- {
		if aNaf[j] != 0 || bNaf[j] != 0 {
			i = j
			break
		}
	}

	tableB := &constAFFINE_ODD_MULTIPLES_OF_BASEPOINT

	var r projectivePoint
	r.Identity()

	var (
		tEp EdwardsPoint
		t   completedPoint
	)
	for {
		t.Double(&r)

		if aNaf[i] > 0 {
			t.AddEdwardsProjectiveNiels(tEp.setCompleted(&t), tableA.Lookup(uint8(aNaf[i])))
		} else if aNaf[i] < 0 {
			t.SubEdwardsProjectiveNiels(tEp.setCompleted(&t), tableA.Lookup(uint8(-aNaf[i])))
		}

		if bNaf[i] > 0 {
			t.AddEdwardsAffineNiels(tEp.setCompleted(&t), tableB.Lookup(uint8(bNaf[i])))
		} else if bNaf[i] < 0 {
			t.SubEdwardsAffineNiels(tEp.setCompleted(&t), tableB.Lookup(uint8(-bNaf[i])))
		}

		r.SetCompleted(&t)

		if i == 0 {
			break
		}
		i--
	}

	return out.setProjective(&r)
}

func edwardsDoubleScalarMulBasepointVartimeVector(out *EdwardsPoint, a *scalar.Scalar, A *EdwardsPoint, b *scalar.Scalar) *EdwardsPoint {
	tableA := newCachedPointNafLookupTable(A)

	return edwardsDoubleScalarMulBasepointVartimeVectorInner(out, a, &tableA, b)
}

func edwardsDoubleScalarMulBasepointVartimeVectorInner(out *EdwardsPoint, a *scalar.Scalar, tableA *cachedPointNafLookupTable, b *scalar.Scalar) *EdwardsPoint {
	aNaf := a.NonAdjacentForm(5)
	bNaf := b.NonAdjacentForm(8)

	var i int
	for j := 255; j >= 0; j-- {
		if aNaf[j] != 0 || bNaf[j] != 0 {
			i = j
			break
		}
	}

	tableB := constVECTOR_ODD_MULTIPLES_OF_BASEPOINT

	var q extendedPoint
	q.Identity()

	for {
		q.Double(&q)

		if aNaf[i] > 0 {
			q.AddExtendedCached(&q, tableA.Lookup(uint8(aNaf[i])))
		} else if aNaf[i] < 0 {
			q.SubExtendedCached(&q, tableA.Lookup(uint8(-aNaf[i])))
		}

		if bNaf[i] > 0 {
			q.AddExtendedCached(&q, tableB.Lookup(uint8(bNaf[i])))
		} else if bNaf[i] < 0 {
			q.SubExtendedCached(&q, tableB.Lookup(uint8(-bNaf[i])))
		}

		if i == 0 {
			break
		}
		i--
	}

	return out.setExtended(&q)
}
