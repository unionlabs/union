package lightclient

import (
	"fmt"
	"galois/pkg/merkle"
	"galois/pkg/proto"
	curve "github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark/frontend"
	gadget "github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	"github.com/consensys/gnark/std/algebra/emulated/sw_emulated"
	"github.com/consensys/gnark/std/math/emulated"
)

// NOTE: this circuit is compatible with the bn254 backend ONLY as we assume
// that the scalar field is the one from this curve for many public inputs.

/*
This protobuf format is highly unlikely to change as this would break the next
validator hash of current blocks for a given cosmos chain. Each variable is
guaranteed to fit in a bn254 field element as long as the public key is a G1
point:

  - 4 bytes PK meta
  - PK X/Y coordinates as bn254 field elements
  - 1 byte for power meta
  - max of 10 bytes (varint) for the power
*/
const ValProtoElems = 4
const ValProtoSize = 4 + 32 + 1 + 10
const ValProtoPKMeta = 0
const ValProtoPKX = 4
const ValProtoPowerMeta = 4 + 32
const ValProtoPower = 4 + 32 + 1

// Max number of validators this lc can handle
const MaxVal = 16

type TendermintLightClientInput struct {
	Sig             gadget.G2Affine
	ProtoValidators [MaxVal][4]frontend.Variable
	NbOfVal         frontend.Variable
	NbOfSignature   frontend.Variable
	Bitmap          frontend.Variable
}

type TendermintLightClientAPI struct {
	api   frontend.API
	input *TendermintLightClientInput
}

func NewTendermintLightClientAPI(api frontend.API, input *TendermintLightClientInput) *TendermintLightClientAPI {
	return &TendermintLightClientAPI{api: api, input: input}
}

// Given the X coordinate and it's compression bit, we can reconstruct the compressed point.
func ToG1AffineCompressed(api frontend.API, x frontend.Variable, compressionMask []frontend.Variable) []frontend.Variable {
	// TODO: Make this public in Gnark
	// mMask               byte = 0b11 << 6
	// mUncompressed       byte = 0b00 << 6
	// mCompressedSmallest byte = 0b10 << 6
	// mCompressedLargest  byte = 0b11 << 6
	// mCompressedInfinity byte = 0b01 << 6
	unpackedX := api.ToBinary(x, 256)
	unpackedX[254] = compressionMask[0]
	unpackedX[255] = compressionMask[1]
	bytes := Repack(api, unpackedX, 256, 8)
	return bytes
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

func (lc *TendermintLightClientAPI) Verify(message *gadget.G2Affine, expectedValRoot [2]frontend.Variable, powerNumerator frontend.Variable, powerDenominator frontend.Variable) error {
	lc.api.AssertIsLessOrEqual(lc.input.NbOfVal, MaxVal)
	lc.api.AssertIsLessOrEqual(lc.input.NbOfSignature, lc.input.NbOfVal)
	bitmap := lc.api.ToBinary(lc.input.Bitmap, MaxVal)

	// Facility to iterate over the validators in the lc, this function will
	// do the necessary decoding/marshalling for the caller.
	//
	// This function will reconstruct each validator protobuf payload from the secret inputs by:
	// - re-compressing the given public key with the given mask
	// - decoding the validator power that is a varint64
	forEachVal := func(f func(i int, signed frontend.Variable, power frontend.Variable, PK *gadget.G1Affine, rawProto [ValProtoSize]frontend.Variable, size frontend.Variable)) {
		for i, signed := range bitmap {
			validatorData := lc.input.ProtoValidators[i]
			validatorPKXBytes := validatorData[0]
			validatorPKYBytes := validatorData[1]
			validatorPowerProto := validatorData[2]
			compressionMaskBits := Unpack(lc.api, validatorData[3], 256, 1)
			validatorCompressedPKBytes := ToG1AffineCompressed(lc.api, validatorPKXBytes, compressionMaskBits)

			// Fixed width uint64 varint for decoding
			// May be 10 bytes long as per protobuf spec
			validatorPowerBytes := Unpack(lc.api, validatorPowerProto, proto.MaxVarintSize*8, 1)

			power, powerSize := proto.NewProtoAPI(lc.api).DecodeVarint64(validatorPowerBytes)

			protoEncodedSize := lc.api.Add(ValProtoPower, powerSize)

			powerBytes := Unpack(lc.api, validatorPowerProto, proto.MaxVarintSize*8, 8)

			rawProto := [ValProtoSize]frontend.Variable{}
			// This values are hardcoded protobuf tags that will
			// never change for the network as changing this would
			// break consensus.
			rawProto[ValProtoPKMeta+0] = 10
			rawProto[ValProtoPKMeta+1] = 34
			rawProto[ValProtoPKMeta+2] = 26
			rawProto[ValProtoPKMeta+3] = 32
			for j := 0; j < 32; j++ {
				rawProto[j+ValProtoPKX] = validatorCompressedPKBytes[31-j]
			}
			rawProto[ValProtoPowerMeta] = 16
			for j := 0; j < proto.MaxVarintSize; j++ {
				rawProto[j+ValProtoPower] = powerBytes[j]
			}

			// Gnark G1Affine coordinates are expressed in 64 bit
			// limbs, we must unpack the inputs. Note that we know
			// that PKX/PKY can fit in a single scalar field as we
			// ALWAYS use the bn254 backend (bn254 public keys).
			var PK gadget.G1Affine
			PK.X.Limbs = Unpack(lc.api, validatorPKXBytes, 256, 64)
			PK.Y.Limbs = Unpack(lc.api, validatorPKYBytes, 256, 64)

			f(i, signed, power, &PK, rawProto, protoEncodedSize)
		}
	}

	curveArithmetic, _ := sw_emulated.New[emulated.BN254Fp, emulated.BN254Fr](lc.api, sw_emulated.GetBN254Params())

	_, _, g1AffGen, _ := curve.Generators()

	emulatedG1 := gadget.NewG1Affine(g1AffGen)

	totalVotingPower := frontend.Variable(0)
	currentVotingPower := frontend.Variable(0)
	aggregatedKeys := frontend.Variable(0)
	var g1Zero curve.G1Affine
	g1Zero.X.SetZero()
	g1Zero.Y.SetZero()
	emulatedG1Zero := gadget.NewG1Affine(g1Zero)
	aggregatedPublicKey := emulatedG1Zero

	leafHashes := make([][]frontend.Variable, MaxVal)

	merkle := merkle.NewMerkleTreeAPI(lc.api)

	forEachVal(func(i int, signed frontend.Variable, power frontend.Variable, PK *gadget.G1Affine, rawProto [ValProtoSize]frontend.Variable, protoSize frontend.Variable) {
		// Aggregate voting power and current power
		totalVotingPower = lc.api.Add(totalVotingPower, power)
		// Optionally aggregated public key/voting power if validator at index signed
		currentVotingPower = lc.api.Add(currentVotingPower, lc.api.Select(signed, power, 0))
		// Avoid issue with null point, emulatedG1 is never used because only reference in the !signed branch
		toAggregate := curveArithmetic.Select(signed, PK, &emulatedG1)
		// Optionally aggregated public key if validator at index signed
		firstPK := lc.api.And(signed, lc.api.IsZero(aggregatedKeys))
		aggregated := curveArithmetic.Add(&aggregatedPublicKey, toAggregate)
		aggregateNext := curveArithmetic.Select(firstPK, PK, aggregated)
		aggregatedPublicKey =
			*curveArithmetic.Select(signed, aggregateNext, &aggregatedPublicKey)
		aggregatedKeys = lc.api.Add(aggregatedKeys, lc.api.Select(signed, 1, 0))
		leafHashes[i] = merkle.LeafHash(rawProto[:], protoSize)
	})

	// Compute validator set merkle root
	rootHash := merkle.RootHash(leafHashes, lc.input.NbOfVal)

	// Decompose the given sha256 that can't fit in a single scalar field (hence, 2 public inputs)
	// TODO: can be optimizing by dropping the 2 msb of the hash to fit in the field
	expectedRootHash0 := Unpack(lc.api, expectedValRoot[0], 128, 8)
	expectedRootHash1 := Unpack(lc.api, expectedValRoot[1], 128, 8)

	// Verify that the merkle root is equal to the given root (public input)
	for i := 0; i < 16; i++ {
		lc.api.AssertIsEqual(expectedRootHash0[i], rootHash[15-i])
	}
	for i := 0; i < 16; i++ {
		lc.api.AssertIsEqual(expectedRootHash1[i], rootHash[31-i])
	}

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
