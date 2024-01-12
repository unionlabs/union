// Package bw6761 efficient elliptic curve, pairing and hash to curve implementation for bw6-761.
//
// bw6-761: A Brezing--Weng curve (2-chain with bls12-377)
//
//	embedding degree k=6
//	seed x₀=9586122913090633729
//	𝔽p: p=6891450384315732539396789682275657542479668912536150109513790160209623422243491736087683183289411687640864567753786613451161759120554247759349511699125301598951605099378508850372543631423596795951899700429969112842764913119068299
//	𝔽r: r=258664426012969094010652733694893533536393512754914660539884262666720468348340822774968888139573360124440321458177
//	(E/𝔽p): Y²=X³-1
//	(Eₜ/𝔽p): Y² = X³+4 (M-type twist)
//	r ∣ #E(Fp) and r ∣ #Eₜ(𝔽p)
//
// case t % r % x₀ = 3
//
// Extension fields tower:
//
//	𝔽p³[u] = 𝔽p/u³+4
//	𝔽p⁶[v] = 𝔽p²/v²-u
//
// optimal Ate loops:
//
//	x₀+1, x₀²-x₀-1
//
// Security: estimated 126-bit level following [https://eprint.iacr.org/2019/885.pdf]
// (r is 377 bits and p⁶ is 4566 bits)
//
// https://eprint.iacr.org/2020/351.pdf
//
// # Warning
//
// This code has not been audited and is provided as-is. In particular, there is no security guarantees such as constant time implementation or side-channel attack resistance.
package bw6761

import (
	"math/big"

	"github.com/consensys/gnark-crypto/ecc/bw6-761/internal/fptower"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark-crypto/ecc/bw6-761/fp"
	"github.com/consensys/gnark-crypto/ecc/bw6-761/fr"
)

// ID BW6_761 ID
const ID = ecc.BW6_761

// aCurveCoeff is the a coefficients of the curve Y²=X³+ax+b
var aCurveCoeff fp.Element
var bCurveCoeff fp.Element

// bTwistCurveCoeff b coeff of the twist (defined over 𝔽p) curve
var bTwistCurveCoeff fp.Element

// generators of the r-torsion group, resp. in ker(pi-id), ker(Tr)
var g1Gen G1Jac
var g2Gen G2Jac

var g1GenAff G1Affine
var g2GenAff G2Affine

// point at infinity
var g1Infinity G1Jac
var g2Infinity G2Jac

// optimal Ate loop counters
var LoopCounter [190]int8
var LoopCounter1 [190]int8

// Parameters useful for the GLV scalar multiplication. The third roots define the
// endomorphisms ϕ₁ and ϕ₂ for <G1Affine> and <G2Affine>. lambda is such that <r, ϕ-λ> lies above
// <r> in the ring Z[ϕ]. More concretely it's the associated eigenvalue
// of ϕ₁ (resp ϕ₂) restricted to <G1Affine> (resp <G2Affine>)
// see https://www.cosic.esat.kuleuven.be/nessie/reports/phase2/GLV.pdf
var thirdRootOneG1 fp.Element
var thirdRootOneG2 fp.Element
var lambdaGLV big.Int

// glvBasis stores R-linearly independent vectors (a,b), (c,d)
// in ker((u,v) → u+vλ[r]), and their determinant
var glvBasis ecc.Lattice

// seed x₀ of the curve
var xGen big.Int

// 𝔽p3
type E3 = fptower.E3

// 𝔽p6
type E6 = fptower.E6

func init() {
	aCurveCoeff.SetUint64(0)
	bCurveCoeff.SetOne().Neg(&bCurveCoeff)
	// M-twist
	bTwistCurveCoeff.SetUint64(4)

	g1Gen.X.SetString("6238772257594679368032145693622812838779005809760824733138787810501188623461307351759238099287535516224314149266511977132140828635950940021790489507611754366317801811090811367945064510304504157188661901055903167026722666149426237")
	g1Gen.Y.SetString("2101735126520897423911504562215834951148127555913367997162789335052900271653517958562461315794228241561913734371411178226936527683203879553093934185950470971848972085321797958124416462268292467002957525517188485984766314758624099")
	g1Gen.Z.SetOne()

	g2Gen.X.SetString("6445332910596979336035888152774071626898886139774101364933948236926875073754470830732273879639675437155036544153105017729592600560631678554299562762294743927912429096636156401171909259073181112518725201388196280039960074422214428")
	g2Gen.Y.SetString("562923658089539719386922163444547387757586534741080263946953401595155211934630598999300396317104182598044793758153214972605680357108252243146746187917218885078195819486220416605630144001533548163105316661692978285266378674355041")
	g2Gen.Z.SetOne()

	g1GenAff.FromJacobian(&g1Gen)
	g2GenAff.FromJacobian(&g2Gen)

	// x₀+1
	LoopCounter = [190]int8{0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, 0, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}

	// x₀³-x₀²-x₀
	T, _ := new(big.Int).SetString("880904806456922042166256752416502360955572640081583800319", 10)
	ecc.NafDecomposition(T, LoopCounter1[:])

	// (X,Y,Z) = (1,1,0)
	g1Infinity.X.SetOne()
	g1Infinity.Y.SetOne()
	g2Infinity.X.SetOne()
	g2Infinity.Y.SetOne()

	thirdRootOneG1.SetString("1968985824090209297278610739700577151397666382303825728450741611566800370218827257750865013421937292370006175842381275743914023380727582819905021229583192207421122272650305267822868639090213645505120388400344940985710520836292650")
	thirdRootOneG2.Square(&thirdRootOneG1)
	lambdaGLV.SetString("80949648264912719408558363140637477264845294720710499478137287262712535938301461879813459410945", 10) // (x⁵-3x⁴+3x³-x+1)
	_r := fr.Modulus()
	ecc.PrecomputeLattice(_r, &lambdaGLV, &glvBasis)

	// x₀
	xGen.SetString("9586122913090633729", 10)

}

// Generators return the generators of the r-torsion group, resp. in ker(pi-id), ker(Tr)
func Generators() (g1Jac G1Jac, g2Jac G2Jac, g1Aff G1Affine, g2Aff G2Affine) {
	g1Aff = g1GenAff
	g2Aff = g2GenAff
	g1Jac = g1Gen
	g2Jac = g2Gen
	return
}

// CurveCoefficients returns the a, b coefficients of the curve equation.
func CurveCoefficients() (a, b fp.Element) {
	return aCurveCoeff, bCurveCoeff
}
