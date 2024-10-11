// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

package secp256k1

import (
	_ "embed"
	"unsafe"

	"gitlab.com/yawning/secp256k1-voi/internal/field"
	"gitlab.com/yawning/secp256k1-voi/internal/helpers"
)

// Tables for doing accelerated scalar multiplication with a window.
//
// This is heavily inspired by Filippo Valsorda's nistec package,
// as it implements the same algorithm I originally settled on,
// with nicer code, and abstracts out the precomputed table.
//
// Note: Effort is made to omit checking `Point.isValid` as much as
// possible as these routines are internal, and it is entirely
// redundant, once the validity of `p` is checked once.

// projectivePointMultTable stores pre-computed multiples [1P, ... 15P],
// with support for `0P` implicitly as part of the table lookup.
//
// For performance reasons, particularly when creating the table, the
// Z-coordinate for entries is not guaranteed to be 1.
type projectivePointMultTable [15]Point

// SelectAndAdd sets `sum = sum + idx * P`, and returns `sum`.  idx
// MUST be in the range of `[0, 15]`.
func (tbl *projectivePointMultTable) SelectAndAdd(sum *Point, idx uint64) *Point {
	addend := newRcvr()
	lookupProjectivePoint(tbl, addend, idx)

	return sum.addComplete(sum, addend)
}

// SelectAndAddVartime sets `sum = sum + idx * P`, and returns `sum` in
// variable time.  idx MUST be in the range of `[0, 15]`.
func (tbl *projectivePointMultTable) SelectAndAddVartime(sum *Point, idx uint64) *Point {
	if idx == 0 {
		return sum
	}

	return sum.addComplete(sum, &tbl[idx-1])
}

func newProjectivePointMultTable(p *Point) projectivePointMultTable {
	var tbl projectivePointMultTable
	tbl[0].Set(p) // will call `assertPointsValid(p)`
	for i := 1; i < len(tbl); i += 2 {
		tbl[i].doubleComplete(&tbl[i/2])
		tbl[i+1].addComplete(&tbl[i], p)
	}

	return tbl
}

// Routines and tables dedicated to scalar basepoint multiplication.
//
// This is a common operation required by higher-level constructs,
// thus more precomputation is used, relative to the generic case.

// affinePoint is a point on the `Z = 1` plane.
type affinePoint struct {
	x, y field.Element
}

// hugeAffinePointMultTable stores precomputed multiples [1P, ... 255P].
type hugeAffinePointMultTable [255]affinePoint

//go:embed internal/gentable/point_mul_table.bin
var generatorHugeAffineTableBytes []byte

var generatorHugeAffineTable = func() *[ScalarSize]hugeAffinePointMultTable {
	// Unpack the pre-generated multiple tables.  nistec's assembly
	// implementations just point into the table, and do fixups as
	// needed to handle byte-order, but just deserializing is easier.
	var off int
	tbl := new([ScalarSize]hugeAffinePointMultTable)
	for i := range tbl {
		for j := range tbl[i] {
			xBytes := generatorHugeAffineTableBytes[off : off+field.ElementSize]
			off += field.ElementSize
			yBytes := generatorHugeAffineTableBytes[off : off+field.ElementSize]
			off += field.ElementSize

			p := &tbl[i][j]
			p.x.MustSetCanonicalBytes((*[field.ElementSize]byte)(xBytes))
			p.y.MustSetCanonicalBytes((*[field.ElementSize]byte)(yBytes))
		}
	}

	generatorHugeAffineTableBytes = nil // Maybe the GC will prune this.

	return tbl
}()

// SelectAndAddVartime sets `sum = sum + idx * P`, and returns `sum` in
// variable time.  idx MUST be in the range of `[0, 255]`.
func (tbl *hugeAffinePointMultTable) SelectAndAddVartime(sum *Point, idx uint64) *Point {
	if idx == 0 {
		return sum
	}

	p := &tbl[idx-1]
	return sum.addMixed(sum, &p.x, &p.y)
}

// affinePointMultTable stores pre-computed multiples [1P, ... 15P].
type affinePointMultTable [15]affinePoint

// SelectAndAdd sets `sum = sum + idx * P`, and returns `sum`.  idx
// MUST be in the range of `[0, 15]`.
func (tbl *affinePointMultTable) SelectAndAdd(sum *Point, idx uint64) *Point {
	var ap affinePoint
	isInfinity := helpers.Uint64IsZero(idx)
	lookupAffinePoint(tbl, &ap, idx)

	// The formula is incorrect for the point at infinity, so store
	// the result in a temporary value...
	tmp := newRcvr().addMixed(sum, &ap.x, &ap.y)

	// ... and conditionally select the correct result.
	return sum.uncheckedConditionalSelect(tmp, sum, isInfinity)
}

// This stores the odd-indexed doubled tables of precomputed multiples of
// G, such that interleaved with generatorHugeAffineTable one ends up
// with a series of 64 tables of precomputed multiples of G [1G, ... 15G],
// with each successive table being the previous table doubled 4 times.
//
// In theory there is no reason why `generatorHugeAffineTable` can't be
// used here, but that would complicate the vectorized lookup, and the
// memory cost for having this separate table is only 30 KiB.
//
// As in:
//
//	generatorHugeAffineTable[0] = [1G, ... 15G]
//	 generatorOddAffineTable[0] = [1G, ... 15G] * 16
//	generatorHugeAffineTable[1] = [1G, ... 15G] * 256
//	 generatorOddAffineTable[1] = [1G, ... 15G] * 4096
//	...
var generatorOddAffineTable = func() *[ScalarSize]affinePointMultTable {
	tbl := new([ScalarSize]affinePointMultTable)
	for i := 0; i < ScalarSize; i++ {
		fromTbl := &generatorHugeAffineTable[i]

		for j := 0; j < 15; j++ {
			fromIdx := (16 + j<<4) - 1
			tbl[i][j].x.Set(&fromTbl[fromIdx].x)
			tbl[i][j].y.Set(&fromTbl[fromIdx].y)
		}
	}

	return tbl
}()

//
// The various "simple" scalar point multiplication routines.
//

// ScalarBaseMult sets `v = s * G`, and returns `v`, where `G` is the
// generator.
func (v *Point) ScalarBaseMult(s *Scalar) *Point {
	// This uses a 4-bit window, with all of the multiples precomputed
	// to entirely eliminate point doubling operations.  The even-indexed
	// tables are shared with the large variable time lookup table,
	// and a separate table is built for the odd-indexed tables, to
	// simplify the constant time lookup code.
	evenTbls := generatorHugeAffineTable
	oddTbls := generatorOddAffineTable

	v.Identity()
	for i, b := range s.Bytes() {
		tblIdx := ScalarSize - (1 + i)
		oddTbls[tblIdx].SelectAndAdd(v, uint64(b>>4))

		// generatorHugeAffineTable stores [1P, ... 255P], while the
		// lookup routine expects an affinePointMultTable which stores
		// [1P, ... 15P].
		//
		// Since what we have is a strict superset of the other, and
		// the lookup ranges are hardcoded, we can cast our way past
		// code duplication.
		evenTbl := (*affinePointMultTable)(unsafe.Pointer(&evenTbls[tblIdx]))
		evenTbl.SelectAndAdd(v, uint64(b&0xf))
	}

	return v
}

// scalarBaseMultVartime sets `v = s * G`, and returns `v` in variable time.
func (v *Point) scalarBaseMultVartime(s *Scalar) *Point {
	// This uses a 8-bit window, with all of the multiples precomputed
	// to entirely eliminate point doubling operations.  Unlike in the
	// constant-time case, a huge table here pays off since there is no
	// need to scan the entire table for timing-sidechannel mitigation
	// reasons.
	tbl := generatorHugeAffineTable

	v.Identity()
	for i, b := range s.Bytes() {
		tbl[ScalarSize-(1+i)].SelectAndAddVartime(v, uint64(b))
	}

	return v
}
