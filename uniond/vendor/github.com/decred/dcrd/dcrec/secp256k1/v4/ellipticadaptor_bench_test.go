// Copyright 2013-2016 The btcsuite developers
// Copyright (c) 2015-2022 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package secp256k1

import (
	"testing"
)

// BenchmarkScalarBaseMultAdaptor benchmarks multiplying a scalar by the base
// point of the curve via the method used to satisfy the elliptic.Curve
// interface.
func BenchmarkScalarBaseMultAdaptor(b *testing.B) {
	k := fromHex("d74bf844b0862475103d96a611cf2d898447e288d34b360bc885cb8ce7c00575")
	curve := S256()

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		curve.ScalarBaseMult(k.Bytes())
	}
}

// BenchmarkScalarBaseMultLargeAdaptor benchmarks multiplying an abnormally
// large scalar by the base point of the curve via the method used to satisfy
// the elliptic.Curve interface.
func BenchmarkScalarBaseMultLargeAdaptor(b *testing.B) {
	k := fromHex("d74bf844b0862475103d96a611cf2d898447e288d34b360bc885cb8ce7c005751111111011111110")
	curve := S256()

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		curve.ScalarBaseMult(k.Bytes())
	}
}

// BenchmarkScalarMultAdaptor benchmarks multiplying a scalar by an arbitrary
// point on the curve via the method used to satisfy the elliptic.Curve
// interface.
func BenchmarkScalarMultAdaptor(b *testing.B) {
	x := fromHex("34f9460f0e4f08393d192b3c5133a6ba099aa0ad9fd54ebccfacdfa239ff49c6")
	y := fromHex("0b71ea9bd730fd8923f6d25a7a91e7dd7728a960686cb5a901bb419e0f2ca232")
	k := fromHex("d74bf844b0862475103d96a611cf2d898447e288d34b360bc885cb8ce7c00575")
	curve := S256()

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		curve.ScalarMult(x, y, k.Bytes())
	}
}
