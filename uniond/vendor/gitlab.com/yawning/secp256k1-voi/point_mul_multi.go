// Copyright (c) 2023 Yawning Angel
//
// SPDX-License-Identifier: BSD-3-Clause

package secp256k1

// These are useful for hipster crypto such as batch Schnorr verification,
// Bulletproofs, MuSig2, etc.
//
// Notes:
// - The Vartime variant is currently an extremely marginal gain on amd64
// due to the vectorized constant time table lookup.  However the gain
// is more substantial on other platforms, and splitting the API allows
// optimizing each variant independently in the future without breaking
// downstream code.
// - Pippenger's algorithm is superior to Straus's for large batches,
// but this is significantly easier to understand, and it's also always
// a gain.
// - The appropriate single ScalarMult is called if the size of the batch
// is 1, to leverage GLV decomposition.  Decomposition is not worth it
// in the other cases (GLV has overhead that scales with batch-size,
// while the amount of work saved is fixed).

// MultiScalarMult sets `v = sum(scalars[i] * points[i])`, and returns `v`.
func (v *Point) MultiScalarMult(scalars []*Scalar, points []*Point) *Point { //nolint:dupl
	l := len(scalars)
	if l != len(points) {
		panic("secp256k1: len(scalars) != len(points)")
	}

	if l == 1 {
		return v.ScalarMult(scalars[0], points[0])
	}

	pTbls := make([]projectivePointMultTable, l)
	sBytes := make([][ScalarSize]byte, l)
	for i := 0; i < l; i++ {
		pTbls[i] = newProjectivePointMultTable(points[i])
		scalars[i].getBytes(&sBytes[i])
	}

	v.Identity()

	for i := 0; i < ScalarSize; i++ {
		if i != 0 {
			v.doubleComplete(v)
			v.doubleComplete(v)
			v.doubleComplete(v)
			v.doubleComplete(v)
		}

		for j := 0; j < l; j++ {
			b := sBytes[j][i]
			pTbls[j].SelectAndAdd(v, uint64(b>>4))
		}

		v.doubleComplete(v)
		v.doubleComplete(v)
		v.doubleComplete(v)
		v.doubleComplete(v)

		for j := 0; j < l; j++ {
			b := sBytes[j][i]
			pTbls[j].SelectAndAdd(v, uint64(b&0xf))
		}
	}

	return v
}

// MultiScalarMultVartime sets `v = sum(scalars[i] * points[i])`, and returns
// `v` in variable time.
func (v *Point) MultiScalarMultVartime(scalars []*Scalar, points []*Point) *Point { //nolint:dupl
	l := len(scalars)
	if l != len(points) {
		panic("secp256k1: len(scalars) != len(points)")
	}

	if l == 1 {
		return v.scalarMultVartimeGLV(scalars[0], points[0])
	}

	pTbls := make([]projectivePointMultTable, l)
	sBytes := make([][ScalarSize]byte, l)
	for i := 0; i < l; i++ {
		pTbls[i] = newProjectivePointMultTable(points[i])
		scalars[i].getBytes(&sBytes[i])
	}

	v.Identity()

	for i := 0; i < ScalarSize; i++ {
		if i != 0 {
			v.doubleComplete(v)
			v.doubleComplete(v)
			v.doubleComplete(v)
			v.doubleComplete(v)
		}

		for j := 0; j < l; j++ {
			b := sBytes[j][i]
			pTbls[j].SelectAndAddVartime(v, uint64(b>>4))
		}

		v.doubleComplete(v)
		v.doubleComplete(v)
		v.doubleComplete(v)
		v.doubleComplete(v)

		for j := 0; j < l; j++ {
			b := sBytes[j][i]
			pTbls[j].SelectAndAddVartime(v, uint64(b&0xf))
		}
	}

	return v
}
