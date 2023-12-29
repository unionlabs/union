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

import "github.com/oasisprotocol/curve25519-voi/curve/scalar"

// ExpandedEdwardsPoint is an Edwards point stored in an expanded
// representation for the purpose of accelerating scalar point
// multiply operations.
//
// The default value is NOT valid and MUST only be used as a receiver.
type ExpandedEdwardsPoint struct {
	point EdwardsPoint

	inner       *projectiveNielsPointNafLookupTable
	innerVector *cachedPointNafLookupTable

	// TODO/perf: Consider adding support for Pippenger's algorithm,
	// though that requires a table with 64 -> 256 entries depending
	// on the `w` used, which is a massive waste of space if the
	// multiscalar multiply is never called with a large number of
	// terms.
}

// Point returns the Edwards point represented by the expanded point.
func (ep *ExpandedEdwardsPoint) Point() *EdwardsPoint {
	var p EdwardsPoint
	return p.Set(&ep.point)
}

// SetEdwardsPoint sets the expanded point to the Edwards point.
func (ep *ExpandedEdwardsPoint) SetEdwardsPoint(p *EdwardsPoint) *ExpandedEdwardsPoint {
	var negP EdwardsPoint
	negP.Neg(p)

	switch supportsVectorizedEdwards {
	case true:
		tbl := newCachedPointNafLookupTable(p)
		ep.inner = nil
		ep.innerVector = &tbl
	default:
		tbl := newProjectiveNielsPointNafLookupTable(p)
		ep.inner = &tbl
		ep.innerVector = nil
	}

	ep.point.Set(p)

	return ep
}

// NewExpandedEdwardsPoint creates an expanded representation of an
// Edwards point.
func NewExpandedEdwardsPoint(p *EdwardsPoint) *ExpandedEdwardsPoint {
	var ep ExpandedEdwardsPoint
	return ep.SetEdwardsPoint(p)
}

// ExpandedDoubleScalarMulBasepointVartime sets `p = (aA + bB)` in variable-time,
// where B is the Ed25519 basepoint, and returns p.
func (p *EdwardsPoint) ExpandedDoubleScalarMulBasepointVartime(a *scalar.Scalar, A *ExpandedEdwardsPoint, b *scalar.Scalar) *EdwardsPoint {
	return expandedEdwardsDoubleScalarMulBasepointVartime(p, a, A, b)
}

// ExpandedTripleScalarMulBasepoint sets `p = [delta a]A + [delta b]B - [delta]C`
// in variable-time, where delta is a value invertible mod ell, which
// is selected internally to this method.
func (p *EdwardsPoint) ExpandedTripleScalarMulBasepointVartime(a *scalar.Scalar, A *ExpandedEdwardsPoint, b *scalar.Scalar, C *EdwardsPoint) *EdwardsPoint {
	return expandedEdwardsMulAbglsvPorninVartime(p, a, A, b, C)
}

// ExpandedMultiscalarMulVartime sets `p = staticScalars[0] * staticPoints[0] +
// ... + staticScalars[n] * staticPoints[n] + dynamicScalars[0] *
// dynamicPoints[0] + ... + dynamicScalars[n] * dynamicPoints[n]` in variable-time,
// and returns p.
//
// WARNING: This function will panic if `len(staticScalars) != len(staticPoints)`
// or `len(dynamicScalars) != len(dynamicPoints)`.
func (p *EdwardsPoint) ExpandedMultiscalarMulVartime(staticScalars []*scalar.Scalar, staticPoints []*ExpandedEdwardsPoint, dynamicScalars []*scalar.Scalar, dynamicPoints []*EdwardsPoint) *EdwardsPoint {
	staticSize, dynamicSize := len(staticScalars), len(dynamicScalars)
	if staticSize != len(staticPoints) {
		panic("curve/edwards: len(staticScalars) != len(staticPoints)")
	}
	if dynamicSize != len(dynamicPoints) {
		panic("curve/edwards: len(dynamicScalars) != len(dynamicPoints)")
	}

	switch {
	case staticSize+dynamicSize > mulPippengerThreshold:
		return expandedEdwardsMultiscalarMulPippengerVartime(p, staticScalars, staticPoints, dynamicScalars, dynamicPoints)
	default:
		return expandedEdwardsMultiscalarMulStrausVartime(p, staticScalars, staticPoints, dynamicScalars, dynamicPoints)
	}
}
