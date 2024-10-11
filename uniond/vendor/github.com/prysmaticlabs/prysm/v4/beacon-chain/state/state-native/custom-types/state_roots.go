package customtypes

import (
	"fmt"

	fssz "github.com/prysmaticlabs/fastssz"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
)

var _ fssz.HashRoot = (StateRoots)([][32]byte{})
var _ fssz.Marshaler = (*StateRoots)(nil)
var _ fssz.Unmarshaler = (*StateRoots)(nil)

// StateRoots represents block roots of the beacon state.
type StateRoots [][32]byte

// HashTreeRoot returns calculated hash root.
func (r StateRoots) HashTreeRoot() ([32]byte, error) {
	return fssz.HashWithDefaultHasher(r)
}

// HashTreeRootWith hashes a StateRoots object with a Hasher from the default HasherPool.
func (r StateRoots) HashTreeRootWith(hh *fssz.Hasher) error {
	index := hh.Index()
	for _, sRoot := range r {
		hh.Append(sRoot[:])
	}
	hh.Merkleize(index)
	return nil
}

// UnmarshalSSZ deserializes the provided bytes buffer into the StateRoots object.
func (r *StateRoots) UnmarshalSSZ(buf []byte) error {
	if len(buf) != r.SizeSSZ() {
		return fmt.Errorf("expected buffer of length %d received %d", r.SizeSSZ(), len(buf))
	}

	roots := StateRoots(make([][32]byte, fieldparams.StateRootsLength))
	for i := range roots {
		copy(roots[i][:], buf[i*32:(i+1)*32])
	}
	*r = roots
	return nil
}

// MarshalSSZTo marshals StateRoots with the provided byte slice.
func (r StateRoots) MarshalSSZTo(dst []byte) ([]byte, error) {
	marshalled, err := r.MarshalSSZ()
	if err != nil {
		return nil, err
	}
	return append(dst, marshalled...), nil
}

// MarshalSSZ marshals StateRoots into a serialized object.
func (r StateRoots) MarshalSSZ() ([]byte, error) {
	marshalled := make([]byte, fieldparams.StateRootsLength*32)
	for i, r32 := range r {
		for j, rr := range r32 {
			marshalled[i*32+j] = rr
		}
	}
	return marshalled, nil
}

// SizeSSZ returns the size of the serialized object.
func (_ StateRoots) SizeSSZ() int {
	return fieldparams.StateRootsLength * 32
}

// Slice converts a customtypes.StateRoots object into a 2D byte slice.
// Each item in the slice is a copy of the original item.
func (r StateRoots) Slice() [][]byte {
	if r == nil {
		return nil
	}
	sRoots := make([][]byte, len(r))
	for i, root := range r {
		tmp := root
		sRoots[i] = tmp[:]
	}
	return sRoots
}
