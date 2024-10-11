// Copyright 2020 ConsenSys Software Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Package starkcurve efficient elliptic curve implementation for stark_curve. This is curve used in StarkNet: https://docs.starkware.co/starkex/crypto/stark-curve.html.
//
// stark_curve: A j!=0 curve with
//
//	𝔽r: r=3618502788666131213697322783095070105526743751716087489154079457884512865583
//	𝔽p: p=3618502788666131213697322783095070105623107215331596699973092056135872020481 (2^251+17*2^192+1)
//	(E/𝔽p): Y²=X³+x+b where b=3141592653589793238462643383279502884197169399375105820974944592307816406665
//
// Security: estimated 126-bit level using Pollard's \rho attack
// (r is 252 bits)
//
// # Warning
//
// This code has been partially audited and is provided as-is. In particular, there is no security guarantees such as constant time implementation or side-channel attack resistance.
package starkcurve

import (
	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark-crypto/ecc/stark-curve/fp"
)

// ID stark_curve ID
const ID = ecc.STARK_CURVE

// aCurveCoeff is the a coefficients of the curve Y²=X³+ax+b
var aCurveCoeff fp.Element
var bCurveCoeff fp.Element

// generator of the r-torsion group
var g1Gen G1Jac

var g1GenAff G1Affine

// point at infinity
var g1Infinity G1Jac

func init() {
	aCurveCoeff.SetUint64(1)
	bCurveCoeff.SetString("3141592653589793238462643383279502884197169399375105820974944592307816406665")

	g1Gen.X.SetString("874739451078007766457464989774322083649278607533249481151382481072868806602")
	g1Gen.Y.SetString("152666792071518830868575557812948353041420400780739481342941381225525861407")
	g1Gen.Z.SetOne()

	g1GenAff.FromJacobian(&g1Gen)

	// (X,Y,Z) = (1,1,0)
	g1Infinity.X.SetOne()
	g1Infinity.Y.SetOne()

}

// Generators return the generators of the r-torsion group, resp. in ker(pi-id), ker(Tr)
func Generators() (g1Jac G1Jac, g1Aff G1Affine) {
	g1Aff = g1GenAff
	g1Jac = g1Gen
	return
}

// CurveCoefficients returns the a, b coefficients of the curve equation.
func CurveCoefficients() (a, b fp.Element) {
	return aCurveCoeff, bCurveCoeff
}
