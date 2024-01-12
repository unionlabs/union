// Package bls12378 efficient elliptic curve, pairing and hash to curve implementation for bls12-378.
//
// bls12-378: A Barreto--Lynn--Scott curve
//
//	embedding degree k=12
//	seed x₀=11045256207009841153
//	𝔽r: r=14883435066912132899950318861128167269793560281114003360875131245101026639873 (x₀⁴-x₀²+1)
//	𝔽p: p=605248206075306171733248481581800960739847691770924913753520744034740935903401304776283802348837311170974282940417 ((x₀-1)² ⋅ r(x₀)/3+x₀)
//	(E/𝔽p): Y²=X³+1
//	(Eₜ/𝔽p²): Y² = X³+u (M-type twist)
//	r ∣ #E(Fp) and r ∣ #Eₜ(𝔽p²)
//
// Extension fields tower:
//
//	𝔽p²[u] = 𝔽p/u²+5
//	𝔽p⁶[v] = 𝔽p²/v³-u
//	𝔽p¹²[w] = 𝔽p⁶/w²-v
//
// optimal Ate loop size:
//
//	x₀
//
// Security: estimated 126-bit level following [https://eprint.iacr.org/2019/885.pdf]
// (r is 254 bits and p¹² is 4536 bits)
//
// # Warning
//
// This code has not been audited and is provided as-is. In particular, there is no security guarantees such as constant time implementation or side-channel attack resistance.
package bls12378

import (
	"math/big"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark-crypto/ecc/bls12-378/fp"
	"github.com/consensys/gnark-crypto/ecc/bls12-378/fr"
	"github.com/consensys/gnark-crypto/ecc/bls12-378/internal/fptower"
)

// ID bls378 ID
const ID = ecc.BLS12_378

// aCurveCoeff is the a coefficients of the curve Y²=X³+ax+b
var aCurveCoeff fp.Element
var bCurveCoeff fp.Element

// bTwistCurveCoeff b coeff of the twist (defined over 𝔽p²) curve
var bTwistCurveCoeff fptower.E2

// generators of the r-torsion group, resp. in ker(pi-id), ker(Tr)
var g1Gen G1Jac
var g2Gen G2Jac

var g1GenAff G1Affine
var g2GenAff G2Affine

// point at infinity
var g1Infinity G1Jac
var g2Infinity G2Jac

// optimal Ate loop counter
var LoopCounter [64]int8

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

// ψ o π o ψ⁻¹, where ψ:E → E' is the degree 6 iso defined over 𝔽p¹²
var endo struct {
	u fptower.E2
	v fptower.E2
}

// seed x₀ of the curve
var xGen big.Int

// expose the tower -- github.com/consensys/gnark uses it in a gnark circuit

// 𝔽p²
type E2 = fptower.E2

// 𝔽p⁶
type E6 = fptower.E6

// 𝔽p¹²
type E12 = fptower.E12

func init() {
	aCurveCoeff.SetUint64(0)
	bCurveCoeff.SetUint64(1)
	bTwistCurveCoeff.A1.SetUint64(1) // M-twist

	// E(3,y) * cofactor
	g1Gen.X.SetString("302027100877540500544138164010696035562809807233645104772290911818386302983750063098216015456036850656714568735197")
	g1Gen.Y.SetString("232851047397483214541821965369374725182070455016459237170823497053622811786333462699984177726412751508198874482530")
	g1Gen.Z.SetOne()

	// E_t(1,y) * cofactor'
	g2Gen.X.SetString("470810816643554779222760025249941413452299198622737082648784137654933833261310635469274149014014206108405592809732",
		"317092959336227428400228502739777439718827088477410533227996105067347670094088101088421556743730925535231685964487")
	g2Gen.Y.SetString("248853758964950314624408411876149087897475217517523838449839260719963153199419627931373025216041741725848318074460",
		"389162134924826972299508957175841717907876177152103852864177212390074067430801162403069988146334006672491106545644")
	g2Gen.Z.SetString("1",
		"0")

	g1GenAff.FromJacobian(&g1Gen)
	g2GenAff.FromJacobian(&g2Gen)

	// (X,Y,Z) = (1,1,0)
	g1Infinity.X.SetOne()
	g1Infinity.Y.SetOne()
	g2Infinity.X.SetOne()
	g2Infinity.Y.SetOne()

	thirdRootOneG1.SetString("164391353554439166353793911729193406645071739502673898176639736370075683438438023898983435337729")
	thirdRootOneG2.Square(&thirdRootOneG1)
	lambdaGLV.SetString("121997684678489422961514670190292369408", 10) //(x₀²-1)
	_r := fr.Modulus()
	ecc.PrecomputeLattice(_r, &lambdaGLV, &glvBasis)

	endo.u.A0.SetString("164391353554439166353793911729193406645071739502673898176639736370075683438438023898983435337730")
	endo.v.A0.SetString("595603361117066405543541008735167904222384847192046901135681663787023479658010166685728902742824780272831835669219")

	// binary decomposition of x₀ little endian
	LoopCounter = [64]int8{1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 1, 0, 0, 1}

	// x₀
	xGen.SetString("11045256207009841153", 10)

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
