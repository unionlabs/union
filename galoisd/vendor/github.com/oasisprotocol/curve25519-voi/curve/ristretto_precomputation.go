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

// ExpandedRistreetoPoint is a RistrettoPoint stored in an expanded
// representation for the purpose of accelerating scalar point
// multiply operations.
//
// The default value is NOT valid and MUST only be used as a receiver.
type ExpandedRistrettoPoint struct {
	inner ExpandedEdwardsPoint
}

// Point returns the Ristretto point represented by the expanded point.
func (ep *ExpandedRistrettoPoint) Point() *RistrettoPoint {
	var p RistrettoPoint
	p.inner.Set(&ep.inner.point)
	return &p
}

// SetExpandedRistrettoPoint sets the expanded point to the Ristretto point.
func (ep *ExpandedRistrettoPoint) SetRistrettoPoint(p *RistrettoPoint) *ExpandedRistrettoPoint {
	ep.inner.SetEdwardsPoint(&p.inner)
	return ep
}

// NewExpandedRistrettoPoint creates an expanded representation of a
// Ristretto point.
func NewExpandedRistrettoPoint(p *RistrettoPoint) *ExpandedRistrettoPoint {
	var ep ExpandedRistrettoPoint
	return ep.SetRistrettoPoint(p)
}

// ExpandedDoubleScalarMulBasepointVartime sets `p = (aA + bB)` in variable-time,
// where B is the Ed25519 basepoint, and returns p.
func (p *RistrettoPoint) ExpandedDoubleScalarMulBasepointVartime(a *scalar.Scalar, A *ExpandedRistrettoPoint, b *scalar.Scalar) *RistrettoPoint {
	p.inner.ExpandedDoubleScalarMulBasepointVartime(a, &A.inner, b)
	return p
}

// ExpandedTripleScalarMulBasepoint sets `p = [delta a]A + [delta b]B - [delta]C`
// in variable-time, where delta is a value invertible mod ell, which
// is selected internally to this method.
func (p *RistrettoPoint) ExpandedTripleScalarMulBasepointVartime(a *scalar.Scalar, A *ExpandedRistrettoPoint, b *scalar.Scalar, C *RistrettoPoint) *RistrettoPoint {
	p.inner.ExpandedTripleScalarMulBasepointVartime(a, &A.inner, b, &C.inner)
	return p
}

// ExpandedMultiscalarMulVartime sets `p = staticScalars[0] * staticPoints[0] +
// ... + staticScalars[n] * staticPoints[n] + dynamicScalars[0] *
// dynamicPoints[0] + ... + dynamicScalars[n] * dynamicPoints[n]` in variable-time,
// and returns p.
//
// WARNING: This function will panic if `len(staticScalars) != len(staticPoints)`
// or `len(dynamicScalars) != len(dynamicPoints)`.
func (p *RistrettoPoint) ExpandedMultiscalarMulVartime(staticScalars []*scalar.Scalar, staticPoints []*ExpandedRistrettoPoint, dynamicScalars []*scalar.Scalar, dynamicPoints []*RistrettoPoint) *RistrettoPoint {
	staticRistrettoPoints := make([]*ExpandedEdwardsPoint, 0, len(staticPoints))
	for _, point := range staticPoints {
		staticRistrettoPoints = append(staticRistrettoPoints, &point.inner)
	}
	dynamicRistrettoPoints := make([]*EdwardsPoint, 0, len(dynamicPoints))
	for _, point := range dynamicPoints {
		dynamicRistrettoPoints = append(dynamicRistrettoPoints, &point.inner)
	}

	p.inner.ExpandedMultiscalarMulVartime(staticScalars, staticRistrettoPoints, dynamicScalars, dynamicRistrettoPoints)
	return p
}
