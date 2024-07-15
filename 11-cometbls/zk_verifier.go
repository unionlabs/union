package cometbls

import (
	"bytes"
	"crypto/hmac"
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"fmt"
	"math/big"
	"time"

	curve "github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
	backend "github.com/consensys/gnark/backend/groth16"
	backend_bn254 "github.com/consensys/gnark/backend/groth16/bn254"
	"github.com/holiman/uint256"
	"golang.org/x/crypto/sha3"
)

const (
	FQ_SIZE         = 32
	G1_SIZE         = 2 * FQ_SIZE
	G2_SIZE         = 2 * G1_SIZE
	CometblsHMACKey = "CometBLS"
)

var (
	Hash = sha3.NewLegacyKeccak256
)

type Proof struct {
	A curve.G1Affine
	B curve.G2Affine
	C curve.G1Affine
}

type ZKP struct {
	Proof              Proof
	ProofCommitment    curve.G1Affine
	ProofCommitmentPoK curve.G1Affine
}

type ProverLightHeader struct {
	ChainId            string
	Height             int64
	Time               time.Time
	ValidatorsHash     []byte
	NextValidatorsHash []byte
	AppHash            []byte
}

var verifyingKey backend_bn254.VerifyingKey

func init() {
	vkHex := "8967072901cc7ab63357f1ddc4196c7c1feda50540d8026d7f6f0167c118a899d923def15f75234f2a6d53b566a2528441e98050b38803673e9179b834fc39a499355fd270b7601d5d88408b7e9e53d260512e2180cd260017dc941f2fc96d65153f0344c6bf2d8a891b979bc61d39a98fb11155fcd57418f30ea018ea842874a0e76be91a3148e2f8ef644222b3ce5b939a73bd2e0a40814f7f92a79c483acf2216bbe0c289e07936b4d9653b91521a24c570c808fa46dfd12ec4429e71b61999fcfb245459d63a4923b8f8c488d1e6af7ca358867b88eb0cdefe896c221f09e95e4c18d1e0475de4549b2547611d8301e1afff1047a6f5a288c9314af0b9fc05d403c8c91820a385a72c18d6a4962cef41a3ab93daa7ed289b1e95db4d04eb00000003e71843e52743864f4bb67ce94a2ce8fe82c8f61042c4c1ced8531d94305392818b0dbe71f4d60e02e9160ec2b015cae3a09cbe4f437226e2c02e1a5e5d124bcac29e93d5f47c0c7671350398ed8c40f5bc5c2f5b00363c7e2eb18a91a1c490c70000000100000000a57df6f8132cb0037f7dfdf1a29b04c1ff92ba082eda513996ba2bfa9fbd198713f0d8d8879885ca567ef99298c30c397e6fba584658f4127713a814c06de55aefbfe141a7555cf7e3e86b092660b81cfb68a025ad817e45cec0b0f2e2ca636802a104df1c015f2307fa2859627098cdf9fdb521d61d323943343a12304e5baf"
	vk, err := hex.DecodeString(vkHex)
	if err != nil {
		panic(fmt.Sprintf("could not decode the hex verifying key: '%s'", vkHex))
	}

	_, err = backend.VerifyingKey(&verifyingKey).ReadFrom(bytes.NewReader(vk))
	if err != nil {
		panic(fmt.Sprintf("could not read the verifying key: '%s'", vkHex))
	}
}

func ParseZKP(data []byte) (*ZKP, error) {

	zkp := ZKP{}

	cursor := 0

	zkp.Proof.A.SetBytes(data[0:G1_SIZE])
	cursor += G1_SIZE

	zkp.Proof.B.SetBytes(data[cursor : cursor+G2_SIZE])
	cursor += G2_SIZE

	zkp.Proof.C.SetBytes(data[cursor : cursor+G1_SIZE])
	cursor += G1_SIZE

	zkp.ProofCommitment.SetBytes(data[cursor : cursor+G1_SIZE])
	cursor += G1_SIZE

	zkp.ProofCommitmentPoK.SetBytes(data[cursor : cursor+G1_SIZE])

	return &zkp, nil
}

func (zkp ZKP) Verify(trustedValidatorsHash []byte, header ProverLightHeader) error {
	if len(header.ChainId) > 31 {
		return errors.New("chain id length cannot be larger than 31")
	}

	commHash := commitmentsHash(zkp.ProofCommitment)
	inpHash := inputsHash(header, trustedValidatorsHash)

	var initialPoint curve.G1Affine
	initialPoint.Add(&verifyingKey.G1.K[0], &zkp.ProofCommitment)

	var commMul curve.G1Affine
	var commBigInt big.Int
	commHash.BigInt(&commBigInt)
	commMul.ScalarMultiplication(&verifyingKey.G1.K[2], &commBigInt)

	var inpMul curve.G1Affine
	var inpBigInt big.Int
	inpHash.BigInt(&inpBigInt)
	inpMul.ScalarMultiplication(&verifyingKey.G1.K[1], &inpBigInt)

	publicInputsMsm := initialPoint
	publicInputsMsm.Add(&publicInputsMsm, &inpMul)
	publicInputsMsm.Add(&publicInputsMsm, &commMul)

	hasher := sha256.New()
	a := zkp.Proof.A.Bytes()
	hasher.Write(a[:])
	c := zkp.Proof.C.Bytes()
	hasher.Write(c[:])
	msm := publicInputsMsm.Bytes()
	hasher.Write(msm[:])

	hasher.Reset()
	pc := zkp.ProofCommitment.Bytes()
	hasher.Write(pc[:])
	pcPok := zkp.ProofCommitmentPoK.Bytes()
	hasher.Write(pcPok[:])

	alpha := verifyingKey.G1.Alpha
	gamma := verifyingKey.G2.Gamma
	delta := verifyingKey.G2.Delta
	beta := verifyingKey.G2.Beta

	// NOTE(aeryz): We didn't use fiat-shamir here since we don't batch two pairings. I'm not sure
	// if we still need it though.
	result, err := curve.PairingCheck([]curve.G1Affine{
		zkp.Proof.A,
		publicInputsMsm,
		zkp.Proof.C,
		alpha,
	}, []curve.G2Affine{
		zkp.Proof.B,
		*gamma.Neg(&gamma),
		*delta.Neg(&delta),
		*beta.Neg(&beta),
	})

	if err != nil {
		return err
	}

	if !result {
		return errors.New("proof verification failed")
	}

	return verifyingKey.CommitmentKey.Verify(zkp.ProofCommitment, zkp.ProofCommitmentPoK)
}

func hashToField(msg []byte) fr.Element {
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

func commitmentsHash(proofCommitment curve.G1Affine) fr.Element {
	var buffer [64]byte

	x := proofCommitment.X.Bytes()
	copy(buffer[0:32], x[:])

	y := proofCommitment.Y.Bytes()
	copy(buffer[32:64], y[:])

	return hashToField(buffer[:])
}

func inputsHash(header ProverLightHeader, trustedValidatorsHash []byte) fr.Element {
	buff := []byte{}
	var padded [32]byte
	writeI64 := func(x int64) {
		big.NewInt(x).FillBytes(padded[:])
		buff = append(buff, padded[:]...)
	}
	writeMiMCHash := func(b []byte) {
		big.NewInt(0).SetBytes(b).FillBytes(padded[:])
		buff = append(buff, padded[:]...)
	}
	writeHash := func(b []byte) {
		buff = append(buff, b...)
	}
	writeMiMCHash([]byte(header.ChainId))
	writeI64(header.Height)
	writeI64(header.Time.Unix())
	writeI64(int64(header.Time.Nanosecond()))
	writeMiMCHash(header.ValidatorsHash)
	writeMiMCHash(header.NextValidatorsHash)
	writeHash(header.AppHash)
	writeMiMCHash(trustedValidatorsHash)
	hash := sha256.Sum256(buff)

	var e fr.Element
	e.SetBytes(hash[1:])
	return e
}
