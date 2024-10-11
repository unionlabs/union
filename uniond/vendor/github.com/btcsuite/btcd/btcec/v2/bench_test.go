// Copyright 2013-2016 The btcsuite developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package btcec

import (
	"encoding/hex"
	"math/big"
	"testing"

	secp "github.com/decred/dcrd/dcrec/secp256k1/v4"
)

// setHex decodes the passed big-endian hex string into the internal field value
// representation.  Only the first 32-bytes are used.
//
// This is NOT constant time.
//
// The field value is returned to support chaining.  This enables syntax like:
// f := new(FieldVal).SetHex("0abc").Add(1) so that f = 0x0abc + 1
func setHex(hexString string) *FieldVal {
	if len(hexString)%2 != 0 {
		hexString = "0" + hexString
	}
	bytes, _ := hex.DecodeString(hexString)

	var f FieldVal
	f.SetByteSlice(bytes)

	return &f
}

// hexToFieldVal converts the passed hex string into a FieldVal and will panic
// if there is an error.  This is only provided for the hard-coded constants so
// errors in the source code can be detected. It will only (and must only) be
// called with hard-coded values.
func hexToFieldVal(s string) *FieldVal {
	b, err := hex.DecodeString(s)
	if err != nil {
		panic("invalid hex in source file: " + s)
	}
	var f FieldVal
	if overflow := f.SetByteSlice(b); overflow {
		panic("hex in source file overflows mod P: " + s)
	}
	return &f
}

// fromHex converts the passed hex string into a big integer pointer and will
// panic is there is an error.  This is only provided for the hard-coded
// constants so errors in the source code can bet detected. It will only (and
// must only) be called for initialization purposes.
func fromHex(s string) *big.Int {
	if s == "" {
		return big.NewInt(0)
	}
	r, ok := new(big.Int).SetString(s, 16)
	if !ok {
		panic("invalid hex in source file: " + s)
	}
	return r
}

// jacobianPointFromHex decodes the passed big-endian hex strings into a
// Jacobian point with its internal fields set to the resulting values.  Only
// the first 32-bytes are used.
func jacobianPointFromHex(x, y, z string) JacobianPoint {
	var p JacobianPoint
	p.X = *setHex(x)
	p.Y = *setHex(y)
	p.Z = *setHex(z)

	return p
}

// BenchmarkAddNonConst benchmarks the secp256k1 curve AddNonConst function with
// Z values of 1 so that the associated optimizations are used.
func BenchmarkAddJacobian(b *testing.B) {
	p1 := jacobianPointFromHex(
		"34f9460f0e4f08393d192b3c5133a6ba099aa0ad9fd54ebccfacdfa239ff49c6",
		"0b71ea9bd730fd8923f6d25a7a91e7dd7728a960686cb5a901bb419e0f2ca232",
		"1",
	)
	p2 := jacobianPointFromHex(
		"34f9460f0e4f08393d192b3c5133a6ba099aa0ad9fd54ebccfacdfa239ff49c6",
		"0b71ea9bd730fd8923f6d25a7a91e7dd7728a960686cb5a901bb419e0f2ca232",
		"1",
	)

	b.ReportAllocs()
	b.ResetTimer()
	var result JacobianPoint
	for i := 0; i < b.N; i++ {
		secp.AddNonConst(&p1, &p2, &result)
	}
}

// BenchmarkAddNonConstNotZOne benchmarks the secp256k1 curve AddNonConst
// function with Z values other than one so the optimizations associated with
// Z=1 aren't used.
func BenchmarkAddJacobianNotZOne(b *testing.B) {
	x1 := setHex("d3e5183c393c20e4f464acf144ce9ae8266a82b67f553af33eb37e88e7fd2718")
	y1 := setHex("5b8f54deb987ec491fb692d3d48f3eebb9454b034365ad480dda0cf079651190")
	z1 := setHex("2")
	x2 := setHex("91abba6a34b7481d922a4bd6a04899d5a686f6cf6da4e66a0cb427fb25c04bd4")
	y2 := setHex("03fede65e30b4e7576a2abefc963ddbf9fdccbf791b77c29beadefe49951f7d1")
	z2 := setHex("3")
	p1 := MakeJacobianPoint(x1, y1, z1)
	p2 := MakeJacobianPoint(x2, y2, z2)

	b.ReportAllocs()
	b.ResetTimer()
	var result JacobianPoint
	for i := 0; i < b.N; i++ {
		AddNonConst(&p1, &p2, &result)
	}
}

// BenchmarkScalarBaseMult benchmarks the secp256k1 curve ScalarBaseMult
// function.
func BenchmarkScalarBaseMult(b *testing.B) {
	k := fromHex("d74bf844b0862475103d96a611cf2d898447e288d34b360bc885cb8ce7c00575")
	curve := S256()
	for i := 0; i < b.N; i++ {
		curve.ScalarBaseMult(k.Bytes())
	}
}

// BenchmarkScalarBaseMultLarge benchmarks the secp256k1 curve ScalarBaseMult
// function with abnormally large k values.
func BenchmarkScalarBaseMultLarge(b *testing.B) {
	k := fromHex("d74bf844b0862475103d96a611cf2d898447e288d34b360bc885cb8ce7c005751111111011111110")
	curve := S256()
	for i := 0; i < b.N; i++ {
		curve.ScalarBaseMult(k.Bytes())
	}
}

// BenchmarkScalarMult benchmarks the secp256k1 curve ScalarMult function.
func BenchmarkScalarMult(b *testing.B) {
	x := fromHex("34f9460f0e4f08393d192b3c5133a6ba099aa0ad9fd54ebccfacdfa239ff49c6")
	y := fromHex("0b71ea9bd730fd8923f6d25a7a91e7dd7728a960686cb5a901bb419e0f2ca232")
	k := fromHex("d74bf844b0862475103d96a611cf2d898447e288d34b360bc885cb8ce7c00575")
	curve := S256()
	for i := 0; i < b.N; i++ {
		curve.ScalarMult(x, y, k.Bytes())
	}
}

// hexToModNScalar converts the passed hex string into a ModNScalar and will
// panic if there is an error.  This is only provided for the hard-coded
// constants so errors in the source code can be detected. It will only (and
// must only) be called with hard-coded values.
func hexToModNScalar(s string) *ModNScalar {
	b, err := hex.DecodeString(s)
	if err != nil {
		panic("invalid hex in source file: " + s)
	}
	var scalar ModNScalar
	if overflow := scalar.SetByteSlice(b); overflow {
		panic("hex in source file overflows mod N scalar: " + s)
	}
	return &scalar
}

// BenchmarkFieldNormalize benchmarks how long it takes the internal field
// to perform normalization (which includes modular reduction).
func BenchmarkFieldNormalize(b *testing.B) {
	// The normalize function is constant time so default value is fine.
	var f FieldVal
	for i := 0; i < b.N; i++ {
		f.Normalize()
	}
}

// BenchmarkParseCompressedPubKey benchmarks how long it takes to decompress and
// validate a compressed public key from a byte array.
func BenchmarkParseCompressedPubKey(b *testing.B) {
	rawPk, _ := hex.DecodeString("0234f9460f0e4f08393d192b3c5133a6ba099aa0ad9fd54ebccfacdfa239ff49c6")

	var (
		pk  *PublicKey
		err error
	)

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		pk, err = ParsePubKey(rawPk)
	}
	_ = pk
	_ = err
}
