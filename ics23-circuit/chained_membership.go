package main

import (
	"github.com/consensys/gnark/frontend"
)

type ChainedMembershipCircuit struct {
	AppHash      frontend.Variable
	Iavl         ExistenceProof
	SimpleMerkle ExistenceProof
}

func (circuit *ChainedMembershipCircuit) Define(api frontend.API) error {
	lsb := api.ToBinary(circuit.SimpleMerkle.HashedValue[1], 128)
	msb := api.ToBinary(circuit.SimpleMerkle.HashedValue[0], 128)
	lsb = append(lsb, msb...)
	root := api.FromBinary(lsb...)

	valid := circuit.Iavl.Verify(api, root)
	api.AssertIsEqual(valid, 1)

	valid = circuit.SimpleMerkle.Verify(api, circuit.AppHash)
	api.AssertIsEqual(valid, 1)

	return nil
}

type ChainedMembershipCircuitOpt struct {
	AppHash      frontend.Variable
	Iavl         ExistenceProofOpt
	SimpleMerkle ExistenceProofOpt
}

func (circuit *ChainedMembershipCircuitOpt) Define(api frontend.API) error {
	valid := circuit.Iavl.Verify(api, circuit.SimpleMerkle.HashedValue)
	api.AssertIsEqual(valid, 1)

	valid = circuit.SimpleMerkle.Verify(api, circuit.AppHash)
	api.AssertIsEqual(valid, 1)

	return nil
}
