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

import "github.com/oasisprotocol/curve25519-voi/internal/subtle"

// This has a mountain of duplicated code because having generics is too
// much to ask for currently.

type projectiveNielsPointLookupTable [8]projectiveNielsPoint

func (tbl *projectiveNielsPointLookupTable) Lookup(x int8) projectiveNielsPoint {
	// Compute xabs = |x|
	xmask := x >> 7
	xabs := uint8((x + xmask) ^ xmask)

	// Set t = 0 * P = identity
	var t projectiveNielsPoint
	t.Identity()
	for j := 1; j < 9; j++ {
		// Copy `points[j-1] == j*P` onto `t` in constant time if `|x| == j`.
		c := subtle.ConstantTimeCompareByte(byte(xabs), byte(j))
		t.ConditionalAssign(&tbl[j-1], c)
	}
	// Now t == |x| * P.

	negMask := int(byte(xmask & 1))
	t.ConditionalNegate(negMask)
	// Now t == x * P.

	return t
}

func newProjectiveNielsPointLookupTable(ep *EdwardsPoint) projectiveNielsPointLookupTable {
	var epPNiels projectiveNielsPoint
	epPNiels.SetEdwards(ep)

	points := [8]projectiveNielsPoint{
		epPNiels, epPNiels, epPNiels, epPNiels,
		epPNiels, epPNiels, epPNiels, epPNiels,
	}
	for j := 0; j < 7; j++ {
		var (
			tmp  completedPoint
			tmp2 EdwardsPoint
		)
		points[j+1].SetEdwards(tmp2.setCompleted(tmp.AddEdwardsProjectiveNiels(ep, &points[j])))
	}

	return projectiveNielsPointLookupTable(points)
}

type affineNielsPointLookupTable [8]affineNielsPoint

func (tbl *affineNielsPointLookupTable) Lookup(x int8) affineNielsPoint {
	// Compute xabs = |x|
	xmask := x >> 7
	xabs := uint8((x + xmask) ^ xmask)

	// Set t = 0 * P = identity
	var t affineNielsPoint
	lookupAffineNiels(tbl, &t, xabs)
	// Now t == |x| * P.

	negMask := int(byte(xmask & 1))
	t.ConditionalNegate(negMask)
	// Now t == x * P.

	return t
}

func (tbl *affineNielsPointLookupTable) Basepoint() *EdwardsPoint {
	aPt := tbl.Lookup(1)

	var ep EdwardsPoint
	ep.setAffineNiels(&aPt)

	return &ep
}

func newAffineNielsPointLookupTable(ep *EdwardsPoint) affineNielsPointLookupTable {
	var epANiels affineNielsPoint
	epANiels.SetEdwards(ep)

	points := [8]affineNielsPoint{
		epANiels, epANiels, epANiels, epANiels,
		epANiels, epANiels, epANiels, epANiels,
	}
	for j := 0; j < 7; j++ {
		var (
			tmp  completedPoint
			tmp2 EdwardsPoint
		)
		points[j+1].SetEdwards(tmp2.setCompleted(tmp.AddEdwardsAffineNiels(ep, &points[j])))
	}

	return affineNielsPointLookupTable(points)
}

type cachedPointLookupTable [8]cachedPoint

func (tbl *cachedPointLookupTable) Basepoint() *EdwardsPoint {
	cPt := tbl.Lookup(1)

	var ep EdwardsPoint
	ep.setCached(&cPt)

	return &ep
}

func (tbl *cachedPointLookupTable) Lookup(x int8) cachedPoint {
	// Compute xabs = |x|
	xmask := x >> 7
	xabs := uint8((x + xmask) ^ xmask)

	// Set t = 0 * P = identity
	var t cachedPoint
	lookupCached(tbl, &t, xabs)
	// Now t == |x| * P.

	negMask := int(byte(xmask & 1))
	t.ConditionalNegate(negMask)
	// Now t == x * P.

	return t
}

func newCachedPointLookupTable(ep *EdwardsPoint) cachedPointLookupTable {
	var (
		epExtended extendedPoint
		epCached   cachedPoint
	)

	epCached.SetExtended(epExtended.SetEdwards(ep))

	points := [8]cachedPoint{
		epCached, epCached, epCached, epCached,
		epCached, epCached, epCached, epCached,
	}
	for i := 0; i < 7; i++ {
		var tmp extendedPoint
		points[i+1].SetExtended(tmp.AddExtendedCached(&epExtended, &points[i]))
	}

	return cachedPointLookupTable(points)
}

// Holds odd multiples 1A, 3A, ..., 15A of a point A.
type projectiveNielsPointNafLookupTable [8]projectiveNielsPoint

func (tbl *projectiveNielsPointNafLookupTable) Lookup(x uint8) *projectiveNielsPoint {
	return &tbl[x/2]
}

func newProjectiveNielsPointNafLookupTable(ep *EdwardsPoint) projectiveNielsPointNafLookupTable {
	var epPNiels projectiveNielsPoint
	epPNiels.SetEdwards(ep)

	Ai := [8]projectiveNielsPoint{
		epPNiels, epPNiels, epPNiels, epPNiels,
		epPNiels, epPNiels, epPNiels, epPNiels,
	}

	var A2 EdwardsPoint
	A2.double(ep)

	for i := 0; i < 7; i++ {
		var (
			tmp  completedPoint
			tmp2 EdwardsPoint
		)
		Ai[i+1].SetEdwards(tmp2.setCompleted(tmp.AddEdwardsProjectiveNiels(&A2, &Ai[i])))
	}

	return projectiveNielsPointNafLookupTable(Ai)
}

// Holds odd multiples 1A, 3A, ..., 15A of a point A.
type cachedPointNafLookupTable [8]cachedPoint

func (tbl *cachedPointNafLookupTable) Basepoint() *EdwardsPoint {
	var ep EdwardsPoint
	ep.setCached(tbl.Lookup(0))

	return &ep
}

func (tbl *cachedPointNafLookupTable) Lookup(x uint8) *cachedPoint {
	return &tbl[x/2]
}

func newCachedPointNafLookupTable(ep *EdwardsPoint) cachedPointNafLookupTable {
	var (
		epExtended extendedPoint
		epCached   cachedPoint
	)

	epCached.SetExtended(epExtended.SetEdwards(ep))

	Ai := [8]cachedPoint{
		epCached, epCached, epCached, epCached,
		epCached, epCached, epCached, epCached,
	}

	var A2 extendedPoint
	A2.Double(&epExtended)

	for i := 0; i < 7; i++ {
		var tmp extendedPoint
		Ai[i+1].SetExtended(tmp.AddExtendedCached(&A2, &Ai[i]))
	}

	return cachedPointNafLookupTable(Ai)
}

// Holds stuff up to 8.
type affineNielsPointNafLookupTable [64]affineNielsPoint

func (tbl *affineNielsPointNafLookupTable) Lookup(x uint8) *affineNielsPoint {
	return &tbl[x/2]
}

func newAffineNielsPointNafLookupTable(ep *EdwardsPoint) affineNielsPointNafLookupTable { //nolint:unused,deadcode
	var epANiels affineNielsPoint
	epANiels.SetEdwards(ep)

	var Ai [64]affineNielsPoint
	for i := range Ai {
		Ai[i] = epANiels
	}

	var A2 EdwardsPoint
	A2.double(ep)

	for i := 0; i < 63; i++ {
		var (
			tmp  completedPoint
			tmp2 EdwardsPoint
		)
		Ai[i+1].SetEdwards(tmp2.setCompleted(tmp.AddEdwardsAffineNiels(&A2, &Ai[i])))
	}

	return affineNielsPointNafLookupTable(Ai)
}

// Holds stuff up to 8.
type cachedPointNafLookupTable8 [64]cachedPoint

func (tbl *cachedPointNafLookupTable8) Lookup(x uint8) *cachedPoint {
	return &tbl[x/2]
}

func newCachedPointNafLookupTable8(ep *EdwardsPoint) cachedPointNafLookupTable8 {
	var (
		epExtended extendedPoint
		epCached   cachedPoint
	)

	epCached.SetExtended(epExtended.SetEdwards(ep))

	var Ai [64]cachedPoint
	for i := range Ai {
		Ai[i] = epCached
	}

	var A2 extendedPoint
	A2.Double(&epExtended)

	for i := 0; i < 63; i++ {
		var tmp extendedPoint
		Ai[i+1].SetExtended(tmp.AddExtendedCached(&A2, &Ai[i]))
	}

	return cachedPointNafLookupTable8(Ai)
}
