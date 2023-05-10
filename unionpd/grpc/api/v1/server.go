package grpc

import (
	"bufio"
	"bytes"
	"cometbls-prover/pkg/lightclient"
	lcgadget "cometbls-prover/pkg/lightclient/nonadjacent"
	context "context"
	"crypto/hmac"
	"fmt"
	"math/big"

	ce "github.com/cometbft/cometbft/crypto/encoding"
	"github.com/cometbft/cometbft/crypto/merkle"
	"github.com/cometbft/cometbft/libs/protoio"
	"github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/consensys/gnark-crypto/ecc"
	curve "github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fp"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
	backend "github.com/consensys/gnark/backend/groth16"
	backend_bn254 "github.com/consensys/gnark/backend/groth16/bn254"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	gadget "github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	"github.com/holiman/uint256"
	"golang.org/x/crypto/sha3"
)

type proverServer struct {
	UnimplementedUnionProverAPIServer
	pk backend.ProvingKey
	vk backend.VerifyingKey
	cs constraint.ConstraintSystem
}

func (*proverServer) mustEmbedUnimplementedUnionProverAPIServer() {}

func (p *proverServer) Prove(c context.Context, request *ProveRequest) (*ProveResponse, error) {
	fmt.Println("Proving...")

	reverseBytes := func(numbers []byte) []byte {
		newNumbers := make([]byte, 0, len(numbers))
		for i := len(numbers) - 1; i >= 0; i-- {
			newNumbers = append(newNumbers, numbers[i])
		}
		return newNumbers
	}

	marshalValidators := func(validators []*types.SimpleValidator) ([lightclient.MaxVal][4]frontend.Variable, []byte, error) {
		validatorsProto := [lightclient.MaxVal][4]frontend.Variable{}
		for i := 0; i < lightclient.MaxVal; i++ {
			validatorsProto[i][0] = 0
			validatorsProto[i][1] = 0
			validatorsProto[i][2] = 0
			validatorsProto[i][3] = 0
		}
		merkleTree := make([][]byte, len(validators))
		for i, val := range validators {
			protoEncoding, err := val.Marshal()
			if err != nil {
				return validatorsProto, nil, err
			}

			merkleTree[i] = protoEncoding

			TmPK, err := ce.PubKeyFromProto(*val.PubKey)
			if err != nil {
				return validatorsProto, nil, err
			}

			compressedPK := TmPK.Bytes()

			var PK curve.G1Affine
			_, err = PK.SetBytes(compressedPK)
			if err != nil {
				return validatorsProto, nil, err
			}

			PKX := PK.X.Bytes()
			PKY := PK.Y.Bytes()
			// Need to reverse to simplify circuit computation
			power := reverseBytes(protoEncoding[lightclient.ValProtoPower:])
			mask := compressedPK[0] >> 6
			validatorsProto[i][0] = PKX[:]
			validatorsProto[i][1] = PKY[:]
			validatorsProto[i][2] = power
			validatorsProto[i][3] = mask
		}
		return validatorsProto, merkle.HashFromByteSlices(merkleTree), nil
	}

	aggregateSignatures := func(signatures [][]byte) (curve.G2Affine, error) {
		var aggregatedSignature curve.G2Affine
		var decompressedSignature curve.G2Affine
		for _, signature := range signatures {
			_, err := decompressedSignature.SetBytes(signature)
			if err != nil {
				return curve.G2Affine{}, err
			}
			aggregatedSignature.Add(&aggregatedSignature, &decompressedSignature)
		}
		return aggregatedSignature, nil
	}

	trustedValidatorsProto, trustedValidatorsRoot, err := marshalValidators(request.TrustedCommit.Validators)
	if err != nil {
		return nil, err
	}
	trustedAggregatedSignature, err := aggregateSignatures(request.TrustedCommit.Signatures)
	if err != nil {
		return nil, err
	}

	untrustedValidatorsProto, untrustedValidatorsRoot, err := marshalValidators(request.UntrustedCommit.Validators)
	if err != nil {
		return nil, err
	}
	untrustedAggregatedSignature, err := aggregateSignatures(request.UntrustedCommit.Signatures)
	if err != nil {
		return nil, err
	}

	trustedInput := lcgadget.TendermintNonAdjacentLightClientInput{
		Sig:             gadget.NewG2Affine(trustedAggregatedSignature),
		ProtoValidators: trustedValidatorsProto,
		NbOfVal:         len(request.TrustedCommit.Validators),
		NbOfSignature:   len(request.TrustedCommit.Signatures),
		Bitmap:          new(big.Int).SetBytes(request.TrustedCommit.Bitmap),
	}

	untrustedInput := lcgadget.TendermintNonAdjacentLightClientInput{
		Sig:             gadget.NewG2Affine(untrustedAggregatedSignature),
		ProtoValidators: untrustedValidatorsProto,
		NbOfVal:         len(request.UntrustedCommit.Validators),
		NbOfSignature:   len(request.UntrustedCommit.Signatures),
		Bitmap:          new(big.Int).SetBytes(request.UntrustedCommit.Bitmap),
	}

	signedBytes, err := protoio.MarshalDelimited(request.Vote)
	if err != nil {
		return nil, err
	}

	hm := hashToField2(signedBytes)

	witness := lcgadget.Circuit{
		TrustedInput:   trustedInput,
		UntrustedInput: untrustedInput,
		ExpectedTrustedValRoot: [2]frontend.Variable{
			trustedValidatorsRoot[0:16],
			trustedValidatorsRoot[16:32],
		},
		ExpectedUntrustedValRoot: [2]frontend.Variable{
			untrustedValidatorsRoot[0:16],
			untrustedValidatorsRoot[16:32],
		},
		Message: [2]frontend.Variable{hm.A0.BigInt(new(big.Int)), hm.A1.BigInt(new(big.Int))},
	}

	fmt.Println(witness)

	privateWitness, err := frontend.NewWitness(&witness, ecc.BN254.ScalarField())
	if err != nil {
		return nil, err
	}

	var commitment constraint.Commitment
	switch _pk := p.pk.(type) {
	case *backend_bn254.ProvingKey:
		switch _vk := p.vk.(type) {
		case *backend_bn254.VerifyingKey:
			_pk.CommitmentKey = _vk.CommitmentKey
			commitment = _vk.CommitmentInfo
			break
		}
		break
	default:
		return nil, fmt.Errorf("Invalid proving key type, must be BN254")
	}

	fmt.Println("Proving...")
	proof, err := backend.Prove(p.cs, p.pk, privateWitness)
	if err != nil {
		return nil, err
	}

	var commitmentHash []byte
	var proofCommitment []byte
	switch _proof := proof.(type) {
	case *backend_bn254.Proof:
		if commitment.Is() {
			res, err := fr.Hash(commitment.SerializeCommitment(_proof.Commitment.Marshal(), []*big.Int{}, (fr.Bits-1)/8+1), []byte(constraint.CommitmentDst), 1)
			if err != nil {
				return nil, err
			}
			proofCommitment = _proof.Commitment.Marshal()
			commitmentHash = res[0].Marshal()
		}
		break
	default:
		return nil, fmt.Errorf("Invalid proof type, must be BN254")
	}

	var buffer bytes.Buffer
	mem := bufio.NewWriter(&buffer)
	_, err = proof.WriteRawTo(mem)
	if err != nil {
		return nil, err
	}
	mem.Flush()

	publicWitness, err := privateWitness.Public()
	if err != nil {
		return nil, err
	}

	fmt.Println(publicWitness)

	publicInputs, err := publicWitness.MarshalBinary()
	if err != nil {
		return nil, err
	}

	proofBz := append(append(publicInputs, commitmentHash...), proofCommitment...)

	return &ProveResponse{
		Proof: &ZeroKnowledgeProof{
			Content: proofBz,
		},
	}, nil
}

func NewProverServer() (*proverServer, error) {
	// TODO: load from file
	r1csInstance := backend.NewCS(ecc.BN254)
	pk := backend.NewProvingKey(ecc.BN254)
	vk := backend.NewVerifyingKey(ecc.BN254)

	var circuit lcgadget.Circuit

	fmt.Println("Compiling circuit...")
	r1csInstance, err := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &circuit, frontend.WithCompressThreshold(300))
	if err != nil {
		return nil, err
	}

	fmt.Println("Setup PK/VK")
	pk, vk, err = backend.Setup(r1csInstance)
	if err != nil {
		return nil, err
	}

	return &proverServer{pk: pk, vk: vk, cs: r1csInstance}, nil
}

var G2Cofactor big.Int

var Hash = sha3.NewLegacyKeccak256

func init() {
	value, err := new(big.Int).SetString("30644e72e131a029b85045b68181585e06ceecda572a2489345f2299c0f9fa8d", 16)
	if !err {
		panic("Cannot build cofactor")
	}
	G2Cofactor.Set(value)
}

func g1NotZero(x *fp.Element) uint64 {

	return x[0] | x[1] | x[2] | x[3]

}

// mapToCurve2 implements the Shallue and van de Woestijne method, applicable to any elliptic curve in Weierstrass form
// No cofactor clearing or isogeny
// https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#straightline-svdw
func mapToCurve2(u *curve.E2) curve.G2Affine {

	var tv1, tv2, tv3, tv4 curve.E2
	var x1, x2, x3, gx1, gx2, gx, x, y curve.E2
	var one curve.E2
	var gx1NotSquare, gx1SquareOrGx2Not int

	//constants
	//c1 = g(Z)
	//c2 = -Z / 2
	//c3 = sqrt(-g(Z) * (3 * Z² + 4 * A))     # sgn0(c3) MUST equal 0
	//c4 = -4 * g(Z) / (3 * Z² + 4 * A)

	Z := curve.E2{
		A0: fp.Element{15230403791020821917, 754611498739239741, 7381016538464732716, 1011752739694698287},
		A1: fp.Element{0},
	}
	c1 := curve.E2{
		A0: fp.Element{15219334786797146878, 8431472696017589261, 15336528771359260718, 196732871012706162},
		A1: fp.Element{4100506350182530919, 7345568344173317438, 15513160039642431658, 90557763186888013},
	}
	c2 := curve.E2{
		A0: fp.Element{12997850613838968789, 14304628359724097447, 2950087706404981016, 1237622763554136189},
		A1: fp.Element{0},
	}
	c3 := curve.E2{
		A0: fp.Element{12298500088583694207, 17447120171744064890, 14097510924717921191, 2278398337453771183},
		A1: fp.Element{4693446565795584099, 18320164443970680666, 6792758484113206563, 2989688171181581768},
	}
	c4 := curve.E2{
		A0: fp.Element{7191623630069643826, 8333948550768170742, 13001081703983517696, 2062355016518372226},
		A1: fp.Element{11163104453509316115, 7271947710149976975, 4894807947557820282, 3366254582553786647},
	}

	var bCurveCoeff fp.Element

	// twist
	var twist curve.E2

	// bTwistCurveCoeff b coeff of the twist (defined over 𝔽p²) curve
	var bTwistCurveCoeff curve.E2

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
	//12. gx1 = gx1 + A     All curves in gnark-crypto have A=0 (j-invariant=0). It is crucial to include this step if the curve has nonzero A coefficient.
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

	return curve.G2Affine{X: x, Y: y}
}

// g2Sgn0 is an algebraic substitute for the notion of sign in ordered fields
// Namely, every non-zero quadratic residue in a finite field of characteristic =/= 2 has exactly two square roots, one of each sign
// https://www.ietf.org/archive/id/draft-irtf-cfrg-hash-to-curve-16.html#name-the-sgn0-function
// The sign of an element is not obviously related to that of its Montgomery form
func g2Sgn0(z *curve.E2) uint64 {

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
func nativeNaiveScalarMul(p curve.G2Affine, s *big.Int) curve.G2Affine {
	var result curve.G2Affine
	result.X.SetZero()
	result.Y.SetZero()
	bits := s.BitLen()
	for i := bits - 1; i >= 0; i-- {
		result.Add(&result, &result)
		if s.Bit(i) == 1 {
			result.Add(&result, &p)
		}
	}
	return result
}

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

func hashToField2(msg []byte) curve.E2 {
	e0 := hashToField(append([]byte{0}, msg...))
	e1 := hashToField(append([]byte{1}, msg...))
	return curve.E2{A0: e0, A1: e1}
}
