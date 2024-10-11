// Copyright 2015 The btcsuite developers
// Copyright (c) 2015-2022 The Decred developers
// Use of this source code is governed by an ISC
// license that can be found in the LICENSE file.

// This file is ignored during the regular build due to the following build tag.
// It is called by go generate and used to automatically generate pre-computed
// tables used to accelerate operations.
//go:build ignore
// +build ignore

package main

// References:
//   [GECC]: Guide to Elliptic Curve Cryptography (Hankerson, Menezes, Vanstone)

import (
	"bytes"
	"compress/zlib"
	"encoding/base64"
	"fmt"
	"log"
	"math/big"
	"os"

	"github.com/decred/dcrd/dcrec/secp256k1/v4"
)

// curveParams houses the secp256k1 curve parameters for convenient access.
var curveParams = secp256k1.Params()

// bigAffineToJacobian takes an affine point (x, y) as big integers and converts
// it to Jacobian point with Z=1.
func bigAffineToJacobian(x, y *big.Int, result *secp256k1.JacobianPoint) {
	result.X.SetByteSlice(x.Bytes())
	result.Y.SetByteSlice(y.Bytes())
	result.Z.SetInt(1)
}

// serializedBytePoints returns a serialized byte slice which contains all of
// the possible points per 8-bit window.  This is used to when generating
// compressedbytepoints.go.
func serializedBytePoints() []byte {
	// Calculate G^(2^i) for i in 0..255.  These are used to avoid recomputing
	// them for each digit of the 8-bit windows.
	doublingPoints := make([]secp256k1.JacobianPoint, curveParams.BitSize)
	var q secp256k1.JacobianPoint
	bigAffineToJacobian(curveParams.Gx, curveParams.Gy, &q)
	for i := 0; i < curveParams.BitSize; i++ {
		// Q = 2*Q.
		doublingPoints[i] = q
		secp256k1.DoubleNonConst(&q, &q)
	}

	// Separate the bits into byte-sized windows.
	curveByteSize := curveParams.BitSize / 8
	serialized := make([]byte, curveByteSize*256*2*32)
	offset := 0
	for byteNum := 0; byteNum < curveByteSize; byteNum++ {
		// Grab the 8 bits that make up this byte from doubling points.
		startingBit := 8 * (curveByteSize - byteNum - 1)
		windowPoints := doublingPoints[startingBit : startingBit+8]

		// Compute all points in this window, convert them to affine, and
		// serialize them.
		for i := 0; i < 256; i++ {
			var point secp256k1.JacobianPoint
			for bit := 0; bit < 8; bit++ {
				if i>>uint(bit)&1 == 1 {
					secp256k1.AddNonConst(&point, &windowPoints[bit], &point)
				}
			}
			point.ToAffine()

			point.X.PutBytesUnchecked(serialized[offset:])
			offset += 32
			point.Y.PutBytesUnchecked(serialized[offset:])
			offset += 32
		}
	}

	return serialized
}

// sqrt returns the square root of the provided big integer using Newton's
// method.  It's only compiled and used during generation of pre-computed
// values, so speed is not a huge concern.
func sqrt(n *big.Int) *big.Int {
	// Initial guess = 2^(log_2(n)/2)
	guess := big.NewInt(2)
	guess.Exp(guess, big.NewInt(int64(n.BitLen()/2)), nil)

	// Now refine using Newton's method.
	big2 := big.NewInt(2)
	prevGuess := big.NewInt(0)
	for {
		prevGuess.Set(guess)
		guess.Add(guess, new(big.Int).Div(n, guess))
		guess.Div(guess, big2)
		if guess.Cmp(prevGuess) == 0 {
			break
		}
	}
	return guess
}

// endomorphismParams houses the parameters needed to make use of the secp256k1
// endomorphism.
type endomorphismParams struct {
	lambda *big.Int
	beta   *big.Int
	a1, b1 *big.Int
	a2, b2 *big.Int
	z1, z2 *big.Int
}

// endomorphismVectors runs the first 3 steps of algorithm 3.74 from [GECC] to
// generate the linearly independent vectors needed to generate a balanced
// length-two representation of a multiplier such that k = k1 + k2λ (mod N) and
// returns them.  Since the values will always be the same given the fact that N
// and λ are fixed, the final results can be accelerated by storing the
// precomputed values.
func endomorphismVectors(lambda *big.Int) (a1, b1, a2, b2 *big.Int) {
	// This section uses an extended Euclidean algorithm to generate a
	// sequence of equations:
	//  s[i] * N + t[i] * λ = r[i]

	nSqrt := sqrt(curveParams.N)
	u, v := new(big.Int).Set(curveParams.N), new(big.Int).Set(lambda)
	x1, y1 := big.NewInt(1), big.NewInt(0)
	x2, y2 := big.NewInt(0), big.NewInt(1)
	q, r := new(big.Int), new(big.Int)
	qu, qx1, qy1 := new(big.Int), new(big.Int), new(big.Int)
	s, t := new(big.Int), new(big.Int)
	ri, ti := new(big.Int), new(big.Int)
	a1, b1, a2, b2 = new(big.Int), new(big.Int), new(big.Int), new(big.Int)
	found, oneMore := false, false
	for u.Sign() != 0 {
		// q = v/u
		q.Div(v, u)

		// r = v - q*u
		qu.Mul(q, u)
		r.Sub(v, qu)

		// s = x2 - q*x1
		qx1.Mul(q, x1)
		s.Sub(x2, qx1)

		// t = y2 - q*y1
		qy1.Mul(q, y1)
		t.Sub(y2, qy1)

		// v = u, u = r, x2 = x1, x1 = s, y2 = y1, y1 = t
		v.Set(u)
		u.Set(r)
		x2.Set(x1)
		x1.Set(s)
		y2.Set(y1)
		y1.Set(t)

		// As soon as the remainder is less than the sqrt of n, the
		// values of a1 and b1 are known.
		if !found && r.Cmp(nSqrt) < 0 {
			// When this condition executes ri and ti represent the
			// r[i] and t[i] values such that i is the greatest
			// index for which r >= sqrt(n).  Meanwhile, the current
			// r and t values are r[i+1] and t[i+1], respectively.

			// a1 = r[i+1], b1 = -t[i+1]
			a1.Set(r)
			b1.Neg(t)
			found = true
			oneMore = true

			// Skip to the next iteration so ri and ti are not
			// modified.
			continue

		} else if oneMore {
			// When this condition executes ri and ti still
			// represent the r[i] and t[i] values while the current
			// r and t are r[i+2] and t[i+2], respectively.

			// sum1 = r[i]^2 + t[i]^2
			rSquared := new(big.Int).Mul(ri, ri)
			tSquared := new(big.Int).Mul(ti, ti)
			sum1 := new(big.Int).Add(rSquared, tSquared)

			// sum2 = r[i+2]^2 + t[i+2]^2
			r2Squared := new(big.Int).Mul(r, r)
			t2Squared := new(big.Int).Mul(t, t)
			sum2 := new(big.Int).Add(r2Squared, t2Squared)

			// if (r[i]^2 + t[i]^2) <= (r[i+2]^2 + t[i+2]^2)
			if sum1.Cmp(sum2) <= 0 {
				// a2 = r[i], b2 = -t[i]
				a2.Set(ri)
				b2.Neg(ti)
			} else {
				// a2 = r[i+2], b2 = -t[i+2]
				a2.Set(r)
				b2.Neg(t)
			}

			// All done.
			break
		}

		ri.Set(r)
		ti.Set(t)
	}

	return a1, b1, a2, b2
}

// deriveEndomorphismParams calculates and returns parameters needed to make use
// of the secp256k1 endomorphism.
func deriveEndomorphismParams() [2]endomorphismParams {
	// roots returns the solutions of the characteristic polynomial of the
	// secp256k1 endomorphism.
	//
	// The characteristic polynomial for the endomorphism is:
	//
	// X^2 + X + 1 ≡ 0 (mod n).
	//
	// Solving for X:
	//
	// 4X^2 + 4X + 4 ≡ 0 (mod n)       | (*4, possible because gcd(4, n) = 1)
	// (2X + 1)^2 + 3 ≡ 0 (mod n)      | (factor by completing the square)
	// (2X + 1)^2 ≡ -3 (mod n)         | (-3)
	// (2X + 1) ≡ ±sqrt(-3) (mod n)    | (sqrt)
	// 2X ≡ ±sqrt(-3) - 1 (mod n)      | (-1)
	// X ≡ (±sqrt(-3)-1)*2^-1 (mod n)  | (*2^-1)
	//
	// So, the roots are:
	// X1 ≡ (-(sqrt(-3)+1)*2^-1 (mod n)
	// X2 ≡ (sqrt(-3)-1)*2^-1 (mod n)
	//
	// It is also worth noting that X is a cube root of unity, meaning
	// X^3 - 1 ≡ 0 (mod n), hence it can be factored as (X - 1)(X^2 + X + 1) ≡ 0
	// and (X1)^2 ≡ X2 (mod n) and (X2)^2 ≡ X1 (mod n).
	roots := func(prime *big.Int) [2]big.Int {
		var result [2]big.Int
		one := big.NewInt(1)
		twoInverse := new(big.Int).ModInverse(big.NewInt(2), prime)
		negThree := new(big.Int).Neg(big.NewInt(3))
		sqrtNegThree := new(big.Int).ModSqrt(negThree, prime)
		sqrtNegThreePlusOne := new(big.Int).Add(sqrtNegThree, one)
		negSqrtNegThreePlusOne := new(big.Int).Neg(sqrtNegThreePlusOne)
		result[0].Mul(negSqrtNegThreePlusOne, twoInverse)
		result[0].Mod(&result[0], prime)
		sqrtNegThreeMinusOne := new(big.Int).Sub(sqrtNegThree, one)
		result[1].Mul(sqrtNegThreeMinusOne, twoInverse)
		result[1].Mod(&result[1], prime)
		return result
	}

	// Find the λ's and β's which are the solutions for the characteristic
	// polynomial of the secp256k1 endomorphism modulo the curve order and
	// modulo the field order, respectively.
	lambdas := roots(curveParams.N)
	betas := roots(curveParams.P)

	// Ensure the calculated roots are actually the roots of the characteristic
	// polynomial.
	checkRoots := func(foundRoots [2]big.Int, prime *big.Int) {
		// X^2 + X + 1 ≡ 0 (mod p)
		one := big.NewInt(1)
		for i := 0; i < len(foundRoots); i++ {
			root := &foundRoots[i]
			result := new(big.Int).Mul(root, root)
			result.Add(result, root)
			result.Add(result, one)
			result.Mod(result, prime)
			if result.Sign() != 0 {
				panic(fmt.Sprintf("%[1]x^2 + %[1]x + 1 != 0 (mod %x)", root,
					prime))
			}
		}
	}
	checkRoots(lambdas, curveParams.N)
	checkRoots(betas, curveParams.P)

	// checkVectors ensures the passed vectors satisfy the equation:
	// a + b*λ ≡ 0 (mod n)
	checkVectors := func(a, b *big.Int, lambda *big.Int) {
		result := new(big.Int).Mul(b, lambda)
		result.Add(result, a)
		result.Mod(result, curveParams.N)
		if result.Sign() != 0 {
			panic(fmt.Sprintf("%x + %x*lambda != 0 (mod %x)", a, b,
				curveParams.N))
		}
	}

	var endoParams [2]endomorphismParams
	for i := 0; i < 2; i++ {
		// Calculate the linearly independent vectors needed to generate a
		// balanced length-two representation of a scalar such that
		// k = k1 + k2*λ (mod n) for each of the solutions.
		lambda := &lambdas[i]
		a1, b1, a2, b2 := endomorphismVectors(lambda)

		// Ensure the derived vectors satisfy the required equation.
		checkVectors(a1, b1, lambda)
		checkVectors(a2, b2, lambda)

		// Calculate the precomputed estimates also used when generating the
		// aforementioned decomposition.
		//
		// z1 = floor(b2<<320 / n)
		// z2 = floor(((-b1)%n)<<320) / n)
		const shift = 320
		z1 := new(big.Int).Lsh(b2, shift)
		z1.Div(z1, curveParams.N)
		z2 := new(big.Int).Neg(b1)
		z2.Lsh(z2, shift)
		z2.Div(z2, curveParams.N)

		params := &endoParams[i]
		params.lambda = lambda
		params.beta = &betas[i]
		params.a1 = a1
		params.b1 = b1
		params.a2 = a2
		params.b2 = b2
		params.z1 = z1
		params.z2 = z2
	}

	return endoParams
}

func main() {
	fi, err := os.Create("compressedbytepoints.go")
	if err != nil {
		log.Fatal(err)
	}
	defer fi.Close()

	// Compress the serialized byte points.
	serialized := serializedBytePoints()
	var compressed bytes.Buffer
	w := zlib.NewWriter(&compressed)
	if _, err := w.Write(serialized); err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
	w.Close()

	// Encode the compressed byte points with base64.
	encoded := make([]byte, base64.StdEncoding.EncodedLen(compressed.Len()))
	base64.StdEncoding.Encode(encoded, compressed.Bytes())

	fmt.Fprintln(fi, "// Copyright (c) 2015 The btcsuite developers")
	fmt.Fprintln(fi, "// Copyright (c) 2015-2022 The Decred developers")
	fmt.Fprintln(fi, "// Use of this source code is governed by an ISC")
	fmt.Fprintln(fi, "// license that can be found in the LICENSE file.")
	fmt.Fprintln(fi)
	fmt.Fprintln(fi, "package secp256k1")
	fmt.Fprintln(fi)
	fmt.Fprintln(fi, "// Auto-generated file (see genprecomps.go)")
	fmt.Fprintln(fi, "// DO NOT EDIT")
	fmt.Fprintln(fi)
	fmt.Fprintf(fi, "var compressedBytePoints = %q\n", string(encoded))
	fmt.Fprintln(fi)
	fmt.Fprintln(fi, "// Set accessor to a real function.")
	fmt.Fprintln(fi, "func init() {")
	fmt.Fprintln(fi, "	compressedBytePointsFn = func() string {")
	fmt.Fprintln(fi, "		return compressedBytePoints")
	fmt.Fprintln(fi, "	}")
	fmt.Fprintln(fi, "}")

	printParams := func(p *endomorphismParams) {
		fmt.Printf("lambda: %x\n", p.lambda)
		fmt.Printf("  beta: %x\n", p.beta)
		fmt.Printf("    a1: %x\n", p.a1)
		fmt.Printf("    b1: %x\n", p.b1)
		fmt.Printf("    a2: %x\n", p.a2)
		fmt.Printf("    b2: %x\n", p.b2)
		fmt.Printf("    z1: %x\n", p.z1)
		fmt.Printf("    z2: %x\n", p.z2)
	}
	endoParams := deriveEndomorphismParams()
	fmt.Println("The following are the computed values to make use of the " +
		"secp256k1 endomorphism.\nThey consist of the lambda and beta " +
		"values along with the associated linearly independent vectors " +
		"(a1, b1, a2, b2) and estimates (z1, z2) used when decomposing " +
		"scalars:")
	printParams(&endoParams[0])
	fmt.Println()

	fmt.Println("Alternatively, the following parameters are valid as well:")
	printParams(&endoParams[1])
}
