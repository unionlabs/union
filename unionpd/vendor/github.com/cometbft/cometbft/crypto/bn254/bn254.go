package bn254

import (
	"bytes"
	"crypto/hmac"
	"crypto/rand"
	"crypto/subtle"
	"fmt"
	"math/big"

	"golang.org/x/crypto/sha3"

	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fp"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
	bls254 "github.com/consensys/gnark-crypto/ecc/bn254/signature/bls"

	"github.com/cometbft/cometbft/crypto"
	cmtjson "github.com/cometbft/cometbft/libs/json"
	"github.com/holiman/uint256"
)

const (
	PrivKeyName    = "tendermint/PrivKeyBn254"
	PubKeyName     = "tendermint/PubKeyBn254"
	KeyType        = "bn254"
	PubKeySize     = sizePublicKey
	PrivKeySize    = sizePrivateKey
	sizeFr         = fr.Bytes
	sizeFp         = fp.Bytes
	sizePublicKey  = sizeFp
	sizePrivateKey = sizeFr + sizePublicKey
)

var _ crypto.PrivKey = PrivKey{}

type PrivKey []byte

func (PrivKey) TypeTag() string { return PrivKeyName }

func (privKey PrivKey) Bytes() []byte {
	return []byte(privKey)
}

// Signature is compressed!
func (privKey PrivKey) Sign(msg []byte) ([]byte, error) {
	s := new(big.Int)
	s = s.SetBytes(privKey)
	point := hashToG2(msg)
	var p bn254.G2Affine
	p.ScalarMultiplication(&point, s)
	compressedSig := p.Bytes()
	return compressedSig[:], nil
}

func (privKey PrivKey) PubKey() crypto.PubKey {
	s := new(big.Int)
	s.SetBytes(privKey)
	var pk bn254.G1Affine
	pk.ScalarMultiplication(&G1Base, s)
	pkBytes := pk.Bytes()
	return PubKey(pkBytes[:])
}

func (privKey PrivKey) Equals(other crypto.PrivKey) bool {
	if otherEd, ok := other.(PrivKey); ok {
		return subtle.ConstantTimeCompare(privKey[:], otherEd[:]) == 1
	}
	return false
}

func (privKey PrivKey) Type() string {
	return KeyType
}

var _ crypto.PubKey = PubKey{}

type PubKey []byte

func (PubKey) TypeTag() string { return PubKeyName }

// Raw public key
func (pubKey PubKey) Address() crypto.Address {
	return crypto.AddressHash(pubKey[:])
}

// Bytes returns the PubKey byte format.
func (pubKey PubKey) Bytes() []byte {
	return pubKey
}

func (pubKey PubKey) VerifySignature(msg []byte, sig []byte) bool {
	hashedMessage := hashToG2(msg)
	var public bn254.G1Affine
	_, err := public.SetBytes(pubKey)
	if err != nil {
		return false
	}

	var signature bn254.G2Affine
	_, err = signature.SetBytes(sig)
	if err != nil {
		return false
	}

	var G1BaseNeg bn254.G1Affine
	G1BaseNeg.Neg(&G1Base)

	valid, err := bn254.PairingCheck([]bn254.G1Affine{G1BaseNeg, public}, []bn254.G2Affine{signature, hashedMessage})
	if err != nil {
		return false
	}
	return valid
}

func (pubKey PubKey) String() string {
	return fmt.Sprintf("PubKeyBn254{%X}", []byte(pubKey[:]))
}

func (pubKey PubKey) Type() string {
	return KeyType
}

func (pubKey PubKey) Equals(other crypto.PubKey) bool {
	if otherEd, ok := other.(PubKey); ok {
		return bytes.Equal(pubKey[:], otherEd[:])
	}
	return false
}

func GenPrivKey() PrivKey {
	secret, err := bls254.GenerateKey(rand.Reader)
	if err != nil {
		panic("bro")
	}
	return PrivKey(secret.Bytes())
}

var G1Base bn254.G1Affine
var G2Base bn254.G2Affine
var G2Cofactor big.Int

var Hash = sha3.NewLegacyKeccak256

func init() {
	cmtjson.RegisterType(PubKey{}, PubKeyName)
	cmtjson.RegisterType(PrivKey{}, PrivKeyName)

	_, _, G1Base, G2Base = bn254.Generators()

	value, err := new(big.Int).SetString("30644e72e131a029b85045b68181585e06ceecda572a2489345f2299c0f9fa8d", 16)
	if !err {
		panic("Cannot build cofactor")
	}
	G2Cofactor.Set(value)
}

// TODO: remove when https://github.com/ConsenSys/gnark-crypto/issues/373 is fixed
func mapToCurve2(u *bn254.E2) bn254.G2Affine {

	var tv1, tv2, tv3, tv4 bn254.E2
	var x1, x2, x3, gx1, gx2, gx, x, y bn254.E2
	var one bn254.E2
	var gx1NotSquare, gx1SquareOrGx2Not int

	//constants
	//c1 = g(Z)
	//c2 = -Z / 2
	//c3 = sqrt(-g(Z) * (3 * Z² + 4 * A))     # sgn0(c3) MUST equal 0
	//c4 = -4 * g(Z) / (3 * Z² + 4 * A)

	Z := bn254.E2{
		A0: fp.Element{15230403791020821917, 754611498739239741, 7381016538464732716, 1011752739694698287},
		A1: fp.Element{0},
	}
	c1 := bn254.E2{
		A0: fp.Element{15219334786797146878, 8431472696017589261, 15336528771359260718, 196732871012706162},
		A1: fp.Element{4100506350182530919, 7345568344173317438, 15513160039642431658, 90557763186888013},
	}
	c2 := bn254.E2{
		A0: fp.Element{12997850613838968789, 14304628359724097447, 2950087706404981016, 1237622763554136189},
		A1: fp.Element{0},
	}
	c3 := bn254.E2{
		A0: fp.Element{12298500088583694207, 17447120171744064890, 14097510924717921191, 2278398337453771183},
		A1: fp.Element{4693446565795584099, 18320164443970680666, 6792758484113206563, 2989688171181581768},
	}
	c4 := bn254.E2{
		A0: fp.Element{7191623630069643826, 8333948550768170742, 13001081703983517696, 2062355016518372226},
		A1: fp.Element{11163104453509316115, 7271947710149976975, 4894807947557820282, 3366254582553786647},
	}

	var bCurveCoeff fp.Element

	// twist
	var twist bn254.E2

	// bTwistCurveCoeff b coeff of the twist (defined over 𝔽p²) bn254
	var bTwistCurveCoeff bn254.E2

	bCurveCoeff.SetUint64(3)
	// D-twist
	twist.A0.SetUint64(9)
	twist.A1.SetUint64(1)
	bTwistCurveCoeff.Inverse(&twist).MulByElement(&bTwistCurveCoeff, &bCurveCoeff)

	one.SetOne()

	tv1.Square(u)       //    1.  tv1 = u²
	tv1.Mul(&tv1, &c1)  //    2.  tv1 = tv1 * c1
	tv2.Add(&one, &tv1) //    3.  tv2 = 1 + tv1
	tv1.Sub(&one, &tv1) //    4.  tv1 = 1 - tv1
	tv3.Mul(&tv1, &tv2) //    5.  tv3 = tv1 * tv2

	tv3.Inverse(&tv3)   //    6.  tv3 = inv0(tv3)
	tv4.Mul(u, &tv1)    //    7.  tv4 = u * tv1
	tv4.Mul(&tv4, &tv3) //    8.  tv4 = tv4 * tv3
	tv4.Mul(&tv4, &c3)  //    9.  tv4 = tv4 * c3
	x1.Sub(&c2, &tv4)   //    10.  x1 = c2 - tv4

	gx1.Square(&x1) //    11. gx1 = x1²
	//12. gx1 = gx1 + A     All curves in gnark-crypto have A=0 (j-invariant=0). It is crucial to include this step if the bn254 has nonzero A coefficient.
	gx1.Mul(&gx1, &x1)                 //    13. gx1 = gx1 * x1
	gx1.Add(&gx1, &bTwistCurveCoeff)   //    14. gx1 = gx1 + B
	gx1NotSquare = gx1.Legendre() >> 1 //    15.  e1 = is_square(gx1)
	// gx1NotSquare = 0 if gx1 is a square, -1 otherwise

	x2.Add(&c2, &tv4) //    16.  x2 = c2 + tv4
	gx2.Square(&x2)   //    17. gx2 = x2²
	//    18. gx2 = gx2 + A     See line 12
	gx2.Mul(&gx2, &x2)               //    19. gx2 = gx2 * x2
	gx2.Add(&gx2, &bTwistCurveCoeff) //    20. gx2 = gx2 + B

	{
		gx2NotSquare := gx2.Legendre() >> 1              // gx2Square = 0 if gx2 is a square, -1 otherwise
		gx1SquareOrGx2Not = gx2NotSquare | ^gx1NotSquare //    21.  e2 = is_square(gx2) AND NOT e1   # Avoid short-circuit logic ops
	}

	x3.Square(&tv2)   //    22.  x3 = tv2²
	x3.Mul(&x3, &tv3) //    23.  x3 = x3 * tv3
	x3.Square(&x3)    //    24.  x3 = x3²
	x3.Mul(&x3, &c4)  //    25.  x3 = x3 * c4

	x3.Add(&x3, &Z)                  //    26.  x3 = x3 + Z
	x.Select(gx1NotSquare, &x1, &x3) //    27.   x = CMOV(x3, x1, e1)   # x = x1 if gx1 is square, else x = x3
	// Select x1 iff gx1 is square iff gx1NotSquare = 0
	x.Select(gx1SquareOrGx2Not, &x2, &x) //    28.   x = CMOV(x, x2, e2)    # x = x2 if gx2 is square and gx1 is not
	// Select x2 iff gx2 is square and gx1 is not, iff gx1SquareOrGx2Not = 0
	gx.Square(&x) //    29.  gx = x²
	//    30.  gx = gx + A

	gx.Mul(&gx, &x)                //    31.  gx = gx * x
	gx.Add(&gx, &bTwistCurveCoeff) //    32.  gx = gx + B

	y.Sqrt(&gx)                             //    33.   y = sqrt(gx)
	signsNotEqual := g2Sgn0(u) ^ g2Sgn0(&y) //    34.  e3 = sgn0(u) == sgn0(y)

	tv1.Neg(&y)
	y.Select(int(signsNotEqual), &y, &tv1) //    35.   y = CMOV(-y, y, e3)       # Select correct sign of y

	return bn254.G2Affine{X: x, Y: y}
}

func g1NotZero(x *fp.Element) uint64 {
	return x[0] | x[1] | x[2] | x[3]
}

// g2Sgn0 is an algebraic substitute for the notion of sign in ordered fields
// Namely, every non-zero quadratic residue in a finite field of characteristic =/= 2 has exactly two square roots, one of each sign
// https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-bn254-16.html#name-the-sgn0-function
// The sign of an element is not obviously related to that of its Montgomery form
func g2Sgn0(z *bn254.E2) uint64 {

	nonMont := z.Bits()

	sign := uint64(0) // 1. sign = 0
	zero := uint64(1) // 2. zero = 1
	var signI uint64
	var zeroI uint64

	// 3. i = 1
	signI = nonMont.A0[0] % 2 // 4.   sign_i = x_i mod 2
	zeroI = g1NotZero(&nonMont.A0)
	zeroI = 1 ^ (zeroI|-zeroI)>>63 // 5.   zero_i = x_i == 0
	sign = sign | (zero & signI)   // 6.   sign = sign OR (zero AND sign_i) # Avoid short-circuit logic ops
	zero = zero & zeroI            // 7.   zero = zero AND zero_i
	// 3. i = 2
	signI = nonMont.A1[0] % 2 // 4.   sign_i = x_i mod 2
	// 5.   zero_i = x_i == 0
	sign = sign | (zero & signI) // 6.   sign = sign OR (zero AND sign_i) # Avoid short-circuit logic ops
	// 7.   zero = zero AND zero_i
	return sign

}

// Naive scalar multiplication used for cofactor clearing
func nativeNaiveScalarMul(p bn254.G2Affine, s *big.Int) bn254.G2Affine {
	// initialize result point to infinity
	var result bn254.G2Affine
	result.X.SetZero()
	result.Y.SetZero()
	bits := s.BitLen()
	// iterate over binary digits of s and double the current result point at each iteration
	for i := bits - 1; i >= 0; i-- {
		result.Add(&result, &result)
		// if current binary digit is 1, add the original point p to the result
		if s.Bit(i) == 1 {
			result.Add(&result, &p)
		}
	}
	return result
}

// Custom function: (hmac_keccak(msg) mod (p - 1)) + 1
func hashToField(msg []byte) fp.Element {
	hmac := hmac.New(Hash, []byte("CometBLS"))
	hmac.Write(msg)
	modMinusOne := new(big.Int).Sub(fp.Modulus(), big.NewInt(1))
	num := new(big.Int).SetBytes(hmac.Sum(nil))
	num.Mod(num, modMinusOne)
	num.Add(num, big.NewInt(1))
	val, overflow := uint256.FromBig(num)
	if overflow {
		panic("impossible; qed;")
	}
	valBytes := val.Bytes32()
	var element fp.Element
	err := element.SetBytesCanonical(valBytes[:])
	if err != nil {
		panic("impossible; qed;")
	}
	return element
}

func hashToField2(msg []byte) bn254.E2 {
	e0 := hashToField(append([]byte{0}, msg...))
	e1 := hashToField(append([]byte{1}, msg...))
	return bn254.E2{A0: e0, A1: e1}
}

func hashToG2(msg []byte) bn254.G2Affine {
	e := hashToField2(msg)
	point := nativeNaiveScalarMul(mapToCurve2(&e), &G2Cofactor)
	if !point.IsOnCurve() {
		panic("Point is not on the curve")
	}
	if !point.IsInSubGroup() {
		panic("Point is not in subgroup")
	}
	if point.IsInfinity() {
		panic("Point is zero")
	}
	return point
}
