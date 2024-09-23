// Package bw6633 efficient elliptic curve, pairing and hash to curve implementation for bw6-633.
//
// bw6-633: A Brezing--Weng curve (2-chain with bls24-315)
//
//	embedding degree k=6
//	seed x₀=-3218079743
//	𝔽p: p=20494478644167774678813387386538961497669590920908778075528754551012016751717791778743535050360001387419576570244406805463255765034468441182772056330021723098661967429339971741066259394985997
//	𝔽r: r=39705142709513438335025689890408969744933502416914749335064285505637884093126342347073617133569
//	(E/𝔽p): Y²=X³+4
//	(Eₜ/𝔽p): Y² = X³+8 (M-type twist)
//	r ∣ #E(Fp) and r ∣ #Eₜ(𝔽p)
//
// Extension fields tower:
//
//	𝔽p³[u] = 𝔽p/u³-2
//	𝔽p⁶[v] = 𝔽p³/v²-u
//
// case t % r % u = 0
//
// optimal Ate loops:
//
//	x₀+1, x₀^5-x₀^4-x₀
//
// Security: estimated 124-bit level following [https://eprint.iacr.org/2019/885.pdf]
// (r is 315 bits and p⁶ is 3798 bits)
//
// # Warning
//
// This code has not been audited and is provided as-is. In particular, there is no security guarantees such as constant time implementation or side-channel attack resistance.
package bw6633

import (
	"math/big"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark-crypto/ecc/bw6-633/fp"
	"github.com/consensys/gnark-crypto/ecc/bw6-633/fr"
)

// ID BW6_633 ID
const ID = ecc.BW6_633

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
var LoopCounter [159]int8
var LoopCounter1 [159]int8

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

// seed -x₀ of the curve
var xGen big.Int

func init() {
	aCurveCoeff.SetUint64(0)
	bCurveCoeff.SetUint64(4)
	bTwistCurveCoeff.SetUint64(8) // M-twist

	// E1(2,y)*cofactor
	g1Gen.X.SetString("14087405796052437206213362229855313116771222912153372774869400386285407949123477431442535997951698710614498307938219633856996133201713506830167161540335446217605918678317160130862890417553415")
	g1Gen.Y.SetString("5208886161111258314476333487866604447704068601830026647530443033297117148121067806438008469463787158470000157308702133756065259580313172904438248825389121766442385979570644351664733475122746")
	g1Gen.Z.SetOne()

	// E2(2,y))*cofactor
	g2Gen.X.SetString("13658793733252505713431834233072715040674666715141692574468286839081203251180283741830175712695426047062165811313478642863696265647598838732554425602399576125615559121457137320131899043374497")
	g2Gen.Y.SetString("599560264833409786573595720823495699033661029721475252751314180543773745554433461106678360045466656230822473390866244089461950086268801746497554519984580043036179195728559548424763890207250")
	g2Gen.Z.SetOne()

	g1GenAff.FromJacobian(&g1Gen)
	g2GenAff.FromJacobian(&g2Gen)

	// binary decomposition of x₀+1 (negative)
	LoopCounter = [159]int8{0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, -1, 0, 0, 0, 0, 0, 0, 0, -1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}

	// x₀⁵-x₀⁴-x₀ (negative)
	T, _ := new(big.Int).SetString("345131030376204096837580131803633448876874137601", 10)
	ecc.NafDecomposition(T, LoopCounter1[:])

	// (X,Y,Z) = (1,1,0)
	g1Infinity.X.SetOne()
	g1Infinity.Y.SetOne()
	g2Infinity.X.SetOne()
	g2Infinity.Y.SetOne()

	thirdRootOneG1.SetString("4098895725012429242072311240482566844345873033931481129362557724405008256668293241245050359832461015092695507587185678086043587575438449040313411246717257958467499181450742260777082884928318") // (45-10*x+151*x²-187*x³+171*x⁴-49*x⁵-110*x⁶+430*x⁷-696*x⁸+702*x⁹-528*x¹⁰+201*x¹¹+144*x¹²-274*x¹³+181*x¹⁴-34*x¹⁵-63*x¹⁶+92*x¹⁷-56*x¹⁸+13*x¹⁹)/15
	thirdRootOneG2.Square(&thirdRootOneG1)
	lambdaGLV.SetString("39705142672498995661671850106945620852186608752525090699191017895721506694646055668218723303426", 10) // 1-x+2*x²-2*x³+3*x⁵-4*x⁶+4*x⁷-3*x⁸+x⁹
	_r := fr.Modulus()
	ecc.PrecomputeLattice(_r, &lambdaGLV, &glvBasis)

	// -x₀
	xGen.SetString("3218079743", 10) // negative

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
