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
	PrivKeyName              = "cometbft/PrivKeyBn254"
	PubKeyName               = "cometbft/PubKeyBn254"
	KeyType                  = "bn254"
	CometblsSigDST           = "COMETBLS_SIG_BN254G2_XMDMIMC256"
	CometblsHMACKey          = "CometBLS"
)

var (
	G1Gen    bn254.G1Affine
	G1GenNeg bn254.G1Affine
	G2Gen    bn254.G2Affine

	Hash = sha3.NewLegacyKeccak256
)

func init() {
	cjson.RegisterType(PubKey{}, PubKeyName)
	cjson.RegisterType(PrivKey{}, PrivKeyName)

	_, _, G1Gen, G2Gen = bn254.Generators()

	G1GenNeg.Neg(&G1Gen)
}

var _ crypto.PrivKey = PrivKey{}

type PrivKey []byte

func (PrivKey) TypeTag() string { return PrivKeyName }

func (privKey PrivKey) Bytes() []byte {
	return []byte(privKey)
}

// Union whitepaper: (5)
//
func (privKey PrivKey) Sign(msg []byte) ([]byte, error) {
	var s big.Int
	s.SetBytes(privKey)
	hm := HashToG2(msg)
	var p bn254.G2Affine
	p.ScalarMultiplication(&hm, &s)
	compressedSig := p.Bytes()
	return compressedSig[:], nil
}

// Union whitepaper: (4)
//
func (privKey PrivKey) PubKey() crypto.PubKey {
	var s big.Int
	s.SetBytes(privKey)
	var pk bn254.G1Affine
	pk.ScalarMultiplication(&G1Gen, &s)
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

func (pubKey PubKey) EnsureValid() error {
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

// Union whitepaper: (6)
//
func (pubKey PubKey) VerifySignature(msg []byte, sig []byte) bool {
	hm := HashToG2(msg)
	var pk bn254.G1Affine
	_, err := pk.SetBytes(pubKey)
	if err != nil {
		return false
	}
	if pk.IsInfinity() {
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

	valid, err := bn254.PairingCheck(
		[]bn254.G1Affine{
			G1GenNeg,
			pk,
		},
		[]bn254.G2Affine{
			signature,
			hm,
		})
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

func GenPrivKeyFromSeed(seed []byte) PrivKey {
	reader := bytes.NewReader(seed)

	secret, err := bls254.GenerateKey(reader)
	if err != nil {
		panic(err)
	}
	return PrivKey(secret.Bytes())
}

func GenPrivKey() PrivKey {
	secret, err := bls254.GenerateKey(rand.Reader)
	if err != nil {
		panic(err)
	}
	return PrivKey(secret.Bytes())
}

// Union whitepaper: (1) H_{hmac_r}
//
func HashToField(msg []byte) fr.Element {
	hmac := hmac.New(Hash, []byte(CometblsHMACKey))
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

// Union whitepaper: (3) H
//
func HashToG2(msg []byte) bn254.G2Affine {
	var img fr.Element
	err := img.SetBytesCanonical(msg)
	if err != nil {
		img = HashToField(msg)
	}
	var imgBytes [32]byte
	fr.LittleEndian.PutElement(&imgBytes, img)
	var dst fr.Element
	dst.SetBytes([]byte(CometblsSigDST))
	var dstBytes [32]byte
	fr.LittleEndian.PutElement(&dstBytes, dst)
	point, err := HashToG2MiMC(imgBytes[:], dstBytes[:])
	if err != nil {
		panic("impossible; qed;")
	}
	if point.IsInfinity() || !point.IsOnCurve() || !point.IsInSubGroup() {
		panic("impossible; qed;")
	}
	return point
}
