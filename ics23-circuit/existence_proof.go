package main

import (
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/hash/mimc"
)

type Inner struct {
	// 16 or 48
	Prefix    [MaxPrefixCount]frontend.Variable
	PrefixLen frontend.Variable
	// 0 or 32
	Suffix    [MaxSuffixCount]frontend.Variable
	SuffixLen frontend.Variable
}

type InnerOpt struct {
	Prefix    frontend.Variable
	IsLeft    frontend.Variable
	OtherHash frontend.Variable
}

type ExistenceProof struct {
	HashedKey   [2]frontend.Variable `gnark:",public"`
	HashedValue [2]frontend.Variable `gnark:",public"`
	Path        [MaxPathDepth]Inner  `gnark:",public"`
	PathLen     frontend.Variable    `gnark:",public"`
	LeafPrefix  frontend.Variable    `gnark:",public"`
}

type ExistenceProofOpt struct {
	HashedKey   frontend.Variable      `gnark:",public"`
	HashedValue frontend.Variable      `gnark:",public"`
	Path        [MaxPathDepth]InnerOpt `gnark:",public"`
	PathLen     frontend.Variable      `gnark:",public"`
	LeafPrefix  frontend.Variable      `gnark:",public"`
}

type ExistenceCircuit struct {
	Proof ExistenceProof
	Root  frontend.Variable `gnark:",public"`
}

type ExistenceCircuitOpt struct {
	Proof ExistenceProofOpt
	Root  frontend.Variable `gnark:",public"`
}

type NonExistCircuit struct {
	Left  ExistenceProof
	Right ExistenceProof
	Root  frontend.Variable
}

func (circuit *ExistenceCircuit) Define(api frontend.API) error {
	valid := circuit.Proof.Verify(api, circuit.Root)
	api.AssertIsEqual(valid, 1)

	return nil
}

func (circuit *ExistenceCircuitOpt) Define(api frontend.API) error {
	valid := circuit.Proof.Verify(api, circuit.Root)
	api.AssertIsEqual(valid, 1)

	return nil
}

func (proof *ExistenceProof) Verify(api frontend.API, root frontend.Variable) frontend.Variable {
	hFunc, _ := mimc.NewMiMC(api)
	hFunc.Write(proof.LeafPrefix, proof.HashedKey[0], proof.HashedKey[1], proof.HashedValue[0], proof.HashedValue[1])
	calcRoot := hFunc.Sum()

	for i := 0; i < MaxPathDepth; i++ {
		// TODO(aeryz): In the original ics23 implementation, the proof is being checked against the cosmos spec and
		// the prefix and suffix sizes are controlled for all inner ops. Since we have fixed prefix and suffix, not sure
		// if that's needed.

		hFunc.Reset()
		rootBin := api.ToBinary(calcRoot, 256)
		rootMSB := api.FromBinary(rootBin[128:256]...)
		rootLSB := api.FromBinary(rootBin[0:128]...)
		// prefix = 48, suffix = 0
		hFunc.Write(proof.Path[i].Prefix[0], proof.Path[i].Prefix[1], proof.Path[i].Prefix[2], rootMSB, rootLSB)
		withPrefix := hFunc.Sum()
		hFunc.Reset()
		// prefix = 16, suffix = 32
		hFunc.Write(proof.Path[i].Prefix[0], rootMSB, rootLSB, proof.Path[i].Suffix[0], proof.Path[i].Suffix[1])
		withSuffix := hFunc.Sum()

		shouldSet := api.IsZero(api.Sub(api.Cmp(i, proof.PathLen), -1))
		tmpRoot := api.Select(api.IsZero(api.Sub(proof.Path[i].PrefixLen, 3)), withPrefix, withSuffix)
		calcRoot = api.Select(shouldSet, tmpRoot, calcRoot)
		// api.Println(calcRoot)
	}

	api.Println(root, calcRoot)
	return api.IsZero(api.Sub(root, calcRoot))
}

func (proof *ExistenceProofOpt) Verify(api frontend.API, root frontend.Variable) frontend.Variable {
	hFunc, _ := mimc.NewMiMC(api)
	// hFunc.Write(proof.LeafPrefix, proof.HashedKey[0], proof.HashedKey[1], proof.HashedValue[0], proof.HashedValue[1])
	hFunc.Write(proof.LeafPrefix, proof.HashedKey, proof.HashedValue)
	calcRoot := hFunc.Sum()

	for i := 0; i < MaxPathDepth; i++ {
		// TODO(aeryz): In the original ics23 implementation, the proof is being checked against the cosmos spec and
		// the prefix and suffix sizes are controlled for all inner ops. Since we have fixed prefix and suffix, not sure
		// if that's needed.

		hFunc.Reset()
		// prefix = 48, suffix = 0
		// hFunc.Write(proof.Path[i].Prefix[0], proof.Path[i].Prefix[1], proof.Path[i].Prefix[2], rootMSB, rootLSB)
		hFunc.Write(proof.Path[i].Prefix, proof.Path[i].OtherHash, calcRoot)
		withPrefix := hFunc.Sum()
		hFunc.Reset()
		// prefix = 16, suffix = 32
		// hFunc.Write(proof.Path[i].Prefix[0], rootMSB, rootLSB, proof.Path[i].Suffix[0], proof.Path[i].Suffix[1])
		hFunc.Write(proof.Path[i].Prefix, calcRoot, proof.Path[i].OtherHash)
		withSuffix := hFunc.Sum()

		shouldSet := api.IsZero(api.Sub(api.Cmp(i, proof.PathLen), -1))
		tmpRoot := api.Select(api.IsZero(proof.Path[i].IsLeft), withSuffix, withPrefix)
		calcRoot = api.Select(shouldSet, tmpRoot, calcRoot)
		// api.Println(calcRoot)
	}

	api.Println(root, calcRoot)
	return api.IsZero(api.Sub(root, calcRoot))
}

func (circuit *NonExistCircuit) Define(api frontend.API) error {
	leftVerify := circuit.Left.Verify(api, circuit.Root)
	rightVerify := circuit.Right.Verify(api, circuit.Root)

	api.Println(leftVerify, rightVerify)

	leftExists := api.Sub(1, api.IsZero(circuit.Left.PathLen))
	rightExists := api.Sub(1, api.IsZero(circuit.Right.PathLen))

	// At least one of right or left should exist
	api.AssertIsEqual(1, api.Or(leftExists, rightExists))

	leftCheck := api.Select(leftExists, leftVerify, 1)
	rightCheck := api.Select(rightExists, rightVerify, 1)

	api.AssertIsEqual(1, api.And(leftCheck, rightCheck))

	return nil
}
