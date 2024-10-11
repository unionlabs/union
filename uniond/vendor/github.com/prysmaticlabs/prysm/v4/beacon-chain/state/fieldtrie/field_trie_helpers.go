package fieldtrie

import (
	"encoding/binary"
	"fmt"
	"reflect"

	"github.com/pkg/errors"
	customtypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native/custom-types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stateutil"
	multi_value_slice "github.com/prysmaticlabs/prysm/v4/container/multi-value-slice"
	pmath "github.com/prysmaticlabs/prysm/v4/math"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// ProofFromMerkleLayers creates a proof starting at the leaf index of the state Merkle layers.
func ProofFromMerkleLayers(layers [][][]byte, startingLeafIndex int) [][]byte {
	// The merkle tree structure looks as follows:
	// [[r1, r2, r3, r4], [parent1, parent2], [root]]
	proof := make([][]byte, 0)
	currentIndex := startingLeafIndex
	for i := 0; i < len(layers)-1; i++ {
		neighborIdx := currentIndex ^ 1
		neighbor := layers[i][neighborIdx]
		proof = append(proof, neighbor)
		currentIndex = currentIndex / 2
	}
	return proof
}

func (f *FieldTrie) validateIndices(idxs []uint64) error {
	length := f.length
	if f.dataType == types.CompressedArray {
		comLength, err := f.field.ElemsInChunk()
		if err != nil {
			return err
		}
		length *= comLength
	}
	for _, idx := range idxs {
		if idx >= length {
			return errors.Errorf("invalid index for field %s: %d >= length %d", f.field.String(), idx, length)
		}
	}
	return nil
}

func validateElements(field types.FieldIndex, fieldInfo types.DataType, elements interface{}, length uint64) error {
	if fieldInfo == types.CompressedArray {
		comLength, err := field.ElemsInChunk()
		if err != nil {
			return err
		}
		length *= comLength
	}
	if val, ok := elements.(sliceAccessor); ok {
		totalLen := val.Len(val.State())
		if uint64(totalLen) > length {
			return errors.Errorf("elements length is larger than expected for field %s: %d > %d", field.String(), totalLen, length)
		}
		return nil
	}
	val := reflect.Indirect(reflect.ValueOf(elements))
	if uint64(val.Len()) > length {
		return errors.Errorf("elements length is larger than expected for field %s: %d > %d", field.String(), val.Len(), length)
	}
	return nil
}

// fieldConverters converts the corresponding field and the provided elements to the appropriate roots.
func fieldConverters(field types.FieldIndex, indices []uint64, elements interface{}, convertAll bool) ([][32]byte, error) {
	switch field {
	case types.BlockRoots, types.StateRoots, types.RandaoMixes:
		return convertRoots(indices, elements, convertAll)
	case types.Eth1DataVotes:
		return convertEth1DataVotes(indices, elements, convertAll)
	case types.Validators:
		return convertValidators(indices, elements, convertAll)
	case types.PreviousEpochAttestations, types.CurrentEpochAttestations:
		return convertAttestations(indices, elements, convertAll)
	case types.Balances:
		return convertBalances(indices, elements, convertAll)
	default:
		return [][32]byte{}, errors.Errorf("got unsupported type of %v", reflect.TypeOf(elements).Name())
	}
}

func convertRoots(indices []uint64, elements interface{}, convertAll bool) ([][32]byte, error) {
	switch castedType := elements.(type) {
	case customtypes.BlockRoots:
		return handle32ByteMVslice(multi_value_slice.BuildEmptyCompositeSlice[[32]byte](castedType), indices, convertAll)
	case customtypes.StateRoots:
		return handle32ByteMVslice(multi_value_slice.BuildEmptyCompositeSlice[[32]byte](castedType), indices, convertAll)
	case customtypes.RandaoMixes:
		return handle32ByteMVslice(multi_value_slice.BuildEmptyCompositeSlice[[32]byte](castedType), indices, convertAll)
	case multi_value_slice.MultiValueSliceComposite[[32]byte]:
		return handle32ByteMVslice(castedType, indices, convertAll)
	default:
		return nil, errors.Errorf("non-existent type provided %T", castedType)
	}
}

func convertEth1DataVotes(indices []uint64, elements interface{}, convertAll bool) ([][32]byte, error) {
	val, ok := elements.([]*ethpb.Eth1Data)
	if !ok {
		return nil, errors.Errorf("Wanted type of %T but got %T", []*ethpb.Eth1Data{}, elements)
	}
	return handleEth1DataSlice(val, indices, convertAll)
}

func convertValidators(indices []uint64, elements interface{}, convertAll bool) ([][32]byte, error) {
	switch casted := elements.(type) {
	case []*ethpb.Validator:
		return handleValidatorMVSlice(multi_value_slice.BuildEmptyCompositeSlice[*ethpb.Validator](casted), indices, convertAll)
	case multi_value_slice.MultiValueSliceComposite[*ethpb.Validator]:
		return handleValidatorMVSlice(casted, indices, convertAll)
	default:
		return nil, errors.Errorf("Wanted type of %T but got %T", []*ethpb.Validator{}, elements)
	}
}

func convertAttestations(indices []uint64, elements interface{}, convertAll bool) ([][32]byte, error) {
	val, ok := elements.([]*ethpb.PendingAttestation)
	if !ok {
		return nil, errors.Errorf("Wanted type of %T but got %T", []*ethpb.PendingAttestation{}, elements)
	}
	return handlePendingAttestationSlice(val, indices, convertAll)
}

func convertBalances(indices []uint64, elements interface{}, convertAll bool) ([][32]byte, error) {
	switch casted := elements.(type) {
	case []uint64:
		return handleBalanceMVSlice(multi_value_slice.BuildEmptyCompositeSlice[uint64](casted), indices, convertAll)
	case multi_value_slice.MultiValueSliceComposite[uint64]:
		return handleBalanceMVSlice(casted, indices, convertAll)
	default:
		return nil, errors.Errorf("Wanted type of %T but got %T", []uint64{}, elements)
	}
}

// handle32ByteMVslice computes and returns 32 byte arrays in a slice of root format. This is modified
// to be used with multivalue slices.
func handle32ByteMVslice(mv multi_value_slice.MultiValueSliceComposite[[32]byte],
	indices []uint64, convertAll bool) ([][32]byte, error) {
	length := len(indices)
	if convertAll {
		length = mv.Len(mv.State())
	}
	roots := make([][32]byte, 0, length)
	rootCreator := func(input [32]byte) {
		roots = append(roots, input)
	}
	if convertAll {
		val := mv.Value(mv.State())
		for i := range val {
			rootCreator(val[i])
		}
		return roots, nil
	}
	totalLen := mv.Len(mv.State())
	if totalLen > 0 {
		for _, idx := range indices {
			if idx > uint64(totalLen)-1 {
				return nil, fmt.Errorf("index %d greater than number of byte arrays %d", idx, totalLen)
			}
			val, err := mv.At(mv.State(), idx)
			if err != nil {
				return nil, err
			}
			rootCreator(val)
		}
	}
	return roots, nil
}

// handleValidatorMVSlice returns the validator indices in a slice of root format.
func handleValidatorMVSlice(mv multi_value_slice.MultiValueSliceComposite[*ethpb.Validator], indices []uint64, convertAll bool) ([][32]byte, error) {
	length := len(indices)
	if convertAll {
		return stateutil.OptimizedValidatorRoots(mv.Value(mv.State()))
	}
	roots := make([][32]byte, 0, length)
	rootCreator := func(input *ethpb.Validator) error {
		newRoot, err := stateutil.ValidatorRootWithHasher(input)
		if err != nil {
			return err
		}
		roots = append(roots, newRoot)
		return nil
	}
	totalLen := mv.Len(mv.State())
	if totalLen > 0 {
		for _, idx := range indices {
			if idx > uint64(totalLen)-1 {
				return nil, fmt.Errorf("index %d greater than number of validators %d", idx, totalLen)
			}
			val, err := mv.At(mv.State(), idx)
			if err != nil {
				return nil, err
			}
			err = rootCreator(val)
			if err != nil {
				return nil, err
			}
		}
	}
	return roots, nil
}

// handleEth1DataSlice processes a list of eth1data and indices into the appropriate roots.
func handleEth1DataSlice(val []*ethpb.Eth1Data, indices []uint64, convertAll bool) ([][32]byte, error) {
	length := len(indices)
	if convertAll {
		length = len(val)
	}
	roots := make([][32]byte, 0, length)
	rootCreator := func(input *ethpb.Eth1Data) error {
		newRoot, err := stateutil.Eth1DataRootWithHasher(input)
		if err != nil {
			return err
		}
		roots = append(roots, newRoot)
		return nil
	}
	if convertAll {
		for i := range val {
			err := rootCreator(val[i])
			if err != nil {
				return nil, err
			}
		}
		return roots, nil
	}
	if len(val) > 0 {
		for _, idx := range indices {
			if idx > uint64(len(val))-1 {
				return nil, fmt.Errorf("index %d greater than number of items in eth1 data slice %d", idx, len(val))
			}
			err := rootCreator(val[idx])
			if err != nil {
				return nil, err
			}
		}
	}
	return roots, nil
}

// handlePendingAttestationSlice returns the root of a slice of pending attestations.
func handlePendingAttestationSlice(val []*ethpb.PendingAttestation, indices []uint64, convertAll bool) ([][32]byte, error) {
	length := len(indices)
	if convertAll {
		length = len(val)
	}
	roots := make([][32]byte, 0, length)
	rootCreator := func(input *ethpb.PendingAttestation) error {
		newRoot, err := stateutil.PendingAttRootWithHasher(input)
		if err != nil {
			return err
		}
		roots = append(roots, newRoot)
		return nil
	}
	if convertAll {
		for i := range val {
			err := rootCreator(val[i])
			if err != nil {
				return nil, err
			}
		}
		return roots, nil
	}
	if len(val) > 0 {
		for _, idx := range indices {
			if idx > uint64(len(val))-1 {
				return nil, fmt.Errorf("index %d greater than number of pending attestations %d", idx, len(val))
			}
			err := rootCreator(val[idx])
			if err != nil {
				return nil, err
			}
		}
	}
	return roots, nil
}

func handleBalanceMVSlice(mv multi_value_slice.MultiValueSliceComposite[uint64], indices []uint64, convertAll bool) ([][32]byte, error) {
	if convertAll {
		val := mv.Value(mv.State())
		return stateutil.PackUint64IntoChunks(val)
	}
	totalLen := mv.Len(mv.State())
	if totalLen > 0 {
		numOfElems, err := types.Balances.ElemsInChunk()
		if err != nil {
			return nil, err
		}
		iNumOfElems, err := pmath.Int(numOfElems)
		if err != nil {
			return nil, err
		}
		var roots [][32]byte
		for _, idx := range indices {
			// We split the indexes into their relevant groups. Balances
			// are compressed according to 4 values -> 1 chunk.
			startIdx := idx / numOfElems
			startGroup := startIdx * numOfElems
			var chunk [32]byte
			sizeOfElem := len(chunk) / iNumOfElems
			for i, j := 0, startGroup; j < startGroup+numOfElems; i, j = i+sizeOfElem, j+1 {
				wantedVal := uint64(0)
				// We are adding chunks in sets of 4, if the set is at the edge of the array
				// then you will need to zero out the rest of the chunk. Ex : 41 indexes,
				// so 41 % 4 = 1 . There are 3 indexes, which do not exist yet but we
				// have to add in as a root. These 3 indexes are then given a 'zero' value.
				if j < uint64(totalLen) {
					val, err := mv.At(mv.State(), j)
					if err != nil {
						return nil, err
					}
					wantedVal = val
				}
				binary.LittleEndian.PutUint64(chunk[i:i+sizeOfElem], wantedVal)
			}
			roots = append(roots, chunk)
		}
		return roots, nil
	}
	return [][32]byte{}, nil
}
