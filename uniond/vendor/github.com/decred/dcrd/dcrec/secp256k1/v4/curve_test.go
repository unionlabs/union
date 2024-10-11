// Copyright (c) 2015-2024 The Decred developers
// Copyright 2013-2016 The btcsuite developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

package secp256k1

import (
	"fmt"
	"math/big"
	"math/bits"
	mrand "math/rand"
	"testing"
	"time"
)

var (
	// oneModN is simply the number 1 as a mod n scalar.
	oneModN = hexToModNScalar("1")

	// endoLambda is the positive version of the lambda constant used in the
	// endomorphism.  It is stored here for convenience and to avoid recomputing
	// it throughout the tests.
	endoLambda = new(ModNScalar).NegateVal(endoNegLambda)
)

// isValidJacobianPoint returns true if the point (x,y,z) is on the secp256k1
// curve or is the point at infinity.
func isValidJacobianPoint(point *JacobianPoint) bool {
	if (point.X.IsZero() && point.Y.IsZero()) || point.Z.IsZero() {
		return true
	}

	// Elliptic curve equation for secp256k1 is: y^2 = x^3 + 7
	// In Jacobian coordinates, Y = y/z^3 and X = x/z^2
	// Thus:
	// (y/z^3)^2 = (x/z^2)^3 + 7
	// y^2/z^6 = x^3/z^6 + 7
	// y^2 = x^3 + 7*z^6
	var y2, z2, x3, result FieldVal
	y2.SquareVal(&point.Y).Normalize()
	z2.SquareVal(&point.Z)
	x3.SquareVal(&point.X).Mul(&point.X)
	result.SquareVal(&z2).Mul(&z2).MulInt(7).Add(&x3).Normalize()
	return y2.Equals(&result)
}

// jacobianPointFromHex decodes the passed big-endian hex strings into a
// Jacobian point with its internal fields set to the resulting values.  Only
// the first 32-bytes are used.
func jacobianPointFromHex(x, y, z string) JacobianPoint {
	var p JacobianPoint
	p.X.SetHex(x)
	p.Y.SetHex(y)
	p.Z.SetHex(z)
	return p
}

// IsStrictlyEqual returns whether or not the two Jacobian points are strictly
// equal for use in the tests.  Recall that several Jacobian points can be equal
// in affine coordinates, while not having the same coordinates in projective
// space, so the two points not being equal doesn't necessarily mean they aren't
// actually the same affine point.
func (p *JacobianPoint) IsStrictlyEqual(other *JacobianPoint) bool {
	return p.X.Equals(&other.X) && p.Y.Equals(&other.Y) && p.Z.Equals(&other.Z)
}

// TestAddJacobian tests addition of points projected in Jacobian coordinates
// works as intended.
func TestAddJacobian(t *testing.T) {
	tests := []struct {
		name       string // test description
		x1, y1, z1 string // hex encoded coordinates of first point to add
		x2, y2, z2 string // hex encoded coordinates of second point to add
		x3, y3, z3 string // hex encoded coordinates of expected point
	}{{
		// Addition with the point at infinity (left hand side).
		name: "∞ + P = P",
		x1:   "0",
		y1:   "0",
		z1:   "0",
		x2:   "d74bf844b0862475103d96a611cf2d898447e288d34b360bc885cb8ce7c00575",
		y2:   "131c670d414c4546b88ac3ff664611b1c38ceb1c21d76369d7a7a0969d61d97d",
		z2:   "1",
		x3:   "d74bf844b0862475103d96a611cf2d898447e288d34b360bc885cb8ce7c00575",
		y3:   "131c670d414c4546b88ac3ff664611b1c38ceb1c21d76369d7a7a0969d61d97d",
		z3:   "1",
	}, {
		// Addition with the point at infinity (right hand side).
		name: "P + ∞ = P",
		x1:   "d74bf844b0862475103d96a611cf2d898447e288d34b360bc885cb8ce7c00575",
		y1:   "131c670d414c4546b88ac3ff664611b1c38ceb1c21d76369d7a7a0969d61d97d",
		z1:   "1",
		x2:   "0",
		y2:   "0",
		z2:   "0",
		x3:   "d74bf844b0862475103d96a611cf2d898447e288d34b360bc885cb8ce7c00575",
		y3:   "131c670d414c4546b88ac3ff664611b1c38ceb1c21d76369d7a7a0969d61d97d",
		z3:   "1",
	}, {
		// Addition with z1=z2=1 different x values.
		name: "P(x1, y1, 1) + P(x2, y1, 1)",
		x1:   "34f9460f0e4f08393d192b3c5133a6ba099aa0ad9fd54ebccfacdfa239ff49c6",
		y1:   "0b71ea9bd730fd8923f6d25a7a91e7dd7728a960686cb5a901bb419e0f2ca232",
		z1:   "1",
		x2:   "d74bf844b0862475103d96a611cf2d898447e288d34b360bc885cb8ce7c00575",
		y2:   "131c670d414c4546b88ac3ff664611b1c38ceb1c21d76369d7a7a0969d61d97d",
		z2:   "1",
		x3:   "0cfbc7da1e569b334460788faae0286e68b3af7379d5504efc25e4dba16e46a6",
		y3:   "e205f79361bbe0346b037b4010985dbf4f9e1e955e7d0d14aca876bfa79aad87",
		z3:   "44a5646b446e3877a648d6d381370d9ef55a83b666ebce9df1b1d7d65b817b2f",
	}, {
		// Addition with z1=z2=1 same x opposite y.
		name: "P(x, y, 1) + P(x, -y, 1) = ∞",
		x1:   "34f9460f0e4f08393d192b3c5133a6ba099aa0ad9fd54ebccfacdfa239ff49c6",
		y1:   "0b71ea9bd730fd8923f6d25a7a91e7dd7728a960686cb5a901bb419e0f2ca232",
		z1:   "1",
		x2:   "34f9460f0e4f08393d192b3c5133a6ba099aa0ad9fd54ebccfacdfa239ff49c6",
		y2:   "f48e156428cf0276dc092da5856e182288d7569f97934a56fe44be60f0d359fd",
		z2:   "1",
		x3:   "0",
		y3:   "0",
		z3:   "0",
	}, {
		// Addition with z1=z2=1 same point.
		name: "P(x, y, 1) + P(x, y, 1) = 2P",
		x1:   "34f9460f0e4f08393d192b3c5133a6ba099aa0ad9fd54ebccfacdfa239ff49c6",
		y1:   "0b71ea9bd730fd8923f6d25a7a91e7dd7728a960686cb5a901bb419e0f2ca232",
		z1:   "1",
		x2:   "34f9460f0e4f08393d192b3c5133a6ba099aa0ad9fd54ebccfacdfa239ff49c6",
		y2:   "0b71ea9bd730fd8923f6d25a7a91e7dd7728a960686cb5a901bb419e0f2ca232",
		z2:   "1",
		x3:   "ec9f153b13ee7bd915882859635ea9730bf0dc7611b2c7b0e37ee64f87c50c27",
		y3:   "b082b53702c466dcf6e984a35671756c506c67c2fcb8adb408c44dd0755c8f2a",
		z3:   "16e3d537ae61fb1247eda4b4f523cfbaee5152c0d0d96b520376833c1e594464",
	}, {
		// Addition with z1=z2 (!=1) different x values.
		name: "P(x1, y1, 2) + P(x2, y2, 2)",
		x1:   "d3e5183c393c20e4f464acf144ce9ae8266a82b67f553af33eb37e88e7fd2718",
		y1:   "5b8f54deb987ec491fb692d3d48f3eebb9454b034365ad480dda0cf079651190",
		z1:   "2",
		x2:   "5d2fe112c21891d440f65a98473cb626111f8a234d2cd82f22172e369f002147",
		y2:   "98e3386a0a622a35c4561ffb32308d8e1c6758e10ebb1b4ebd3d04b4eb0ecbe8",
		z2:   "2",
		x3:   "cfbc7da1e569b334460788faae0286e68b3af7379d5504efc25e4dba16e46a60",
		y3:   "817de4d86ef80d1ac0ded00426176fd3e787a5579f43452b2a1db021e6ac3778",
		z3:   "129591ad11b8e1de99235b4e04dc367bd56a0ed99baf3a77c6c75f5a6e05f08d",
	}, {
		// Addition with z1=z2 (!=1) same x opposite y.
		name: "P(x, y, 2) + P(x, -y, 2) = ∞",
		x1:   "d3e5183c393c20e4f464acf144ce9ae8266a82b67f553af33eb37e88e7fd2718",
		y1:   "5b8f54deb987ec491fb692d3d48f3eebb9454b034365ad480dda0cf079651190",
		z1:   "2",
		x2:   "d3e5183c393c20e4f464acf144ce9ae8266a82b67f553af33eb37e88e7fd2718",
		y2:   "a470ab21467813b6e0496d2c2b70c11446bab4fcbc9a52b7f225f30e869aea9f",
		z2:   "2",
		x3:   "0",
		y3:   "0",
		z3:   "0",
	}, {
		// Addition with z1=z2 (!=1) same point.
		name: "P(x, y, 2) + P(x, y, 2) = 2P",
		x1:   "d3e5183c393c20e4f464acf144ce9ae8266a82b67f553af33eb37e88e7fd2718",
		y1:   "5b8f54deb987ec491fb692d3d48f3eebb9454b034365ad480dda0cf079651190",
		z1:   "2",
		x2:   "d3e5183c393c20e4f464acf144ce9ae8266a82b67f553af33eb37e88e7fd2718",
		y2:   "5b8f54deb987ec491fb692d3d48f3eebb9454b034365ad480dda0cf079651190",
		z2:   "2",
		x3:   "9f153b13ee7bd915882859635ea9730bf0dc7611b2c7b0e37ee65073c50fabac",
		y3:   "2b53702c466dcf6e984a35671756c506c67c2fcb8adb408c44dd125dc91cb988",
		z3:   "6e3d537ae61fb1247eda4b4f523cfbaee5152c0d0d96b520376833c2e5944a11",
	}, {
		// Addition with z1!=z2 and z2=1 different x values.
		name: "P(x1, y1, 2) + P(x2, y2, 1)",
		x1:   "d3e5183c393c20e4f464acf144ce9ae8266a82b67f553af33eb37e88e7fd2718",
		y1:   "5b8f54deb987ec491fb692d3d48f3eebb9454b034365ad480dda0cf079651190",
		z1:   "2",
		x2:   "d74bf844b0862475103d96a611cf2d898447e288d34b360bc885cb8ce7c00575",
		y2:   "131c670d414c4546b88ac3ff664611b1c38ceb1c21d76369d7a7a0969d61d97d",
		z2:   "1",
		x3:   "3ef1f68795a6ccd1181e23eab80a1b9a2cebdcde755413bf097936eb5b91b4f3",
		y3:   "0bef26c377c068d606f6802130bb7e9f3c3d2abcfa1a295950ed81133561cb04",
		z3:   "252b235a2371c3bd3246b69c09b86cf7aad41db3375e74ef8d8ebeb4dc0be11a",
	}, {
		// Addition with z1!=z2 and z2=1 same x opposite y.
		name: "P(x, y, 2) + P(x, -y, 1) = ∞",
		x1:   "d3e5183c393c20e4f464acf144ce9ae8266a82b67f553af33eb37e88e7fd2718",
		y1:   "5b8f54deb987ec491fb692d3d48f3eebb9454b034365ad480dda0cf079651190",
		z1:   "2",
		x2:   "34f9460f0e4f08393d192b3c5133a6ba099aa0ad9fd54ebccfacdfa239ff49c6",
		y2:   "f48e156428cf0276dc092da5856e182288d7569f97934a56fe44be60f0d359fd",
		z2:   "1",
		x3:   "0",
		y3:   "0",
		z3:   "0",
	}, {
		// Addition with z1!=z2 and z2=1 same point.
		name: "P(x, y, 2) + P(x, y, 1) = 2P",
		x1:   "d3e5183c393c20e4f464acf144ce9ae8266a82b67f553af33eb37e88e7fd2718",
		y1:   "5b8f54deb987ec491fb692d3d48f3eebb9454b034365ad480dda0cf079651190",
		z1:   "2",
		x2:   "34f9460f0e4f08393d192b3c5133a6ba099aa0ad9fd54ebccfacdfa239ff49c6",
		y2:   "0b71ea9bd730fd8923f6d25a7a91e7dd7728a960686cb5a901bb419e0f2ca232",
		z2:   "1",
		x3:   "9f153b13ee7bd915882859635ea9730bf0dc7611b2c7b0e37ee65073c50fabac",
		y3:   "2b53702c466dcf6e984a35671756c506c67c2fcb8adb408c44dd125dc91cb988",
		z3:   "6e3d537ae61fb1247eda4b4f523cfbaee5152c0d0d96b520376833c2e5944a11",
	}, {
		// Addition with z1!=z2 and z2!=1 different x values.
		name: "P(x1, y1, 2) + P(x2, y2, 3)",
		x1:   "d3e5183c393c20e4f464acf144ce9ae8266a82b67f553af33eb37e88e7fd2718",
		y1:   "5b8f54deb987ec491fb692d3d48f3eebb9454b034365ad480dda0cf079651190",
		z1:   "2",
		x2:   "91abba6a34b7481d922a4bd6a04899d5a686f6cf6da4e66a0cb427fb25c04bd4",
		y2:   "03fede65e30b4e7576a2abefc963ddbf9fdccbf791b77c29beadefe49951f7d1",
		z2:   "3",
		x3:   "3f07081927fd3f6dadd4476614c89a09eba7f57c1c6c3b01fa2d64eac1eef31e",
		y3:   "949166e04ebc7fd95a9d77e5dfd88d1492ecffd189792e3944eb2b765e09e031",
		z3:   "eb8cba81bcffa4f44d75427506737e1f045f21e6d6f65543ee0e1d163540c931",
	}, {
		// Addition with z1!=z2 and z2!=1 same x opposite y.
		name: "P(x, y, 2) + P(x, -y, 3) = ∞",
		x1:   "d3e5183c393c20e4f464acf144ce9ae8266a82b67f553af33eb37e88e7fd2718",
		y1:   "5b8f54deb987ec491fb692d3d48f3eebb9454b034365ad480dda0cf079651190",
		z1:   "2",
		x2:   "dcc3768780c74a0325e2851edad0dc8a566fa61a9e7fc4a34d13dcb509f99bc7",
		y2:   "cafc41904dd5428934f7d075129c8ba46eb622d4fc88d72cd1401452664add18",
		z2:   "3",
		x3:   "0",
		y3:   "0",
		z3:   "0",
	}, {
		// Addition with z1!=z2 and z2!=1 same point.
		name: "P(x, y, 2) + P(x, y, 3) = 2P",
		x1:   "d3e5183c393c20e4f464acf144ce9ae8266a82b67f553af33eb37e88e7fd2718",
		y1:   "5b8f54deb987ec491fb692d3d48f3eebb9454b034365ad480dda0cf079651190",
		z1:   "2",
		x2:   "dcc3768780c74a0325e2851edad0dc8a566fa61a9e7fc4a34d13dcb509f99bc7",
		y2:   "3503be6fb22abd76cb082f8aed63745b9149dd2b037728d32ebfebac99b51f17",
		z2:   "3",
		x3:   "9f153b13ee7bd915882859635ea9730bf0dc7611b2c7b0e37ee65073c50fabac",
		y3:   "2b53702c466dcf6e984a35671756c506c67c2fcb8adb408c44dd125dc91cb988",
		z3:   "6e3d537ae61fb1247eda4b4f523cfbaee5152c0d0d96b520376833c2e5944a11",
	}}

	for _, test := range tests {
		// Convert hex to Jacobian points.
		p1 := jacobianPointFromHex(test.x1, test.y1, test.z1)
		p2 := jacobianPointFromHex(test.x2, test.y2, test.z2)
		want := jacobianPointFromHex(test.x3, test.y3, test.z3)

		// Ensure the test data is using points that are actually on the curve
		// (or the point at infinity).
		if !isValidJacobianPoint(&p1) {
			t.Errorf("%s: first point is not on the curve", test.name)
			continue
		}
		if !isValidJacobianPoint(&p2) {
			t.Errorf("%s: second point is not on the curve", test.name)
			continue
		}
		if !isValidJacobianPoint(&want) {
			t.Errorf("%s: expected point is not on the curve", test.name)
			continue
		}

		// Add the two points.
		var r JacobianPoint
		AddNonConst(&p1, &p2, &r)

		// Ensure result matches expected.
		if !r.IsStrictlyEqual(&want) {
			t.Errorf("%s: wrong result\ngot: (%v, %v, %v)\nwant: (%v, %v, %v)",
				test.name, r.X, r.Y, r.Z, want.X, want.Y, want.Z)
			continue
		}
	}
}

// TestDoubleJacobian tests doubling of points projected in Jacobian coordinates
// works as intended for some edge cases and known good values.
func TestDoubleJacobian(t *testing.T) {
	tests := []struct {
		name       string // test description
		x1, y1, z1 string // hex encoded coordinates of point to double
		x3, y3, z3 string // hex encoded coordinates of expected point
	}{{
		// Doubling the point at infinity is still infinity.
		name: "2*∞ = ∞ (point at infinity)",
		x1:   "0",
		y1:   "0",
		z1:   "0",
		x3:   "0",
		y3:   "0",
		z3:   "0",
	}, {
		// Doubling with z1=1.
		name: "2*P(x, y, 1)",
		x1:   "34f9460f0e4f08393d192b3c5133a6ba099aa0ad9fd54ebccfacdfa239ff49c6",
		y1:   "0b71ea9bd730fd8923f6d25a7a91e7dd7728a960686cb5a901bb419e0f2ca232",
		z1:   "1",
		x3:   "ec9f153b13ee7bd915882859635ea9730bf0dc7611b2c7b0e37ee64f87c50c27",
		y3:   "b082b53702c466dcf6e984a35671756c506c67c2fcb8adb408c44dd0755c8f2a",
		z3:   "16e3d537ae61fb1247eda4b4f523cfbaee5152c0d0d96b520376833c1e594464",
	}, {
		// Doubling with z1!=1.
		name: "2*P(x, y, 2)",
		x1:   "d3e5183c393c20e4f464acf144ce9ae8266a82b67f553af33eb37e88e7fd2718",
		y1:   "5b8f54deb987ec491fb692d3d48f3eebb9454b034365ad480dda0cf079651190",
		z1:   "2",
		x3:   "9f153b13ee7bd915882859635ea9730bf0dc7611b2c7b0e37ee65073c50fabac",
		y3:   "2b53702c466dcf6e984a35671756c506c67c2fcb8adb408c44dd125dc91cb988",
		z3:   "6e3d537ae61fb1247eda4b4f523cfbaee5152c0d0d96b520376833c2e5944a11",
	}, {
		// From btcd issue #709.
		name: "carry to bit 256 during normalize",
		x1:   "201e3f75715136d2f93c4f4598f91826f94ca01f4233a5bd35de9708859ca50d",
		y1:   "bdf18566445e7562c6ada68aef02d498d7301503de5b18c6aef6e2b1722412e1",
		z1:   "0000000000000000000000000000000000000000000000000000000000000001",
		x3:   "4a5e0559863ebb4e9ed85f5c4fa76003d05d9a7626616e614a1f738621e3c220",
		y3:   "00000000000000000000000000000000000000000000000000000001b1388778",
		z3:   "7be30acc88bceac58d5b4d15de05a931ae602a07bcb6318d5dedc563e4482993",
	}}

	for _, test := range tests {
		// Convert hex to field values.
		p1 := jacobianPointFromHex(test.x1, test.y1, test.z1)
		want := jacobianPointFromHex(test.x3, test.y3, test.z3)

		// Ensure the test data is using points that are actually on the curve
		// (or the point at infinity).
		if !isValidJacobianPoint(&p1) {
			t.Errorf("%s: first point is not on the curve", test.name)
			continue
		}
		if !isValidJacobianPoint(&want) {
			t.Errorf("%s: expected point is not on the curve", test.name)
			continue
		}

		// Double the point.
		var result JacobianPoint
		DoubleNonConst(&p1, &result)

		// Ensure result matches expected.
		if !result.IsStrictlyEqual(&want) {
			t.Errorf("%s: wrong result\ngot: (%v, %v, %v)\nwant: (%v, %v, %v)",
				test.name, result.X, result.Y, result.Z, want.X, want.Y,
				want.Z)
			continue
		}
	}
}

// checkNAFEncoding returns an error if the provided positive and negative
// portions of an overall NAF encoding do not adhere to the requirements or they
// do not sum back to the provided original value.
func checkNAFEncoding(pos, neg []byte, origValue *big.Int) error {
	// NAF must not have a leading zero byte and the number of negative
	// bytes must not exceed the positive portion.
	if len(pos) > 0 && pos[0] == 0 {
		return fmt.Errorf("positive has leading zero -- got %x", pos)
	}
	if len(neg) > len(pos) {
		return fmt.Errorf("negative has len %d > pos len %d", len(neg),
			len(pos))
	}

	// Ensure the result doesn't have any adjacent non-zero digits.
	gotPos := new(big.Int).SetBytes(pos)
	gotNeg := new(big.Int).SetBytes(neg)
	posOrNeg := new(big.Int).Or(gotPos, gotNeg)
	prevBit := posOrNeg.Bit(0)
	for bit := 1; bit < posOrNeg.BitLen(); bit++ {
		thisBit := posOrNeg.Bit(bit)
		if prevBit == 1 && thisBit == 1 {
			return fmt.Errorf("adjacent non-zero digits found at bit pos %d",
				bit-1)
		}
		prevBit = thisBit
	}

	// Ensure the resulting positive and negative portions of the overall
	// NAF representation sum back to the original value.
	gotValue := new(big.Int).Sub(gotPos, gotNeg)
	if origValue.Cmp(gotValue) != 0 {
		return fmt.Errorf("pos-neg is not original value: got %x, want %x",
			gotValue, origValue)
	}

	return nil
}

// TestNAF ensures encoding various edge cases and values to non-adjacent form
// produces valid results.
func TestNAF(t *testing.T) {
	tests := []struct {
		name string // test description
		in   string // hex encoded test value
	}{{
		name: "empty is zero",
		in:   "",
	}, {
		name: "zero",
		in:   "00",
	}, {
		name: "just before first carry",
		in:   "aa",
	}, {
		name: "first carry",
		in:   "ab",
	}, {
		name: "leading zeroes",
		in:   "002f20569b90697ad471c1be6107814f53f47446be298a3a2a6b686b97d35cf9",
	}, {
		name: "257 bits when NAF encoded",
		in:   "c000000000000000000000000000000000000000000000000000000000000001",
	}, {
		name: "32-byte scalar",
		in:   "6df2b5d30854069ccdec40ae022f5c948936324a4e9ebed8eb82cfd5a6b6d766",
	}, {
		name: "first term of balanced length-two representation #1",
		in:   "b776e53fb55f6b006a270d42d64ec2b1",
	}, {
		name: "second term balanced length-two representation #1",
		in:   "d6cc32c857f1174b604eefc544f0c7f7",
	}, {
		name: "first term of balanced length-two representation #2",
		in:   "45c53aa1bb56fcd68c011e2dad6758e4",
	}, {
		name: "second term of balanced length-two representation #2",
		in:   "a2e79d200f27f2360fba57619936159b",
	}}

	for _, test := range tests {
		// Ensure the resulting positive and negative portions of the overall
		// NAF representation adhere to the requirements of NAF encoding and
		// they sum back to the original value.
		result := naf(hexToBytes(test.in))
		pos, neg := result.Pos(), result.Neg()
		if err := checkNAFEncoding(pos, neg, fromHex(test.in)); err != nil {
			t.Errorf("%q: %v", test.name, err)
		}
	}
}

// TestNAFRandom ensures that encoding randomly-generated values to non-adjacent
// form produces valid results.
func TestNAFRandom(t *testing.T) {
	// Use a unique random seed each test instance and log it if the tests fail.
	seed := time.Now().Unix()
	rng := mrand.New(mrand.NewSource(seed))
	defer func(t *testing.T, seed int64) {
		if t.Failed() {
			t.Logf("random seed: %d", seed)
		}
	}(t, seed)

	for i := 0; i < 100; i++ {
		// Ensure the resulting positive and negative portions of the overall
		// NAF representation adhere to the requirements of NAF encoding and
		// they sum back to the original value.
		bigIntVal, modNVal := randIntAndModNScalar(t, rng)
		valBytes := modNVal.Bytes()
		result := naf(valBytes[:])
		pos, neg := result.Pos(), result.Neg()
		if err := checkNAFEncoding(pos, neg, bigIntVal); err != nil {
			t.Fatalf("encoding err: %v\nin: %x\npos: %x\nneg: %x", err,
				bigIntVal, pos, neg)
		}
	}
}

// TestScalarBaseMultJacobian ensures multiplying a given scalar by the base
// point projected in Jacobian coordinates works as intended for some edge cases
// and known values.  It also verifies in affine coordinates as well.
func TestScalarBaseMultJacobian(t *testing.T) {
	tests := []struct {
		name       string // test description
		k          string // hex encoded scalar
		x1, y1, z1 string // hex encoded Jacobian coordinates of expected point
		x2, y2     string // hex encoded affine coordinates of expected point
	}{{
		name: "zero",
		k:    "0000000000000000000000000000000000000000000000000000000000000000",
		x1:   "0000000000000000000000000000000000000000000000000000000000000000",
		y1:   "0000000000000000000000000000000000000000000000000000000000000000",
		z1:   "0000000000000000000000000000000000000000000000000000000000000001",
		x2:   "0000000000000000000000000000000000000000000000000000000000000000",
		y2:   "0000000000000000000000000000000000000000000000000000000000000000",
	}, {
		name: "one (aka 1*G = G)",
		k:    "0000000000000000000000000000000000000000000000000000000000000001",
		x1:   "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
		y1:   "483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8",
		z1:   "0000000000000000000000000000000000000000000000000000000000000001",
		x2:   "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
		y2:   "483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8",
	}, {
		name: "group order - 1 (aka -1*G = -G)",
		k:    "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364140",
		x1:   "667d5346809ba7602db1ea0bd990eee6ff75d7a64004d563534123e6f12a12d7",
		y1:   "344f2f772f8f4cbd04709dba7837ff1422db8fa6f99a00f93852de2c45284838",
		z1:   "19e5a058ef4eaada40d19063917bb4dc07f50c3a0f76bd5348a51057a3721c57",
		x2:   "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
		y2:   "b7c52588d95c3b9aa25b0403f1eef75702e84bb7597aabe663b82f6f04ef2777",
	}, {
		name: "known good point 1",
		k:    "aa5e28d6a97a2479a65527f7290311a3624d4cc0fa1578598ee3c2613bf99522",
		x1:   "5f64fd9364bac24dc32bc01b7d63aaa8249babbdc26b03233e14120840ae20f6",
		y1:   "a4ced9be1e1ed6ef73bec6866c3adc0695347303c30b814fb0dfddb3a22b090d",
		z1:   "931a3477a1b1d866842b22577618e134c89ba12e5bb38c465265c8a2cefa69dc",
		x2:   "34f9460f0e4f08393d192b3c5133a6ba099aa0ad9fd54ebccfacdfa239ff49c6",
		y2:   "0b71ea9bd730fd8923f6d25a7a91e7dd7728a960686cb5a901bb419e0f2ca232",
	}, {
		name: "known good point 2",
		k:    "7e2b897b8cebc6361663ad410835639826d590f393d90a9538881735256dfae3",
		x1:   "c2cb761af4d6410bea0ed7d5f3c7397b63739b0f37e5c3047f8a45537a9d413e",
		y1:   "34b9204c55336d2fb94e20e53d5aa2ffe4da6f80d72315b4dcafca11e7c0f768",
		z1:   "ca5d9e8024575c80fe185416ff4736aff8278873da60cf101d10ab49780ee33b",
		x2:   "d74bf844b0862475103d96a611cf2d898447e288d34b360bc885cb8ce7c00575",
		y2:   "131c670d414c4546b88ac3ff664611b1c38ceb1c21d76369d7a7a0969d61d97d",
	}, {
		name: "known good point 3",
		k:    "6461e6df0fe7dfd05329f41bf771b86578143d4dd1f7866fb4ca7e97c5fa945d",
		x1:   "09160b87ee751ef9fd51db49afc7af9c534917fad72bf461d21fec2590878267",
		y1:   "dbc2757c5038e0b059d1e05c2d3706baf1a164e3836a02c240173b22c92da7c0",
		z1:   "c157ea3f784c37603d9f55e661dd1d6b8759fccbfb2c8cf64c46529d94c8c950",
		x2:   "e8aecc370aedd953483719a116711963ce201ac3eb21d3f3257bb48668c6a72f",
		y2:   "c25caf2f0eba1ddb2f0f3f47866299ef907867b7d27e95b3873bf98397b24ee1",
	}, {
		name: "known good point 4",
		k:    "376a3a2cdcd12581efff13ee4ad44c4044b8a0524c42422a7e1e181e4deeccec",
		x1:   "7820c46de3b5a0202bea06870013fcb23adb4a000f89d5b86fe1df24be58fa79",
		y1:   "95e5a977eb53a582677ff0432eef5bc66f1dd983c3e8c07e1c77c3655542c31e",
		z1:   "7d71ecfdfa66b003fe96f925b5907f67a1a4a6489f4940ec3b78edbbf847334f",
		x2:   "14890e61fcd4b0bd92e5b36c81372ca6fed471ef3aa60a3e415ee4fe987daba1",
		y2:   "297b858d9f752ab42d3bca67ee0eb6dcd1c2b7b0dbe23397e66adc272263f982",
	}, {
		name: "known good point 5",
		k:    "1b22644a7be026548810c378d0b2994eefa6d2b9881803cb02ceff865287d1b9",
		x1:   "68a934fa2d28fb0b0d2b6801a9335d62e65acef9467be2ea67f5b11614b59c78",
		y1:   "5edd7491e503acf61ed651a10cf466de06bf5c6ba285a7a2885a384bbdd32898",
		z1:   "f3b28d36c3132b6f4bd66bf0da64b8dc79d66f9a854ba8b609558b6328796755",
		x2:   "f73c65ead01c5126f28f442d087689bfa08e12763e0cec1d35b01751fd735ed3",
		y2:   "f449a8376906482a84ed01479bd18882b919c140d638307f0c0934ba12590bde",
	}}

	for _, test := range tests {
		// Parse test data.
		want := jacobianPointFromHex(test.x1, test.y1, test.z1)
		wantAffine := jacobianPointFromHex(test.x2, test.y2, "01")
		k := hexToModNScalar(test.k)

		// Ensure the test data is using points that are actually on the curve
		// (or the point at infinity).
		if !isValidJacobianPoint(&want) {
			t.Errorf("%q: expected point is not on the curve", test.name)
			continue
		}
		if !isValidJacobianPoint(&wantAffine) {
			t.Errorf("%q: expected affine point is not on the curve", test.name)
			continue
		}

		// Ensure the result matches the expected value in Jacobian coordinates.
		var r JacobianPoint
		scalarBaseMultNonConstFast(k, &r)
		if !r.IsStrictlyEqual(&want) {
			t.Errorf("%q: wrong result:\ngot: (%s, %s, %s)\nwant: (%s, %s, %s)",
				test.name, r.X, r.Y, r.Z, want.X, want.Y, want.Z)
			continue
		}

		// Ensure the result matches the expected value in affine coordinates.
		r.ToAffine()
		if !r.IsStrictlyEqual(&wantAffine) {
			t.Errorf("%q: wrong affine result:\ngot: (%s, %s)\nwant: (%s, %s)",
				test.name, r.X, r.Y, wantAffine.X, wantAffine.Y)
			continue
		}

		// The slow fallback doesn't return identical Jacobian coordinates,
		// but the affine coordinates should match.
		scalarBaseMultNonConstSlow(k, &r)
		r.ToAffine()
		if !r.IsStrictlyEqual(&wantAffine) {
			t.Errorf("%q: wrong affine result:\ngot: (%s, %s)\nwant: (%s, %s)",
				test.name, r.X, r.Y, wantAffine.X, wantAffine.Y)
			continue
		}
	}
}

// modNBitLen returns the minimum number of bits required to represent the mod n
// scalar.  The result is 0 when the value is 0.
func modNBitLen(s *ModNScalar) uint16 {
	if w := s.n[7]; w > 0 {
		return uint16(bits.Len32(w)) + 224
	}
	if w := s.n[6]; w > 0 {
		return uint16(bits.Len32(w)) + 192
	}
	if w := s.n[5]; w > 0 {
		return uint16(bits.Len32(w)) + 160
	}
	if w := s.n[4]; w > 0 {
		return uint16(bits.Len32(w)) + 128
	}
	if w := s.n[3]; w > 0 {
		return uint16(bits.Len32(w)) + 96
	}
	if w := s.n[2]; w > 0 {
		return uint16(bits.Len32(w)) + 64
	}
	if w := s.n[1]; w > 0 {
		return uint16(bits.Len32(w)) + 32
	}
	return uint16(bits.Len32(s.n[0]))
}

// checkLambdaDecomposition returns an error if the provided decomposed scalars
// do not satisfy the required equation or they are not small in magnitude.
func checkLambdaDecomposition(origK, k1, k2 *ModNScalar) error {
	// Recompose the scalar from the decomposed scalars to ensure they satisfy
	// the required equation.
	calcK := new(ModNScalar).Mul2(k2, endoLambda).Add(k1)
	if !calcK.Equals(origK) {
		return fmt.Errorf("recomposed scalar %v != orig scalar", calcK)
	}

	// Ensure the decomposed scalars are small in magnitude by affirming their
	// bit lengths do not exceed one more than half of the bit size of the
	// underlying field.  This value is max(||v1||, ||v2||), where:
	//
	// vector v1 = <endoA1, endoB1>
	// vector v2 = <endoA2, endoB2>
	const maxBitLen = 129
	if k1.IsOverHalfOrder() {
		k1.Negate()
	}
	if k2.IsOverHalfOrder() {
		k2.Negate()
	}
	k1BitLen, k2BitLen := modNBitLen(k1), modNBitLen(k2)
	if k1BitLen > maxBitLen {
		return fmt.Errorf("k1 scalar bit len %d > max allowed %d",
			k1BitLen, maxBitLen)
	}
	if k2BitLen > maxBitLen {
		return fmt.Errorf("k2 scalar bit len %d > max allowed %d",
			k2BitLen, maxBitLen)
	}

	return nil
}

// TestSplitK ensures decomposing various edge cases and values into a balanced
// length-two representation produces valid results.
func TestSplitK(t *testing.T) {
	// Values computed from the group half order and lambda such that they
	// exercise the decomposition edge cases and maximize the bit lengths of the
	// produced scalars.
	h := "7fffffffffffffffffffffffffffffff5d576e7357a4501ddfe92f46681b20a0"
	negOne := new(ModNScalar).NegateVal(oneModN)
	halfOrder := hexToModNScalar(h)
	halfOrderMOne := new(ModNScalar).Add2(halfOrder, negOne)
	halfOrderPOne := new(ModNScalar).Add2(halfOrder, oneModN)
	lambdaMOne := new(ModNScalar).Add2(endoLambda, negOne)
	lambdaPOne := new(ModNScalar).Add2(endoLambda, oneModN)
	negLambda := new(ModNScalar).NegateVal(endoLambda)
	halfOrderMOneMLambda := new(ModNScalar).Add2(halfOrderMOne, negLambda)
	halfOrderMLambda := new(ModNScalar).Add2(halfOrder, negLambda)
	halfOrderPOneMLambda := new(ModNScalar).Add2(halfOrderPOne, negLambda)
	lambdaPHalfOrder := new(ModNScalar).Add2(endoLambda, halfOrder)
	lambdaPOnePHalfOrder := new(ModNScalar).Add2(lambdaPOne, halfOrder)

	tests := []struct {
		name string      // test description
		k    *ModNScalar // scalar to decompose
	}{{
		name: "zero",
		k:    new(ModNScalar),
	}, {
		name: "one",
		k:    oneModN,
	}, {
		name: "group order - 1 (aka -1 mod N)",
		k:    negOne,
	}, {
		name: "group half order - 1 - lambda",
		k:    halfOrderMOneMLambda,
	}, {
		name: "group half order - lambda",
		k:    halfOrderMLambda,
	}, {
		name: "group half order + 1 - lambda",
		k:    halfOrderPOneMLambda,
	}, {
		name: "group half order - 1",
		k:    halfOrderMOne,
	}, {
		name: "group half order",
		k:    halfOrder,
	}, {
		name: "group half order + 1",
		k:    halfOrderPOne,
	}, {
		name: "lambda - 1",
		k:    lambdaMOne,
	}, {
		name: "lambda",
		k:    endoLambda,
	}, {
		name: "lambda + 1",
		k:    lambdaPOne,
	}, {
		name: "lambda + group half order",
		k:    lambdaPHalfOrder,
	}, {
		name: "lambda + 1 + group half order",
		k:    lambdaPOnePHalfOrder,
	}}

	for _, test := range tests {
		// Decompose the scalar and ensure the resulting decomposition satisfies
		// the required equation and consists of scalars that are small in
		// magnitude.
		k1, k2 := splitK(test.k)
		if err := checkLambdaDecomposition(test.k, &k1, &k2); err != nil {
			t.Errorf("%q: %v", test.name, err)
		}
	}
}

// TestSplitKRandom ensures that decomposing randomly-generated scalars into a
// balanced length-two representation produces valid results.
func TestSplitKRandom(t *testing.T) {
	// Use a unique random seed each test instance and log it if the tests fail.
	seed := time.Now().Unix()
	rng := mrand.New(mrand.NewSource(seed))
	defer func(t *testing.T, seed int64) {
		if t.Failed() {
			t.Logf("random seed: %d", seed)
		}
	}(t, seed)

	for i := 0; i < 100; i++ {
		// Generate a random scalar, decompose it, and ensure the resulting
		// decomposition satisfies the required equation and consists of scalars
		// that are small in magnitude.
		origK := randModNScalar(t, rng)
		k1, k2 := splitK(origK)
		if err := checkLambdaDecomposition(origK, &k1, &k2); err != nil {
			t.Fatalf("decomposition err: %v\nin: %v\nk1: %v\nk2: %v", err,
				origK, k1, k2)
		}
	}
}

// TestScalarMultJacobianRandom ensures scalar point multiplication with points
// projected into Jacobian coordinates works as intended for randomly-generated
// scalars and points.
func TestScalarMultJacobianRandom(t *testing.T) {
	// Use a unique random seed each test instance and log it if the tests fail.
	seed := time.Now().Unix()
	rng := mrand.New(mrand.NewSource(seed))
	defer func(t *testing.T, seed int64) {
		if t.Failed() {
			t.Logf("random seed: %d", seed)
		}
	}(t, seed)

	// isSamePoint returns whether or not the two Jacobian points represent the
	// same affine point without modifying the provided points.
	isSamePoint := func(p1, p2 *JacobianPoint) bool {
		var p1Affine, p2Affine JacobianPoint
		p1Affine.Set(p1)
		p1Affine.ToAffine()
		p2Affine.Set(p2)
		p2Affine.ToAffine()
		return p1Affine.IsStrictlyEqual(&p2Affine)
	}

	// The overall idea is to compute the same point different ways.  The
	// strategy uses two properties:
	//
	// 1) Compatibility of scalar multiplication with field multiplication
	// 2) A point added to its negation is the point at infinity (P+(-P) = ∞)
	//
	// First, calculate a "chained" point by starting with the base (generator)
	// point and then consecutively multiply the resulting points by a series of
	// random scalars.
	//
	// Then, multiply the base point by the product of all of the random scalars
	// and ensure the "chained" point matches.
	//
	// In other words:
	//
	// k[n]*(...*(k[2]*(k[1]*(k[0]*G)))) = (k[0]*k[1]*k[2]*...*k[n])*G
	//
	// Along the way, also calculate (-k)*P for each chained point and ensure it
	// sums with the current point to the point at infinity.
	//
	// That is:
	//
	// k*P + ((-k)*P) = ∞
	const numIterations = 1024
	var infinity JacobianPoint
	var chained, negChained, result JacobianPoint
	var negK ModNScalar
	bigAffineToJacobian(curveParams.Gx, curveParams.Gy, &chained)
	product := new(ModNScalar).SetInt(1)
	for i := 0; i < numIterations; i++ {
		// Generate a random scalar and calculate:
		//
		//  P = k*P
		// -P = (-k)*P
		//
		// Notice that this is intentionally doing the full scalar mult with -k
		// as opposed to just flipping the Y coordinate in order to test scalar
		// multiplication.
		k := randModNScalar(t, rng)
		negK.NegateVal(k)
		ScalarMultNonConst(&negK, &chained, &negChained)
		ScalarMultNonConst(k, &chained, &chained)

		// Ensure kP + ((-k)P) = ∞.
		AddNonConst(&chained, &negChained, &result)
		if !isSamePoint(&result, &infinity) {
			t.Fatalf("%d: expected point at infinity\ngot (%v, %v, %v)\n", i,
				result.X, result.Y, result.Z)
		}

		product.Mul(k)
	}

	// Ensure the point calculated above matches the product of the scalars
	// times the base point.
	scalarBaseMultNonConstFast(product, &result)
	if !isSamePoint(&chained, &result) {
		t.Fatalf("unexpected result \ngot (%v, %v, %v)\n"+
			"want (%v, %v, %v)", chained.X, chained.Y, chained.Z, result.X,
			result.Y, result.Z)
	}

	scalarBaseMultNonConstSlow(product, &result)
	if !isSamePoint(&chained, &result) {
		t.Fatalf("unexpected result \ngot (%v, %v, %v)\n"+
			"want (%v, %v, %v)", chained.X, chained.Y, chained.Z, result.X,
			result.Y, result.Z)
	}
}

// TestDecompressY ensures that decompressY works as expected for some edge
// cases.
func TestDecompressY(t *testing.T) {
	tests := []struct {
		name      string // test description
		x         string // hex encoded x coordinate
		valid     bool   // expected decompress result
		wantOddY  string // hex encoded expected odd y coordinate
		wantEvenY string // hex encoded expected even y coordinate
	}{{
		name:      "x = 0 -- not a point on the curve",
		x:         "0",
		valid:     false,
		wantOddY:  "",
		wantEvenY: "",
	}, {
		name:      "x = 1",
		x:         "1",
		valid:     true,
		wantOddY:  "bde70df51939b94c9c24979fa7dd04ebd9b3572da7802290438af2a681895441",
		wantEvenY: "4218f20ae6c646b363db68605822fb14264ca8d2587fdd6fbc750d587e76a7ee",
	}, {
		name:      "x = secp256k1 prime (aka 0) -- not a point on the curve",
		x:         "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f",
		valid:     false,
		wantOddY:  "",
		wantEvenY: "",
	}, {
		name:      "x = secp256k1 prime - 1 -- not a point on the curve",
		x:         "fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e",
		valid:     false,
		wantOddY:  "",
		wantEvenY: "",
	}, {
		name:      "x = secp256k1 group order",
		x:         "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141",
		valid:     true,
		wantOddY:  "670999be34f51e8894b9c14211c28801d9a70fde24b71d3753854b35d07c9a11",
		wantEvenY: "98f66641cb0ae1776b463ebdee3d77fe2658f021db48e2c8ac7ab4c92f83621e",
	}, {
		name:      "x = secp256k1 group order - 1 -- not a point on the curve",
		x:         "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364140",
		valid:     false,
		wantOddY:  "",
		wantEvenY: "",
	}}

	for _, test := range tests {
		// Decompress the test odd y coordinate for the given test x coordinate
		// and ensure the returned validity flag matches the expected result.
		var oddY FieldVal
		fx := new(FieldVal).SetHex(test.x)
		valid := DecompressY(fx, true, &oddY)
		if valid != test.valid {
			t.Errorf("%s: unexpected valid flag -- got: %v, want: %v",
				test.name, valid, test.valid)
			continue
		}

		// Decompress the test even y coordinate for the given test x coordinate
		// and ensure the returned validity flag matches the expected result.
		var evenY FieldVal
		valid = DecompressY(fx, false, &evenY)
		if valid != test.valid {
			t.Errorf("%s: unexpected valid flag -- got: %v, want: %v",
				test.name, valid, test.valid)
			continue
		}

		// Skip checks related to the y coordinate when there isn't one.
		if !valid {
			continue
		}

		// Ensure the decompressed odd Y coordinate is the expected value.
		oddY.Normalize()
		wantOddY := new(FieldVal).SetHex(test.wantOddY)
		if !wantOddY.Equals(&oddY) {
			t.Errorf("%s: mismatched odd y\ngot: %v, want: %v", test.name,
				oddY, wantOddY)
			continue
		}

		// Ensure the decompressed even Y coordinate is the expected value.
		evenY.Normalize()
		wantEvenY := new(FieldVal).SetHex(test.wantEvenY)
		if !wantEvenY.Equals(&evenY) {
			t.Errorf("%s: mismatched even y\ngot: %v, want: %v", test.name,
				evenY, wantEvenY)
			continue
		}

		// Ensure the decompressed odd y coordinate is actually odd.
		if !oddY.IsOdd() {
			t.Errorf("%s: odd y coordinate is even", test.name)
			continue
		}

		// Ensure the decompressed even y coordinate is actually even.
		if evenY.IsOdd() {
			t.Errorf("%s: even y coordinate is odd", test.name)
			continue
		}
	}
}

// TestDecompressYRandom ensures that decompressY works as expected with
// randomly-generated x coordinates.
func TestDecompressYRandom(t *testing.T) {
	// Use a unique random seed each test instance and log it if the tests fail.
	seed := time.Now().Unix()
	rng := mrand.New(mrand.NewSource(seed))
	defer func(t *testing.T, seed int64) {
		if t.Failed() {
			t.Logf("random seed: %d", seed)
		}
	}(t, seed)

	for i := 0; i < 100; i++ {
		origX := randFieldVal(t, rng)

		// Calculate both corresponding y coordinates for the random x when it
		// is a valid coordinate.
		var oddY, evenY FieldVal
		x := new(FieldVal).Set(origX)
		oddSuccess := DecompressY(x, true, &oddY)
		evenSuccess := DecompressY(x, false, &evenY)

		// Ensure that the decompression success matches for both the even and
		// odd cases depending on whether or not x is a valid coordinate.
		if oddSuccess != evenSuccess {
			t.Fatalf("mismatched decompress success for x = %v -- odd: %v, "+
				"even: %v", x, oddSuccess, evenSuccess)
		}
		if !oddSuccess {
			continue
		}

		// Ensure the x coordinate was not changed.
		if !x.Equals(origX) {
			t.Fatalf("x coordinate changed -- orig: %v, changed: %v", origX, x)
		}

		// Ensure that the resulting y coordinates match their respective
		// expected oddness.
		oddY.Normalize()
		evenY.Normalize()
		if !oddY.IsOdd() {
			t.Fatalf("requested odd y is even for x = %v", x)
		}
		if evenY.IsOdd() {
			t.Fatalf("requested even y is odd for x = %v", x)
		}

		// Ensure that the resulting x and y coordinates are actually on the
		// curve for both cases.
		if !isOnCurve(x, &oddY) {
			t.Fatalf("(%v, %v) is not a valid point", x, oddY)
		}
		if !isOnCurve(x, &evenY) {
			t.Fatalf("(%v, %v) is not a valid point", x, evenY)
		}
	}
}
