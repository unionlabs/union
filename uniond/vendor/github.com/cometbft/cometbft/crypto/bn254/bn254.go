package bn254

import (
	"bytes"
	"crypto/rand"
	"crypto/subtle"
	"encoding/binary"
	"fmt"
	"math/big"

	"golang.org/x/crypto/sha3"

	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fp"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
	bls254 "github.com/consensys/gnark-crypto/ecc/bn254/signature/bls"

	"github.com/cometbft/cometbft/crypto"
	cmtjson "github.com/cometbft/cometbft/libs/json"
)

const (
	PrivKeyName = "tendermint/PrivKeyBn254"
	PubKeyName  = "tendermint/PubKeyBn254"
	KeyType     = "bn254"
	PubKeySize  = sizePublicKey
	PrivKeySize = sizePrivateKey
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
	hashed, _ := hashedMessage(msg)
	var p bn254.G2Affine
	p.ScalarMultiplication(&hashed, s)
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
	hashedMessage, _ := hashedMessage(msg)
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

var Hash = sha3.NewLegacyKeccak256

func init() {
	cmtjson.RegisterType(PubKey{}, PubKeyName)
	cmtjson.RegisterType(PrivKey{}, PrivKeyName)

	_, _, G1Base, G2Base = bn254.Generators()
}

/* Loop until we find a valid G2 point derived from:
   X0=uint256(keccak256(msg || i)))
   X1=uint256(keccak256(i || msg)))

   Y0,Y1=Decompress(X0, X1)

   Point is then recoverable from the tuple (msg, i, Y0, Y1)

TODO: performance
*/
func hashedMessage(msg []byte) (bn254.G2Affine, uint32) {
	var point bn254.G2Affine
	var i = uint32(0)
	b := make([]byte, 4)
	h := Hash()
	for {
		binary.BigEndian.PutUint32(b, i)
		h.Reset()
		h.Write(b)
		h.Write(msg)
		X0 := h.Sum(nil)
		h.Reset()
		h.Write(msg)
		h.Write(b)
		X1 := h.Sum(nil)
		_, err := point.SetBytes(append(X0, X1...))
		if err != nil || !point.IsOnCurve() {
			fmt.Printf("Invalid point at %d\n", i)
			i++
			continue
		}
		break
	}
	return point, i
}
