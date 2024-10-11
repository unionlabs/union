package stateutil

import (
	"encoding/binary"

	"github.com/pkg/errors"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/encoding/ssz"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// ValidatorRootWithHasher describes a method from which the hash tree root
// of a validator is returned.
func ValidatorRootWithHasher(validator *ethpb.Validator) ([32]byte, error) {
	fieldRoots, err := ValidatorFieldRoots(validator)
	if err != nil {
		return [32]byte{}, err
	}
	return ssz.BitwiseMerkleize(fieldRoots, uint64(len(fieldRoots)), uint64(len(fieldRoots)))
}

// ValidatorFieldRoots describes a method from which the hash tree root
// of a validator is returned.
func ValidatorFieldRoots(validator *ethpb.Validator) ([][32]byte, error) {
	var fieldRoots [][32]byte
	if validator != nil {
		pubkey := bytesutil.ToBytes48(validator.PublicKey)
		withdrawCreds := bytesutil.ToBytes32(validator.WithdrawalCredentials)
		var effectiveBalanceBuf [32]byte
		binary.LittleEndian.PutUint64(effectiveBalanceBuf[:8], validator.EffectiveBalance)
		// Slashed.
		var slashBuf [32]byte
		if validator.Slashed {
			slashBuf[0] = uint8(1)
		} else {
			slashBuf[0] = uint8(0)
		}
		var activationEligibilityBuf [32]byte
		binary.LittleEndian.PutUint64(activationEligibilityBuf[:8], uint64(validator.ActivationEligibilityEpoch))

		var activationBuf [32]byte
		binary.LittleEndian.PutUint64(activationBuf[:8], uint64(validator.ActivationEpoch))

		var exitBuf [32]byte
		binary.LittleEndian.PutUint64(exitBuf[:8], uint64(validator.ExitEpoch))

		var withdrawalBuf [32]byte
		binary.LittleEndian.PutUint64(withdrawalBuf[:8], uint64(validator.WithdrawableEpoch))

		// Public key.
		pubKeyRoot, err := merkleizePubkey(pubkey[:])
		if err != nil {
			return [][32]byte{}, err
		}
		fieldRoots = [][32]byte{pubKeyRoot, withdrawCreds, effectiveBalanceBuf, slashBuf, activationEligibilityBuf,
			activationBuf, exitBuf, withdrawalBuf}
	}
	return fieldRoots, nil
}

// Uint64ListRootWithRegistryLimit computes the HashTreeRoot Merkleization of
// a list of uint64 and mixed with registry limit.
func Uint64ListRootWithRegistryLimit(balances []uint64) ([32]byte, error) {
	balancesChunks, err := PackUint64IntoChunks(balances)
	if err != nil {
		return [32]byte{}, errors.Wrap(err, "could not pack balances into chunks")
	}
	balancesRootsRoot, err := ssz.BitwiseMerkleize(balancesChunks, uint64(len(balancesChunks)), ValidatorLimitForBalancesChunks())
	if err != nil {
		return [32]byte{}, errors.Wrap(err, "could not compute balances merkleization")
	}

	balancesLengthRoot := make([]byte, 32)
	binary.LittleEndian.PutUint64(balancesLengthRoot, uint64(len(balances)))
	return ssz.MixInLength(balancesRootsRoot, balancesLengthRoot), nil
}

// ValidatorLimitForBalancesChunks returns the limit of validators after going through the chunking process.
func ValidatorLimitForBalancesChunks() uint64 {
	maxValidatorLimit := uint64(fieldparams.ValidatorRegistryLimit)
	bytesInUint64 := uint64(8)
	return (maxValidatorLimit*bytesInUint64 + 31) / 32 // round to nearest chunk
}

// PackUint64IntoChunks packs a list of uint64 values into 32 byte roots.
func PackUint64IntoChunks(vals []uint64) ([][32]byte, error) {
	// Initialize how many uint64 values we can pack
	// into a single chunk(32 bytes). Each uint64 value
	// would take up 8 bytes.
	numOfElems := 4
	sizeOfElem := 32 / numOfElems
	// Determine total number of chunks to be
	// allocated to provided list of unsigned
	// 64-bit integers.
	numOfChunks := len(vals) / numOfElems
	// Add an extra chunk if the list size
	// is not a perfect multiple of the number
	// of elements.
	if len(vals)%numOfElems != 0 {
		numOfChunks++
	}
	chunkList := make([][32]byte, numOfChunks)
	for idx, b := range vals {
		// In order to determine how to pack in the uint64 value by index into
		// our chunk list we need to determine a few things.
		// 1) The chunk which the particular uint64 value corresponds to.
		// 2) The position of the value in the chunk itself.
		//
		// Once we have determined these 2 values we can simply find the correct
		// section of contiguous bytes to insert the value in the chunk.
		chunkIdx := idx / numOfElems
		idxInChunk := idx % numOfElems
		chunkPos := idxInChunk * sizeOfElem
		binary.LittleEndian.PutUint64(chunkList[chunkIdx][chunkPos:chunkPos+sizeOfElem], b)
	}
	return chunkList, nil
}
