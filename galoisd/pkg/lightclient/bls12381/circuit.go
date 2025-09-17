package bls12381

import (
	"fmt"

	// "github.com/consensys/gnark-crypto/ecc/bn254/fr"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/algebra/emulated/sw_bn254"

	"github.com/consensys/gnark/std/algebra/emulated/sw_emulated"
	"github.com/consensys/gnark/std/math/emulated"
	"github.com/consensys/gnark/std/recursion/groth16"
)

type Circuit struct {
	Proof           groth16.Proof[sw_bn254.G1Affine, sw_bn254.G2Affine]
	VerifyingKey    groth16.VerifyingKey[sw_bn254.G1Affine, sw_bn254.G2Affine, sw_bn254.GTEl]
	InnerWitness    groth16.Witness[sw_bn254.ScalarField]
	CommitmentHash  frontend.Variable `gnark:",public"`
	CommitmentX     frontend.Variable `gnark:",public"`
	CommitmentY     frontend.Variable `gnark:",public"`
	InnerInputsHash frontend.Variable `gnark:",public"`
	// VkHash          frontend.Variable `gnark:",public"`
	// OptimizedInnerWitness frontend.Variable `gnark:",public"`
}

func (c *Circuit) Define(api frontend.API) error {
	verifier, err := groth16.NewVerifier[sw_bn254.ScalarField, sw_bn254.G1Affine, sw_bn254.G2Affine, sw_bn254.GTEl](api)
	if err != nil {
		return fmt.Errorf("new verifier: %w", err)
	}

	// AssertEq(mimcHash(verifyikgkey.G1.X.Limbs..., verifyingKey.G1.Y.Limbs...), VkHash)

	xLimbs := Unpack(api, c.CommitmentX, 256, 64)
	yLimbs := Unpack(api, c.CommitmentY, 256, 64)

	var commitment sw_bn254.G1Affine
	commitment.X.Limbs = xLimbs
	commitment.Y.Limbs = yLimbs

	f, _ := sw_emulated.New[emulated.BN254Fp, emulated.BN254Fr](api, sw_emulated.GetCurveParams[emulated.BN254Fp]())

	f.AssertIsEqual(&c.Proof.Commitments[0].G1El, &commitment)

	scalarApi, _ := emulated.NewField[emulated.BN254Fr](api)

	innerInputsHash := scalarApi.FromBits(api.ToBinary(c.InnerInputsHash)...)
	scalarApi.AssertIsEqual(&c.InnerWitness.Public[0], innerInputsHash)

	return verifier.AssertProof(c.VerifyingKey, c.Proof, c.InnerWitness, groth16.WithCommitmentHash(c.CommitmentHash))
}

func Unpack(api frontend.API, packed frontend.Variable, sizeOfInput int, sizeOfElem int) []frontend.Variable {
	nbOfElems := sizeOfInput / sizeOfElem
	if sizeOfElem == 1 {
		return api.ToBinary(packed, nbOfElems)
	} else {
		unpacked := api.ToBinary(packed, sizeOfInput)
		elems := make([]frontend.Variable, nbOfElems)
		for i := 0; i < nbOfElems; i++ {
			elems[i] = api.FromBinary(unpacked[i*sizeOfElem : (i+1)*sizeOfElem]...)
		}
		return elems
	}
}
