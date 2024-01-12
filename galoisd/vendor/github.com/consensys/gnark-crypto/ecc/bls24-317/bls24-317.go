// Package bls24317 efficient elliptic curve, pairing and hash to curve implementation for bls24-317.
//
// bls24-317: A Barreto--Lynn--Scott curve
//
//	embedding degree k=24
//	seed x₀=3640754176
//	𝔽r: r=30869589236456844204538189757527902584594726589286811523515204428962673459201 (x₀^8-x₀^4+2)
//	𝔽p: p=136393071104295911515099765908274057061945112121419593977210139303905973197232025618026156731051 ((x₀-1)² ⋅ r(x₀)/3+x₀)
//	(E/𝔽p): Y²=X³+4
//	(Eₜ/𝔽p⁴): Y² = X³+4v (M-type twist)
//	r ∣ #E(Fp) and r ∣ #Eₜ(𝔽p⁴)
//
// Extension fields tower:
//
//	𝔽p²[u] = 𝔽p/u²+1
//	𝔽p⁴[v] = 𝔽p²/v²-u-1
//	𝔽p¹²[w] = 𝔽p⁴/w³-v
//	𝔽p²⁴[i] = 𝔽p¹²/i²-w
//
// optimal Ate loop size:
//
//	x₀
//
// Security: estimated 160-bit level following [https://eprint.iacr.org/2019/885.pdf]
// (r is 255 bits and p²⁴ is 7599 bits)
//
// # Warning
//
// This code has not been audited and is provided as-is. In particular, there is no security guarantees such as constant time implementation or side-channel attack resistance.
package bls24317

import (
	"math/big"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark-crypto/ecc/bls24-317/fp"
	"github.com/consensys/gnark-crypto/ecc/bls24-317/fr"
	"github.com/consensys/gnark-crypto/ecc/bls24-317/internal/fptower"
)

// ID bls317 ID
const ID = ecc.BLS24_317

// aCurveCoeff is the a coefficients of the curve Y²=X³+ax+b
var aCurveCoeff fp.Element
var bCurveCoeff fp.Element

// twist
var twist fptower.E4

// bTwistCurveCoeff b coeff of the twist (defined over 𝔽p⁴) curve
var bTwistCurveCoeff fptower.E4

// generators of the r-torsion group, resp. in ker(pi-id), ker(Tr)
var g1Gen G1Jac
var g2Gen G2Jac

var g1GenAff G1Affine
var g2GenAff G2Affine

// point at infinity
var g1Infinity G1Jac
var g2Infinity G2Jac

// optimal Ate loop counter
var LoopCounter [33]int8

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
	u fptower.E4
	v fptower.E4
}

// seed x₀ of the curve
var xGen big.Int

func init() {
	aCurveCoeff.SetUint64(0)
	bCurveCoeff.SetUint64(4)
	// M-twist
	twist.B1.SetOne()
	bTwistCurveCoeff.MulByElement(&twist, &bCurveCoeff)

	// E(1,y)*c
	g1Gen.X.SetString("26261810162995192444253184251590159762050205376519976412461726336843100448942248976252388876791")
	g1Gen.Y.SetString("26146603602820658047261036676090398397874822703333117264049387703172159980214065566219085800243")
	g1Gen.Z.SetOne()

	// E'(1,y)*c'
	g2Gen.X.B0.SetString("28498404142312365002533744693556861244212064443103687717510540998257508853975496760832205123607",
		"104881342316154169720140745551267577558255475983798552134082689646705436288255501236462500135051")
	g2Gen.X.B1.SetString("134208762611471838850128095341317427866582025424914361408168906642550705688378271974920859507485",
		"47807860684290705153036437491997319116342330273104493957877398921782737166446662055996604784294")
	g2Gen.Y.B0.SetString("91516448788529060702418635560646746547369142933278847722177434542449427480796649633689953798948",
		"13448671391015186163413673966297442264556781166352891049005282051703895543542296449974630011689")
	g2Gen.Y.B1.SetString("1980905665816458576882252418967038151483710575831277397652951146268622037800272983431026055487",
		"134363379072057086809745572347104070037544575425956896869689256737197090432635401300100624083192")
	g2Gen.Z.B0.SetString("1",
		"0")
	g2Gen.Z.B1.SetString("0",
		"0")

	g1GenAff.FromJacobian(&g1Gen)
	g2GenAff.FromJacobian(&g2Gen)

	// (X,Y,Z) = (1,1,0)
	g1Infinity.X.SetOne()
	g1Infinity.Y.SetOne()
	g2Infinity.X.SetOne()
	g2Infinity.Y.SetOne()

	thirdRootOneG1.SetString("112388585831426139305998878408983604164339968939599860577886592073045019257058155724801")
	thirdRootOneG2.Square(&thirdRootOneG1)
	lambdaGLV.SetString("30869589236456844204538189757527902584770424025911415822847175497150445387776", 10) // x₀⁸
	_r := fr.Modulus()
	ecc.PrecomputeLattice(_r, &lambdaGLV, &glvBasis)

	endo.u.B0.A0.SetString("100835231576138384070271140557450756773581004948002542492497192760544145876107391019725843007951")
	endo.u.B0.A1.SetString("100835231576138384070271140557450756773581004948002542492497192760544145876107391019725843007951")
	endo.v.B1.A0.SetString("65063930028143676778466901566890018271632055221368035552739808236464024322431728149960968101")
	endo.v.B1.A1.SetString("65063930028143676778466901566890018271632055221368035552739808236464024322431728149960968101")

	// 2-NAF decomposition of x₀ little endian
	optimaAteLoop, _ := new(big.Int).SetString("3640754176", 10)
	ecc.NafDecomposition(optimaAteLoop, LoopCounter[:])

	// x₀
	xGen.SetString("3640754176", 10)

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
