package main

import (
	"crypto/rand"
	"fmt"
	"math/big"
	"testing"
	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/frontend"
	backend "github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	backend_bn254 "github.com/consensys/gnark/backend/groth16/bn254"
	// curve "github.com/consensys/gnark-crypto/ecc/bn254"
)

type testCircuit struct {
	X frontend.Variable `gnark:",public"`
}

func (c *testCircuit) Define(api frontend.API) error {
	api.AssertIsEqual(c.X, c.X)
	return nil
}

func TestTest(t *testing.T) {
	genPriv := func() big.Int {
		privateKey, err := rand.Int(rand.Reader, big.NewInt(0).Exp(big.NewInt(2), big.NewInt(130), nil))
		if err != nil {
			panic(err)
		}
		return *privateKey
	}

	_ = genPriv

	r1csInstance := backend.NewCS(ecc.BN254)
	pk := backend.NewProvingKey(ecc.BN254)
	vk := backend.NewVerifyingKey(ecc.BN254)

	var circuit testCircuit

	fmt.Println("Compiling circuit...")
	r1csInstance, err := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &circuit, frontend.WithCompressThreshold(300))
	if err != nil {
		panic(err)
	}

	fmt.Println("Setup PK/VK")
	pk, vk, err = backend.Setup(r1csInstance)
	if err != nil {
		panic(err)
	}

	switch _pk := pk.(type) {
	case *backend_bn254.ProvingKey:
		switch _vk := vk.(type) {
		case *backend_bn254.VerifyingKey:
			_ = _pk
			_ = _vk
			_pk.G1.Alpha.ScalarMultiplicationBase(big.NewInt(2))
			_vk.G1.Alpha.ScalarMultiplicationBase(big.NewInt(2))
			break;
		}
		break;
	}

	privateWitness, err := frontend.NewWitness(&testCircuit{X: 1}, ecc.BN254.ScalarField())
	if err != nil {
		panic(err)
	}

	fmt.Println("Proving...")
	proof, err := backend.Prove(r1csInstance, pk, privateWitness)
	if err != nil {
		panic(err)
	}

	fmt.Printf("Extracting %d public:\n", vk.NbPublicWitness())
	publicWitness, err := privateWitness.Public()
	if err != nil {
		panic(err)
	}

	fmt.Println("Verifying...")
	err = backend.Verify(proof, vk, publicWitness)
	if err != nil {
		panic(err)
	}
}
