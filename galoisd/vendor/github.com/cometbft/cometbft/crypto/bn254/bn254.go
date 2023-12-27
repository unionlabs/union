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
	"github.com/consensys/gnark-crypto/ecc/bn254/fr/mimc"
	bls254 "github.com/consensys/gnark-crypto/ecc/bn254/signature/bls"

	"github.com/cometbft/cometbft/crypto"
	cjson "github.com/cometbft/cometbft/libs/json"
	"github.com/holiman/uint256"
)

const (
	PubKeySize               = sizePublicKey
	PrivKeySize              = sizePrivateKey
	sizeFr                   = fr.Bytes
	sizeFp                   = fp.Bytes
	sizePublicKey            = sizeFp
	sizePrivateKey           = sizeFr + sizePublicKey
	XHashToScalarFieldPrefix = 0
	YHashToScalarFieldPrefix = 1
	PrivKeyName              = "tendermint/PrivKeyBn254"
	PubKeyName               = "tendermint/PubKeyBn254"
	KeyType                  = "bn254"
)

var G1Gen bn254.G1Affine
var G2Gen bn254.G2Affine
var G2Cofactor big.Int

var Hash = sha3.NewLegacyKeccak256

func init() {
	cjson.RegisterType(PubKey{}, PubKeyName)
	cjson.RegisterType(PrivKey{}, PrivKeyName)

	_, _, G1Gen, G2Gen = bn254.Generators()

	// BN254 cofactor
	value, err := new(big.Int).SetString("30644e72e131a029b85045b68181585e06ceecda572a2489345f2299c0f9fa8d", 16)
	if !err {
		panic("Cannot build cofactor")
	}

	G2Cofactor.Set(value)
}

var _ crypto.PrivKey = PrivKey{}

type PrivKey []byte

func (PrivKey) TypeTag() string { return PrivKeyName }

func (privKey PrivKey) Bytes() []byte {
	return []byte(privKey)
}

func (privKey PrivKey) Sign(msg []byte) ([]byte, error) {
	s := new(big.Int)
	s = s.SetBytes(privKey)
	hm := HashToG2(msg)
	var p bn254.G2Affine
	p.ScalarMultiplication(&hm, s)
	compressedSig := p.Bytes()
	return compressedSig[:], nil
}

func (privKey PrivKey) PubKey() crypto.PubKey {
	s := new(big.Int)
	s.SetBytes(privKey)
	var pk bn254.G1Affine
	pk.ScalarMultiplication(&G1Gen, s)
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

func(pubKey PubKey) EnsureValid() error {
	var public bn254.G1Affine
	_, err := public.SetBytes(pubKey)
	if err != nil {
		return err
	}
	if public.IsInfinity() {
		return fmt.Errorf("point at infinity")
	}
	return nil
}

func (PubKey) TypeTag() string { return PubKeyName }

func (pubKey PubKey) Address() crypto.Address {
	return crypto.AddressHash(pubKey[:])
}

func (pubKey PubKey) Bytes() []byte {
	return pubKey
}

func (pubKey PubKey) VerifySignature(msg []byte, sig []byte) bool {
	hashedMessage := HashToG2(msg)
	var public bn254.G1Affine
	_, err := public.SetBytes(pubKey)
	if err != nil {
		return false
	}
	if public.IsInfinity() {
		return false
	}

	var signature bn254.G2Affine
	_, err = signature.SetBytes(sig)
	if err != nil {
		return false
	}
	if signature.IsInfinity() {
		return false
	}

	var G1BaseNeg bn254.G1Affine
	G1BaseNeg.Neg(&G1Gen)

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
		panic(err)
	}
	return PrivKey(secret.Bytes())
}

// Naive scalar multiplication used for cofactor clearing, basic double-and-add
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

// TODO: link union whitepaper 4.1.1, equation (1), H_p
func HashToField(msg []byte) fr.Element {
	hmac := hmac.New(Hash, []byte("CometBLS"))
	hmac.Write(msg)
	modMinusOne := new(big.Int).Sub(fr.Modulus(), big.NewInt(1))
	num := new(big.Int).SetBytes(hmac.Sum(nil))
	num.Mod(num, modMinusOne)
	num.Add(num, big.NewInt(1))
	val, overflow := uint256.FromBig(num)
	if overflow {
		panic("impossible; qed;")
	}
	valBytes := val.Bytes32()
	var element fr.Element
	err := element.SetBytesCanonical(valBytes[:])
	if err != nil {
		panic("impossible; qed;")
	}
	return element
}

// TODO: link union whitepaper 4.1.1, H_{p^2}
func HashToField2(msg []byte) (fr.Element, fr.Element) {
	x := HashToField(append([]byte{XHashToScalarFieldPrefix}, msg...))
	y := HashToField(append([]byte{YHashToScalarFieldPrefix}, msg...))
	return x, y
}

// TODO: link union whitepaper 4.1.1, M
func HashToG2(msg []byte) bn254.G2Affine {
	x, y := HashToField2(msg)
	point := nativeNaiveScalarMul(bn254.MapToCurve2(&bn254.E2{
		A0: *new(fp.Element).SetBigInt(x.BigInt(new(big.Int))),
		A1: *new(fp.Element).SetBigInt(y.BigInt(new(big.Int))),
	}), &G2Cofactor)
	// Any of the following case are impossible and should break consensus
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

type MerkleLeaf struct {
	VotingPower int64
	ShiftedX    fr.Element
	ShiftedY    fr.Element
	MsbX        uint8
	MsbY        uint8
}

func NewMerkleLeaf(pubKey bn254.G1Affine, votingPower int64) (MerkleLeaf, error) {
	x := pubKey.X.BigInt(new(big.Int))
	y := pubKey.Y.BigInt(new(big.Int))
	msbX := x.Bit(254)
	msbY := y.Bit(254)
	var frX, frY fr.Element
	x.SetBit(x, 254, 0)
	var padded [32]byte
	x.FillBytes(padded[:])
	err := frX.SetBytesCanonical(padded[:])
	if err != nil {
		return MerkleLeaf{}, err
	}
	y.SetBit(y, 254, 0)
	y.FillBytes(padded[:])
	err = frY.SetBytesCanonical(padded[:])
	if err != nil {
		return MerkleLeaf{}, err
	}
	return MerkleLeaf{
		VotingPower: votingPower,
		ShiftedX:    frX,
		ShiftedY:    frY,
		MsbX:        uint8(msbX),
		MsbY:        uint8(msbY),
	}, nil
}

// mimc(X, Xmsb, Y, Ymsb, power)
func (l MerkleLeaf) Hash() []byte {
	frXBytes := l.ShiftedX.Bytes()
	frYBytes := l.ShiftedY.Bytes()
	mimc := mimc.NewMiMC()
	mimc.Write(frXBytes[:])
	mimc.Write(frYBytes[:])
	var padded [32]byte
	big.NewInt(int64(l.MsbX)).FillBytes(padded[:])
	mimc.Write(padded[:])
	big.NewInt(int64(l.MsbY)).FillBytes(padded[:])
	mimc.Write(padded[:])
	var powerBytes big.Int
	powerBytes.SetUint64(uint64(l.VotingPower))
	powerBytes.FillBytes(padded[:])
	mimc.Write(padded[:])
	return mimc.Sum(nil)
}
