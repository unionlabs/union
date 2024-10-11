package customtypes

import (
	"fmt"

	fssz "github.com/prysmaticlabs/fastssz"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
)

var _ fssz.HashRoot = (RandaoMixes)([][32]byte{})
var _ fssz.Marshaler = (*RandaoMixes)(nil)
var _ fssz.Unmarshaler = (*RandaoMixes)(nil)

// RandaoMixes represents RANDAO mixes of the beacon state.
type RandaoMixes [][32]byte

// HashTreeRoot returns calculated hash root.
func (r RandaoMixes) HashTreeRoot() ([32]byte, error) {
	return fssz.HashWithDefaultHasher(r)
}

// HashTreeRootWith hashes a RandaoMixes object with a Hasher from the default HasherPool.
func (r RandaoMixes) HashTreeRootWith(hh *fssz.Hasher) error {
	index := hh.Index()
	for _, sRoot := range r {
		hh.Append(sRoot[:])
	}
	hh.Merkleize(index)
	return nil
}

// UnmarshalSSZ deserializes the provided bytes buffer into the RandaoMixes object.
func (r *RandaoMixes) UnmarshalSSZ(buf []byte) error {
	if len(buf) != r.SizeSSZ() {
		return fmt.Errorf("expected buffer of length %d received %d", r.SizeSSZ(), len(buf))
	}

	roots := RandaoMixes(make([][32]byte, fieldparams.RandaoMixesLength))
	for i := range roots {
		copy(roots[i][:], buf[i*32:(i+1)*32])
	}
	*r = roots
	return nil
}

// MarshalSSZTo marshals RandaoMixes with the provided byte slice.
func (r RandaoMixes) MarshalSSZTo(dst []byte) ([]byte, error) {
	marshalled, err := r.MarshalSSZ()
	if err != nil {
		return nil, err
	}
	return append(dst, marshalled...), nil
}

// MarshalSSZ marshals RandaoMixes into a serialized object.
func (r RandaoMixes) MarshalSSZ() ([]byte, error) {
	marshalled := make([]byte, fieldparams.RandaoMixesLength*32)
	for i, r32 := range r {
		for j, rr := range r32 {
			marshalled[i*32+j] = rr
		}
	}
	return marshalled, nil
}

// SizeSSZ returns the size of the serialized object.
func (_ RandaoMixes) SizeSSZ() int {
	return fieldparams.RandaoMixesLength * 32
}

// Slice converts a customtypes.RandaoMixes object into a 2D byte slice.
// Each item in the slice is a copy of the original item.
func (r RandaoMixes) Slice() [][]byte {
	if r == nil {
		return nil
	}
	mixes := make([][]byte, len(r))
	for i, root := range r {
		tmp := root
		mixes[i] = tmp[:]
	}
	return mixes
}
