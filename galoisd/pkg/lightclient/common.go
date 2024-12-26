package lightclient

import (
	"fmt"
	"galois/pkg/bls"
	"galois/pkg/merkle"

	mimc "galois/pkg/emulatedmimc"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	gadget "github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	"github.com/consensys/gnark/std/algebra/emulated/sw_emulated"
	"github.com/consensys/gnark/std/math/emulated"
)

// Max number of validators the light client can handle
const MaxVal = 128

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

// Given a variable of size N and limbs of size M, split the variable in N/M limbs.
func UnpackEmulated[T emulated.FieldParams](field *emulated.Field[T], api frontend.API, packed *emulated.Element[T], sizeOfInput int, sizeOfElem int) []*emulated.Element[T] {
	nbOfElems := sizeOfInput / sizeOfElem
	if sizeOfElem == 1 {
		binary := field.ToBits(field.Reduce(packed))
		elems := make([]*emulated.Element[T], nbOfElems)
		for i := 0; i < nbOfElems; i++ {
			elems[i] = field.Select(api.IsZero(binary[i]), field.Zero(), field.One())
		}
		return elems
	} else {
		unpacked := field.ToBits(field.Reduce(packed))[:sizeOfInput]
		elems := make([]*emulated.Element[T], nbOfElems)
		for i := 0; i < nbOfElems; i++ {
			elems[i] = field.FromBits(unpacked[i*sizeOfElem : (i+1)*sizeOfElem]...)
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

// Union whitepaper: Algorithm 2. procedure V
func (lc *TendermintLightClientAPI) Verify(message *gadget.G2Affine, expectedValRoot frontend.Variable, powerNumerator frontend.Variable, powerDenominator frontend.Variable) error {
	lc.api.AssertIsLessOrEqual(lc.input.NbOfVal, MaxVal)
	lc.api.AssertIsLessOrEqual(lc.input.NbOfSignature, lc.input.NbOfVal)
	// Ensure that at least one validator/signature are provided
	lc.api.AssertIsLessOrEqual(1, lc.input.NbOfSignature)

	// Note that because the scalar field modulus is 253 bits wide, the maximum bitmap size is 252
	// We would need to split the bitmap into multiple public inputs if we wanted to push this limit
	bitmap := lc.api.ToBinary(lc.input.Bitmap, MaxVal)

	field, err := emulated.NewField[sw_bn254.ScalarField](lc.api)
	if err != nil {
		return err
	}

	// Facility to iterate over the validators in the lc, this function will
	// do the necessary decoding/marshalling for the caller.
	//
	// This function will reconstruct each validator from the secret inputs by:
	// - re-composing the public key from its shifted/msb values
	forEachVal := func(f func(i int, signed frontend.Variable, cannotSign frontend.Variable, publicKey *gadget.G1Affine, power frontend.Variable, leaf *emulated.Element[sw_bn254.ScalarField]) error) error {
		bitmapMask := lc.input.NbOfVal
		for i, signed := range bitmap {
			validator := lc.input.Validators[i]

			h, err := mimc.NewMiMC[sw_bn254.ScalarField](field)
			if err != nil {
				return fmt.Errorf("new mimc: %w", err)
			}
			// Union whitepaper: (11) H_pre
			//
			h.Write(
				field.FromBits(lc.api.ToBinary(validator.HashableX, 256)...),
				field.FromBits(lc.api.ToBinary(validator.HashableY, 256)...),
				field.FromBits(lc.api.ToBinary(validator.HashableXMSB, 256)...),
				field.FromBits(lc.api.ToBinary(validator.HashableYMSB, 256)...),
				field.FromBits(lc.api.ToBinary(validator.Power, 256)...))
			leaf := h.Sum()

			// Reconstruct the public key from the merkle leaf
			/*
			   pk = (val.pk.X | (val.pk.XMSB << 253), val.pk.Y | (val.pk.YMSB << 253))
			*/
			shiftedX := Unpack(lc.api, validator.HashableX, 256, 1)
			shiftedX[253] = validator.HashableXMSB
			unshiftedX := Repack(lc.api, shiftedX, 256, 64)

			shiftedY := Unpack(lc.api, validator.HashableY, 256, 1)
			shiftedY[253] = validator.HashableYMSB
			unshiftedY := Repack(lc.api, shiftedY, 256, 64)

			var rebuiltPublicKey gadget.G1Affine
			rebuiltPublicKey.X.Limbs = unshiftedX
			rebuiltPublicKey.Y.Limbs = unshiftedY

			cannotSign := lc.api.IsZero(bitmapMask)

			if err = f(i, signed, cannotSign, &rebuiltPublicKey, validator.Power, leaf); err != nil {
				return err
			}

			bitmapMask = lc.api.Select(cannotSign, cannotSign, lc.api.Sub(bitmapMask, 1))
		}
		return nil
	}

	totalVotingPower := frontend.Variable(0)
	currentVotingPower := frontend.Variable(0)

	leafHashes := make([]*emulated.Element[sw_bn254.ScalarField], MaxVal)

	merkle := merkle.NewMerkleTreeAPI(lc.api)

	bls, err := bls.NewBlsAPI(lc.api)
	if err != nil {
		return fmt.Errorf("new bls: %w", err)
	}

	aggregatedPublicKey, nbOfKeys, err := bls.WithAggregation(
		func(aggregate func(selector frontend.Variable, publicKey *sw_emulated.AffinePoint[emulated.BN254Fp])) error {
			if err := forEachVal(func(i int, signed frontend.Variable, cannotSign frontend.Variable, publicKey *gadget.G1Affine, power frontend.Variable, leaf *emulated.Element[sw_bn254.ScalarField]) error {
				actuallySigned := lc.api.Select(cannotSign, 0, signed)
				// totalVotingPower = totalVotingPower + power
				totalVotingPower = lc.api.Add(totalVotingPower, lc.api.Select(cannotSign, 0, power))
				// currentVotingPower = currentVotingPower + if signed then power else 0
				currentVotingPower = lc.api.Add(currentVotingPower, lc.api.Select(actuallySigned, power, 0))
				// Optionally aggregated public key if validator at index signed
				aggregate(actuallySigned, publicKey)
				leafHashes[i] = field.Select(cannotSign, field.Zero(), merkle.LeafHash(field, []*emulated.Element[sw_bn254.ScalarField]{leaf}))
				return nil
			}); err != nil {
				return err
			}
			return nil
		})
	if err != nil {
		return err
	}

	// Ensure that we actually aggregated the correct number of signatures
	lc.api.AssertIsEqual(nbOfKeys, lc.input.NbOfSignature)

	// Ensure that the current sum of voting power exceed the expected threshold
	votingPowerNeeded := lc.api.Mul(totalVotingPower, powerNumerator)
	currentVotingPowerScaled := lc.api.Mul(currentVotingPower, powerDenominator)
	lc.api.AssertIsLessOrEqual(votingPowerNeeded, currentVotingPowerScaled)

	// Verify that the merkle root is equal to the given root (public input)
	rootHash := merkle.RootHash(field, leafHashes, lc.input.NbOfVal)
	field.AssertIsEqual(field.FromBits(lc.api.ToBinary(expectedValRoot, 256)...), rootHash)

	return bls.VerifySignature(aggregatedPublicKey, message, &lc.input.Sig)
}
