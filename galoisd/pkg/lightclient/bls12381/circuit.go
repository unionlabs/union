package bls12381

import (
	"fmt"

	"github.com/consensys/gnark/frontend"

	"github.com/consensys/gnark/std/math/emulated"
	"github.com/consensys/gnark/std/recursion/groth16"

	"github.com/consensys/gnark/std/algebra"
)

type Circuit[FR emulated.FieldParams, G1El algebra.G1ElementT, G2El algebra.G2ElementT, GtEl algebra.GtElementT] struct {
	Proof        groth16.Proof[G1El, G2El]
	VerifyingKey groth16.VerifyingKey[G1El, G2El, GtEl]
	InnerWitness groth16.Witness[FR]
}

func (c *Circuit[FR, G1El, G2El, GtEl]) Define(api frontend.API) error {
	verifier, err := groth16.NewVerifier[FR, G1El, G2El, GtEl](api)
	if err != nil {
		return fmt.Errorf("new verifier: %w", err)
	}
	return verifier.AssertProof(c.VerifyingKey, c.Proof, c.InnerWitness)
}
