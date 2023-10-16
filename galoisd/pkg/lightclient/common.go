package lightclient

import (
	"fmt"
	"galois/pkg/merkle"
	curve "github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark/frontend"
	gadget "github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	"github.com/consensys/gnark/std/algebra/emulated/sw_emulated"
	"github.com/consensys/gnark/std/hash/mimc"
	"github.com/consensys/gnark/std/math/emulated"
)

// NOTE: this circuit is compatible with the bn254 backend ONLY as we assume
// that the scalar field is the one from this curve for many public inputs.

// Max number of validators this lc can handle
const MaxVal = 16

type Validator struct {
	HashableX    frontend.Variable
	HashableXMSB frontend.Variable
	HashableY    frontend.Variable
	HashableYMSB frontend.Variable
	Power        frontend.Variable
}

type TendermintLightClientInput struct {
	Sig           gadget.G2Affine
	Validators    [MaxVal]Validator
	NbOfVal       frontend.Variable
	NbOfSignature frontend.Variable
	Bitmap        frontend.Variable
}

type TendermintLightClientAPI struct {
	api   frontend.API
	input *TendermintLightClientInput
}

func NewTendermintLightClientAPI(api frontend.API, input *TendermintLightClientInput) *TendermintLightClientAPI {
	return &TendermintLightClientAPI{api: api, input: input}
}

// Given a variable of size N and limbs of size M, split the variable in N/M limbs.
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

// Reconstruct a value from it's limbs.
func Repack(api frontend.API, unpacked []frontend.Variable, sizeOfInput int, sizeOfElem int) []frontend.Variable {
	nbOfElems := sizeOfInput / sizeOfElem
	elems := make([]frontend.Variable, nbOfElems)
	for i := 0; i < nbOfElems; i++ {
		elems[i] = api.FromBinary(unpacked[i*sizeOfElem : (i+1)*sizeOfElem]...)
	}
	return elems
}

func (lc *TendermintLightClientAPI) Verify(message *gadget.G2Affine, expectedValRoot frontend.Variable, powerNumerator frontend.Variable, powerDenominator frontend.Variable) error {
	lc.api.AssertIsLessOrEqual(lc.input.NbOfVal, MaxVal)
	lc.api.AssertIsLessOrEqual(lc.input.NbOfSignature, lc.input.NbOfVal)
	bitmap := lc.api.ToBinary(lc.input.Bitmap, MaxVal)

	// Facility to iterate over the validators in the lc, this function will
	// do the necessary decoding/marshalling for the caller.
	//
	// This function will reconstruct each validator from the secret inputs by:
	// - re-composing the public key from its shifted/msb values
	forEachVal := func(f func(i int, signed frontend.Variable, publicKey *gadget.G1Affine, power frontend.Variable, leaf frontend.Variable) error) error {
		for i, signed := range bitmap {
			validator := lc.input.Validators[i]
			h, err := mimc.NewMiMC(lc.api)
			if err != nil {
				return err
			}
			h.Write(validator.HashableX, validator.HashableY, validator.HashableXMSB, validator.HashableYMSB, validator.Power)
			leaf := h.Sum()

			shiftedX := Unpack(lc.api, validator.HashableX, 256, 1)
			shiftedX[254] = validator.HashableXMSB
			unshiftedX := Repack(lc.api, shiftedX, 256, 64)

			shiftedY := Unpack(lc.api, validator.HashableY, 256, 1)
			shiftedY[254] = validator.HashableYMSB
			unshiftedY := Repack(lc.api, shiftedY, 256, 64)

			var rebuiltPublicKey gadget.G1Affine
			rebuiltPublicKey.X.Limbs = unshiftedX
			rebuiltPublicKey.Y.Limbs = unshiftedY

			if err = f(i, signed, &rebuiltPublicKey, validator.Power, leaf); err != nil {
				return err
			}
		}
		return nil
	}

	curveArithmetic, _ := sw_emulated.New[emulated.BN254Fp, emulated.BN254Fr](lc.api, sw_emulated.GetBN254Params())

	totalVotingPower := frontend.Variable(0)
	currentVotingPower := frontend.Variable(0)
	aggregatedKeys := frontend.Variable(0)
	var g1Zero curve.G1Affine
	g1Zero.X.SetZero()
	g1Zero.Y.SetZero()
	emulatedG1Zero := gadget.NewG1Affine(g1Zero)
	aggregatedPublicKey := emulatedG1Zero

	leafHashes := make([]frontend.Variable, MaxVal)

	merkle := merkle.NewMerkleTreeAPI(lc.api)

	if err := forEachVal(func(i int, signed frontend.Variable, publicKey *gadget.G1Affine, power frontend.Variable, leaf frontend.Variable) error {
		// Aggregate voting power and current power
		totalVotingPower = lc.api.Add(totalVotingPower, power)
		// Optionally aggregated public key/voting power if validator at index signed
		currentVotingPower = lc.api.Add(currentVotingPower, lc.api.Select(signed, power, 0))
		// Optionally aggregated public key if validator at index signed
		firstPK := lc.api.And(signed, lc.api.IsZero(aggregatedKeys))
		aggregated := curveArithmetic.AddUnified(&aggregatedPublicKey, curveArithmetic.Select(signed, publicKey, &gadget.G1Affine{}))
		aggregateNext := curveArithmetic.Select(firstPK, publicKey, aggregated)
		aggregatedPublicKey =
			*curveArithmetic.Select(signed, aggregateNext, &aggregatedPublicKey)
		aggregatedKeys = lc.api.Add(aggregatedKeys, lc.api.Select(signed, 1, 0))
		leafHashes[i] = merkle.LeafHash([]frontend.Variable{leaf})
		return nil
	}); err != nil {
		return err
	}

	// Compute validator set merkle root
	rootHash := merkle.RootHash(leafHashes, lc.input.NbOfVal)

	// Verify that the merkle root is equal to the given root (public input)
	lc.api.AssertIsEqual(expectedValRoot, rootHash)

	// Ensure that we actually aggregated the correct number of signatures
	lc.api.AssertIsEqual(aggregatedKeys, lc.input.NbOfSignature)

	// Ensure that the current sum of voting power exceed the expected threshold
	// x > ay/b <=> ay < bx
	votingPowerNeeded := lc.api.Mul(totalVotingPower, powerNumerator)
	currentVotingPowerScaled := lc.api.Mul(currentVotingPower, powerDenominator)
	lc.api.AssertIsLessOrEqual(votingPowerNeeded, currentVotingPowerScaled)

	pairing, err := gadget.NewPairing(lc.api)
	if err != nil {
		return fmt.Errorf("new pairing: %w", err)
	}

	_, _, g1AffGen, _ := curve.Generators()

	// Verify that the aggregated signature is correct
	var g1AffGenNeg curve.G1Affine
	g1AffGenNeg.Neg(&g1AffGen)
	negG1 := gadget.NewG1Affine(g1AffGenNeg)
	e, err := pairing.Pair(
		[]*gadget.G1Affine{&negG1, &aggregatedPublicKey},
		[]*gadget.G2Affine{&lc.input.Sig, message},
	)
	if err != nil {
		return fmt.Errorf("pair: %w", err)
	}

	var oneN curve.GT
	oneN.SetOne()
	one := gadget.NewGTEl(oneN)

	pairing.AssertIsEqual(e, &one)

	return nil
}
