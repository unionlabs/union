package fieldtrie

import (
	"reflect"
	"sync"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stateutil"
	multi_value_slice "github.com/prysmaticlabs/prysm/v4/container/multi-value-slice"
	pmath "github.com/prysmaticlabs/prysm/v4/math"
)

var (
	ErrInvalidFieldTrie = errors.New("invalid field trie")
	ErrEmptyFieldTrie   = errors.New("empty field trie")
)

// sliceAccessor describes an interface for a multivalue slice
// object that returns information about the multivalue slice along with the
// particular state instance we are referencing.
type sliceAccessor interface {
	Len(obj multi_value_slice.Identifiable) int
	State() multi_value_slice.Identifiable
}

// FieldTrie is the representation of the representative
// trie of the particular field.
type FieldTrie struct {
	*sync.RWMutex
	reference     *stateutil.Reference
	fieldLayers   [][]*[32]byte
	field         types.FieldIndex
	dataType      types.DataType
	length        uint64
	numOfElems    int
	isTransferred bool
}

// NewFieldTrie is the constructor for the field trie data structure. It creates the corresponding
// trie according to the given parameters. Depending on whether the field is a basic/composite array
// which is either fixed/variable length, it will appropriately determine the trie.
func NewFieldTrie(field types.FieldIndex, fieldInfo types.DataType, elements interface{}, length uint64) (*FieldTrie, error) {
	if elements == nil {
		return &FieldTrie{
			field:      field,
			dataType:   fieldInfo,
			reference:  stateutil.NewRef(1),
			RWMutex:    new(sync.RWMutex),
			length:     length,
			numOfElems: 0,
		}, nil
	}

	fieldRoots, err := fieldConverters(field, []uint64{}, elements, true)
	if err != nil {
		return nil, err
	}

	if err := validateElements(field, fieldInfo, elements, length); err != nil {
		return nil, err
	}
	var numOfElems int
	if val, ok := elements.(sliceAccessor); ok {
		numOfElems = val.Len(val.State())
	} else {
		numOfElems = reflect.Indirect(reflect.ValueOf(elements)).Len()
	}
	switch fieldInfo {
	case types.BasicArray:
		fl, err := stateutil.ReturnTrieLayer(fieldRoots, length)
		if err != nil {
			return nil, err
		}
		return &FieldTrie{
			fieldLayers: fl,
			field:       field,
			dataType:    fieldInfo,
			reference:   stateutil.NewRef(1),
			RWMutex:     new(sync.RWMutex),
			length:      length,
			numOfElems:  numOfElems,
		}, nil
	case types.CompositeArray, types.CompressedArray:
		return &FieldTrie{
			fieldLayers: stateutil.ReturnTrieLayerVariable(fieldRoots, length),
			field:       field,
			dataType:    fieldInfo,
			reference:   stateutil.NewRef(1),
			RWMutex:     new(sync.RWMutex),
			length:      length,
			numOfElems:  numOfElems,
		}, nil
	default:
		return nil, errors.Errorf("unrecognized data type in field map: %v", reflect.TypeOf(fieldInfo).Name())
	}
}

// RecomputeTrie rebuilds the affected branches in the trie according to the provided
// changed indices and elements. This recomputes the trie according to the particular
// field the trie is based on.
func (f *FieldTrie) RecomputeTrie(indices []uint64, elements interface{}) ([32]byte, error) {
	f.Lock()
	defer f.Unlock()
	var fieldRoot [32]byte
	if len(indices) == 0 {
		return f.TrieRoot()
	}

	fieldRoots, err := fieldConverters(f.field, indices, elements, false)
	if err != nil {
		return [32]byte{}, err
	}

	if err := f.validateIndices(indices); err != nil {
		return [32]byte{}, err
	}
	if val, ok := elements.(sliceAccessor); ok {
		f.numOfElems = val.Len(val.State())
	} else {
		f.numOfElems = reflect.Indirect(reflect.ValueOf(elements)).Len()
	}
	switch f.dataType {
	case types.BasicArray:
		fieldRoot, f.fieldLayers, err = stateutil.RecomputeFromLayer(fieldRoots, indices, f.fieldLayers)
		if err != nil {
			return [32]byte{}, err
		}
		return fieldRoot, nil
	case types.CompositeArray:
		fieldRoot, f.fieldLayers, err = stateutil.RecomputeFromLayerVariable(fieldRoots, indices, f.fieldLayers)
		if err != nil {
			return [32]byte{}, err
		}
		return stateutil.AddInMixin(fieldRoot, uint64(len(f.fieldLayers[0])))
	case types.CompressedArray:
		numOfElems, err := f.field.ElemsInChunk()
		if err != nil {
			return [32]byte{}, err
		}
		iNumOfElems, err := pmath.Int(numOfElems)
		if err != nil {
			return [32]byte{}, err
		}
		// We remove the duplicates here in order to prevent
		// duplicated insertions into the trie.
		var newIndices []uint64
		indexExists := make(map[uint64]bool)
		newRoots := make([][32]byte, 0, len(fieldRoots)/iNumOfElems)
		for i, idx := range indices {
			startIdx := idx / numOfElems
			if indexExists[startIdx] {
				continue
			}
			newIndices = append(newIndices, startIdx)
			indexExists[startIdx] = true
			newRoots = append(newRoots, fieldRoots[i])
		}
		fieldRoot, f.fieldLayers, err = stateutil.RecomputeFromLayerVariable(newRoots, newIndices, f.fieldLayers)
		if err != nil {
			return [32]byte{}, err
		}
		return stateutil.AddInMixin(fieldRoot, uint64(f.numOfElems))
	default:
		return [32]byte{}, errors.Errorf("unrecognized data type in field map: %v", reflect.TypeOf(f.dataType).Name())
	}
}

// CopyTrie copies the references to the elements the trie
// is built on.
func (f *FieldTrie) CopyTrie() *FieldTrie {
	if f.fieldLayers == nil {
		return &FieldTrie{
			field:      f.field,
			dataType:   f.dataType,
			reference:  stateutil.NewRef(1),
			RWMutex:    new(sync.RWMutex),
			length:     f.length,
			numOfElems: f.numOfElems,
		}
	}
	dstFieldTrie := make([][]*[32]byte, len(f.fieldLayers))
	for i, layer := range f.fieldLayers {
		dstFieldTrie[i] = make([]*[32]byte, len(layer))
		copy(dstFieldTrie[i], layer)
	}
	return &FieldTrie{
		fieldLayers: dstFieldTrie,
		field:       f.field,
		dataType:    f.dataType,
		reference:   stateutil.NewRef(1),
		RWMutex:     new(sync.RWMutex),
		length:      f.length,
		numOfElems:  f.numOfElems,
	}
}

// Length return the length of the whole field trie.
func (f *FieldTrie) Length() uint64 {
	return f.length
}

// TransferTrie starts the process of transferring all the
// trie related data to a new trie. This is done if we
// know that other states which hold references to this
// trie will unlikely need it for recomputation. This helps
// us save on a copy. Any caller of this method will need
// to take care that this isn't called on an empty trie.
func (f *FieldTrie) TransferTrie() *FieldTrie {
	if f.fieldLayers == nil {
		return &FieldTrie{
			field:      f.field,
			dataType:   f.dataType,
			reference:  stateutil.NewRef(1),
			RWMutex:    new(sync.RWMutex),
			length:     f.length,
			numOfElems: f.numOfElems,
		}
	}
	f.isTransferred = true
	nTrie := &FieldTrie{
		fieldLayers: f.fieldLayers,
		field:       f.field,
		dataType:    f.dataType,
		reference:   stateutil.NewRef(1),
		RWMutex:     new(sync.RWMutex),
		length:      f.length,
		numOfElems:  f.numOfElems,
	}
	// Zero out field layers here.
	f.fieldLayers = nil
	return nTrie
}

// TrieRoot returns the corresponding root of the trie.
func (f *FieldTrie) TrieRoot() ([32]byte, error) {
	if f.Empty() {
		return [32]byte{}, ErrEmptyFieldTrie
	}
	if len(f.fieldLayers[len(f.fieldLayers)-1]) == 0 {
		return [32]byte{}, ErrInvalidFieldTrie
	}
	switch f.dataType {
	case types.BasicArray:
		return *f.fieldLayers[len(f.fieldLayers)-1][0], nil
	case types.CompositeArray:
		trieRoot := *f.fieldLayers[len(f.fieldLayers)-1][0]
		return stateutil.AddInMixin(trieRoot, uint64(len(f.fieldLayers[0])))
	case types.CompressedArray:
		trieRoot := *f.fieldLayers[len(f.fieldLayers)-1][0]
		return stateutil.AddInMixin(trieRoot, uint64(f.numOfElems))
	default:
		return [32]byte{}, errors.Errorf("unrecognized data type in field map: %v", reflect.TypeOf(f.dataType).Name())
	}
}

// FieldReference returns the underlying field reference
// object for the trie.
func (f *FieldTrie) FieldReference() *stateutil.Reference {
	return f.reference
}

// Empty checks whether the underlying field trie is
// empty or not.
func (f *FieldTrie) Empty() bool {
	return f == nil || len(f.fieldLayers) == 0 || f.isTransferred
}

// InsertFieldLayer manually inserts a field layer. This method
// bypasses the normal method of field computation, it is only
// meant to be used in tests.
func (f *FieldTrie) InsertFieldLayer(layer [][]*[32]byte) {
	f.fieldLayers = layer
}
