package main

import (
	"bufio"
	"bytes"
	provercmd "cometbls-prover/cmd"
	"cometbls-prover/pkg/lightclient"
	lcgadget "cometbls-prover/pkg/lightclient/nonadjacent"
	"crypto/hmac"
	"encoding/base64"
	"encoding/hex"
	"fmt"
	cometbn254 "github.com/cometbft/cometbft/crypto/bn254"
	ce "github.com/cometbft/cometbft/crypto/encoding"
	"github.com/cometbft/cometbft/crypto/merkle"
	"github.com/cometbft/cometbft/libs/protoio"
	"github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/consensys/gnark-crypto/ecc"
	curve "github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fp"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
	"github.com/consensys/gnark-crypto/ecc/bn254/signature/bls"
	backend "github.com/consensys/gnark/backend/groth16"
	backend_bn254 "github.com/consensys/gnark/backend/groth16/bn254"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	gadget "github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/holiman/uint256"
	"github.com/spf13/cobra"
	"golang.org/x/crypto/sha3"
	"math/big"
)

const NbOfVal = 4

var BitmapSig = [NbOfVal]bool{}

var G2Cofactor big.Int

var Hash = sha3.NewLegacyKeccak256

func init() {
	for i := 0; i < NbOfVal; i++ {
		BitmapSig[i] = true
	}
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

func HashToField2(msg []byte) curve.E2 {
	e0 := hashToField(append([]byte{0}, msg...))
	e1 := hashToField(append([]byte{1}, msg...))
	return curve.E2{A0: e0, A1: e1}
}

func HashToG2(msg []byte) curve.G2Affine {
	e := HashToField2(msg)
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

func main() {
	var rootCmd = &cobra.Command{Use: "cometbls-prover"}
	rootCmd.AddCommand(provercmd.ServeCmd)
	rootCmd.AddCommand(provercmd.ProveCmd)
	rootCmd.Execute()
}

func prove() {
	blockHash, err := hex.DecodeString("CF8FB45282F3687C4BF305090C950BC28C7A7A5E35C2A9A1F5930D56A77F3C75")
	if err != nil {
		panic(err)
	}

	partSetHeaderHash, err := hex.DecodeString("39C604A64DDBDA8F2E0F31F0DF30315CE4B8E65DB91F74F29A5ED6926C70A03F")
	if err != nil {
		panic(err)
	}

	canonicalVote := types.CanonicalVote{
		Type:   types.PrecommitType,
		Height: 1,
		Round:  0,
		BlockID: &types.CanonicalBlockID{
			Hash: blockHash,
			PartSetHeader: types.CanonicalPartSetHeader{
				Total: 1,
				Hash:  partSetHeaderHash,
			},
		},
		ChainID: "union-devnet-1",
	}

	bz, err := protoio.MarshalDelimited(&canonicalVote)
	if err != nil {
		panic(err)
	}

	fmt.Printf("Signed vote: %X\n", bz)

	HMF := HashToField2(bz)

	fmt.Printf("Hashed messageX: %X\n", HMF.A0.Bytes())
	fmt.Printf("Hashed messageY: %X\n", HMF.A1.Bytes())

	HM := HashToG2(bz)

	fmt.Printf("Mapped message: %X\n", HM.Bytes())

	_, _, g1AffGen, g2AffGen := curve.Generators()
	_ = g2AffGen

	var g1AffGenNeg curve.G1Affine
	g1AffGenNeg.Neg(&g1AffGen)

	decodeB64 := func(s string) []byte {
		bz, err := base64.StdEncoding.DecodeString(s)
		if err != nil {
			panic(err)
		}
		return bz
	}

	encodePK := func(pk bls.PublicKey) string {
		return base64.StdEncoding.EncodeToString(pk.Bytes())
	}
	encodeSig := func(s bls.Signature) string {
		return base64.StdEncoding.EncodeToString(s.Bytes())
	}

	// !!!!!!!!!!!!! Order of registration matter for the root hash !!!!!!!!!!
	publicKeys := [lightclient.MaxVal]bls.PublicKey{}
	_, err = publicKeys[1].SetBytes(decodeB64("wiY2IMV1eUwte40Km2Lw4H1zYGQ0ZvemMPoru9rf/pQ="))
	if err != nil {
		panic(err)
	}
	_, err = publicKeys[0].SetBytes(decodeB64("xCIuHcyesunreiQ86q+R2+KgP/rVYaGJ+XQGP8VShNc="))
	if err != nil {
		panic(err)
	}
	_, err = publicKeys[2].SetBytes(decodeB64("q/8jFgPQVjyyLqCvJo0Qsk8v8M0M51Ojw0Eg1KCsebo="))
	if err != nil {
		panic(err)
	}
	_, err = publicKeys[3].SetBytes(decodeB64("0QPPjuq9oaGp6nRm/SKrwNJkQTQDT2DtdVQm/9yJ3g0="))
	if err != nil {
		panic(err)
	}

	var aggPK curve.G1Affine
	for i, pk := range publicKeys {
		if i < len(BitmapSig) && BitmapSig[i] {
			aggPK.Add(&aggPK, &pk.A)
		}
	}

	signatures := [lightclient.MaxVal]bls.Signature{}
	_, err = signatures[0].SetBytes(decodeB64("k6kYQdqpOikXAPAm0uZGUHv6E2J5eT0SfehBSLcsFRELX0eWzq0spupRtr1z1f9I9fvSDVXcUNAMSLIjd9Rrtw=="))
	if err != nil {
		panic(err)
	}
	_, err = signatures[1].SetBytes(decodeB64("5E2bEigmCNTTf21Y1mP2mSKflg5r/oM1F3uGQDyqmbwN6lUmJ5lxx2lUTcmUWsYce3860+TQE/NORlW3d79Uzw=="))
	if err != nil {
		panic(err)
	}
	_, err = signatures[2].SetBytes(decodeB64("x0w000Y91HCtN5+j4cSO66c9Wsdznr/SomryXMXCQRQZWuBYZWIMF7K4hL+U35Q28IHpkGaJXWnRXs/oStm0Pw=="))
	if err != nil {
		panic(err)
	}
	_, err = signatures[3].SetBytes(decodeB64("zNVt/Ivgvjwl9EEGOo7YH3AfOTIpJCwcf+5IpslQmmUsfvrFMrwhOybQNgNukSZAzrujxCbZimBxbNgw/a5OCw=="))
	if err != nil {
		panic(err)
	}

	var aggSig curve.G2Affine
	for i, signature := range signatures {
		if i < len(BitmapSig) && BitmapSig[i] {
			aggSig.Add(&aggSig, &signature.S)
		}
	}

	fmt.Println("AggPK: ", encodePK(bls.PublicKey{A: aggPK}))
	fmt.Println("AggSig: ", encodeSig(bls.Signature{S: aggSig}))

	// Quickly check the pairing
	result, err := curve.PairingCheck([]curve.G1Affine{g1AffGenNeg, aggPK}, []curve.G2Affine{aggSig, HM})
	if err != nil {
		panic(err)
	}
	if !result {
		panic("Failed to check pairing")
	}

	tokens, success := new(big.Int).SetString("1000000000000000000000", 10)
	if !success {
		panic("oops")
	}

	power := sdk.TokensToConsensusPower(sdk.NewIntFromBigInt(tokens), sdk.DefaultPowerReduction)

	protoValidators := make([][]byte, NbOfVal)
	for i, PK := range publicKeys {
		if i < NbOfVal {
			protoPK, err := ce.PubKeyToProto(cometbn254.PubKey(PK.Bytes()))
			val := types.SimpleValidator{
				PubKey:      &protoPK,
				VotingPower: power,
			}
			protoEncoding, err := val.Marshal()
			if err != nil {
				panic(err)
			}
			protoValidators[i] = protoEncoding
		}
	}

	valRoot := merkle.HashFromByteSlices(protoValidators)

	fmt.Printf("ValRoot: %X\n", valRoot)

	bitmap := new(big.Int)
	nbOfSig := 0
	for i := 0; i < NbOfVal; i++ {
		if BitmapSig[i] {
			bitmap.SetBit(bitmap, i, 1)
			nbOfSig += 1
		}
	}

	rev := func(numbers []byte) []byte {
		newNumbers := make([]byte, 0, len(numbers))
		for i := len(numbers) - 1; i >= 0; i-- {
			newNumbers = append(newNumbers, numbers[i])
		}
		return newNumbers
	}

	var assignment lightclient.TendermintLightClientInput
	assignment.Sig = gadget.NewG2Affine(aggSig)
	assignment.ProtoValidators = [lightclient.MaxVal][lightclient.ValProtoElems]frontend.Variable{}
	for i := 0; i < lightclient.MaxVal; i++ {
		for j := 0; j < lightclient.ValProtoElems; j++ {
			assignment.ProtoValidators[i][j] = 0
		}
		if i < NbOfVal {
			valProto := protoValidators[i]
			PKX := publicKeys[i].A.X.Bytes()
			PKY := publicKeys[i].A.Y.Bytes()
			compressedPK := publicKeys[i].Bytes()
			// Need to reverse to simplify circuit computation
			power := rev(valProto[lightclient.ValProtoPower:])
			mask := compressedPK[0] >> 6
			assignment.ProtoValidators[i][0] = PKX[:]
			assignment.ProtoValidators[i][1] = PKY[:]
			assignment.ProtoValidators[i][2] = power
			assignment.ProtoValidators[i][3] = mask
		}
	}
	assignment.NbOfVal = NbOfVal
	assignment.NbOfSignature = nbOfSig
	assignment.Bitmap = bitmap

	ExpectedValRoot := [2]frontend.Variable{
		valRoot[0:16],
		valRoot[16:32],
	}

	trustedInput := lcgadget.TendermintNonAdjacentLightClientInput{
		Sig:             assignment.Sig,
		ProtoValidators: assignment.ProtoValidators,
		NbOfVal:         assignment.NbOfVal,
		NbOfSignature:   assignment.NbOfSignature,
		Bitmap:          assignment.Bitmap,
	}
	untrustedInput := lcgadget.TendermintNonAdjacentLightClientInput{
		Sig:             assignment.Sig,
		ProtoValidators: assignment.ProtoValidators,
		NbOfVal:         assignment.NbOfVal,
		NbOfSignature:   assignment.NbOfSignature,
		Bitmap:          assignment.Bitmap,
	}

	privateWitness, err := frontend.NewWitness(&lcgadget.Circuit{
		TrustedInput:             trustedInput,
		UntrustedInput:           untrustedInput,
		ExpectedTrustedValRoot:   ExpectedValRoot,
		ExpectedUntrustedValRoot: ExpectedValRoot,
		Message:                  [2]frontend.Variable{HMF.A0.BigInt(new(big.Int)), HMF.A1.BigInt(new(big.Int))},
	}, ecc.BN254.ScalarField())
	if err != nil {
		panic(err)
	}

	r1csInstance := backend.NewCS(ecc.BN254)
	pk := backend.NewProvingKey(ecc.BN254)
	vk := backend.NewVerifyingKey(ecc.BN254)

	var circuit lcgadget.Circuit

	fmt.Println("Compiling circuit...")
	r1csInstance, err = frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &circuit, frontend.WithCompressThreshold(300))
	if err != nil {
		panic(err)
	}

	fmt.Println("Setup PK/VK")
	pk, vk, err = backend.Setup(r1csInstance)
	if err != nil {
		panic(err)
	}

	// readFrom := func(file string) *os.File {
	// 	f, err := os.OpenFile(file, os.O_RDONLY, os.ModePerm)
	// 	if err != nil {
	// 		panic(err)
	// 	}
	// 	return f
	// }

	// fmt.Println("Setup R1CS")
	// r1csBin := readFrom("r1cs.bin")
	// _, err = r1csInstance.ReadFrom(r1csBin)
	// if err != nil {
	// 	panic(err)
	// }
	// r1csBin.Close()

	// fmt.Println("Setup PK")
	// pkBin := readFrom("pk.bin")
	// _, err = pk.ReadFrom(pkBin)
	// if err != nil {
	// 	panic(err)
	// }
	// pkBin.Close()

	// fmt.Println("Setup VK")
	// vkBin := readFrom("vk.bin")
	// _, err = vk.ReadFrom(vkBin)
	// if err != nil {
	// 	panic(err)
	// }
	// vkBin.Close()

	var commitment constraint.Commitment
	switch _pk := pk.(type) {
	case *backend_bn254.ProvingKey:
		switch _vk := vk.(type) {
		case *backend_bn254.VerifyingKey:
			_pk.CommitmentKey = _vk.CommitmentKey
			commitment = _vk.CommitmentInfo
			break
		}
		break
	}

	// saveTo := func(file string, x io.WriterTo) {
	// 	fmt.Printf("Saving %s\n", file)
	// 	f, err := os.Create(file)
	// 	if err != nil {
	// 		panic(err)
	// 	}
	// 	defer f.Close()
	// 	w := bufio.NewWriter(f)
	// 	written, err := x.WriteTo(w)
	// 	if err != nil {
	// 		panic(err)
	// 	}
	// 	fmt.Printf("Saved %d bytes\n", written)
	// 	w.Flush()
	// }

	// saveTo("r1cs.bin", r1csInstance)
	// saveTo("pk.bin", pk)
	// saveTo("vk.bin", vk)

	fmt.Println("Proving...")
	proof, err := backend.Prove(r1csInstance, pk, privateWitness)
	if err != nil {
		panic(err)
	}

	var commitmentHash []byte
	var proofCommitment []byte
	switch _proof := proof.(type) {
	case *backend_bn254.Proof:
		if commitment.Is() {
			res, err := fr.Hash(commitment.SerializeCommitment(_proof.Commitment.Marshal(), []*big.Int{}, (fr.Bits-1)/8+1), []byte(constraint.CommitmentDst), 1)
			if err != nil {
				panic(err)
			}
			proofCommitment = _proof.Commitment.Marshal()
			commitmentHash = res[0].Marshal()
		}
		break
	}

	var buffer bytes.Buffer
	mem := bufio.NewWriter(&buffer)
	_, err = proof.WriteRawTo(mem)
	if err != nil {
		panic(err)
	}
	mem.Flush()

	fmt.Printf("Proof: 0x%x\n", buffer.Bytes())

	fmt.Printf("Extracting %d public:\n", vk.NbPublicWitness())
	publicWitness, err := privateWitness.Public()
	if err != nil {
		panic(err)
	}

	fmt.Println(publicWitness)

	publicInputs, err := publicWitness.MarshalBinary()
	if err != nil {
		panic(err)
	}

	fmt.Printf("CommitmentHash: 0x%x\n", commitmentHash)
	fmt.Printf("ProofCommitment: 0x%x\n", proofCommitment)
	fmt.Printf("PublicWitness: 0x%x\n", append(append(publicInputs, commitmentHash...), proofCommitment...))

	fmt.Println("Verifying...")
	err = backend.Verify(proof, vk, publicWitness)
	if err != nil {
		panic(err)
	}

	// err = vk.ExportSolidity(os.Stdout)
	// if err != nil {
	// 	fmt.Println(err)
	// 	return
	// }
}
