package customtypes

import (
	"fmt"

	fssz "github.com/prysmaticlabs/fastssz"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
)

var _ fssz.HashRoot = (BlockRoots)([][32]byte{})
var _ fssz.Marshaler = (*BlockRoots)(nil)
var _ fssz.Unmarshaler = (*BlockRoots)(nil)

// BlockRoots represents block roots of the beacon state.
type BlockRoots [][32]byte

// HashTreeRoot returns calculated hash root.
func (r BlockRoots) HashTreeRoot() ([32]byte, error) {
	return fssz.HashWithDefaultHasher(r)
}

// HashTreeRootWith hashes a BlockRoots object with a Hasher from the default HasherPool.
func (r BlockRoots) HashTreeRootWith(hh *fssz.Hasher) error {
	index := hh.Index()
	for _, sRoot := range r {
		hh.Append(sRoot[:])
	}
	hh.Merkleize(index)
	return nil
}

// UnmarshalSSZ deserializes the provided bytes buffer into the BlockRoots object.
func (r *BlockRoots) UnmarshalSSZ(buf []byte) error {
	if len(buf) != r.SizeSSZ() {
		return fmt.Errorf("expected buffer of length %d received %d", r.SizeSSZ(), len(buf))
	}

	roots := BlockRoots(make([][32]byte, fieldparams.BlockRootsLength))
	for i := range roots {
		copy(roots[i][:], buf[i*32:(i+1)*32])
	}
	*r = roots
	return nil
}

// MarshalSSZTo marshals BlockRoots with the provided byte slice.
func (r BlockRoots) MarshalSSZTo(dst []byte) ([]byte, error) {
	marshalled, err := r.MarshalSSZ()
	if err != nil {
		return nil, err
	}
	return append(dst, marshalled...), nil
}

// MarshalSSZ marshals BlockRoots into a serialized object.
func (r BlockRoots) MarshalSSZ() ([]byte, error) {
	marshalled := make([]byte, fieldparams.BlockRootsLength*32)
	for i, r32 := range r {
		for j, rr := range r32 {
			marshalled[i*32+j] = rr
		}
	}
	return marshalled, nil
}

// SizeSSZ returns the size of the serialized object.
func (_ BlockRoots) SizeSSZ() int {
	return fieldparams.BlockRootsLength * 32
}

// Slice converts a customtypes.BlockRoots object into a 2D byte slice.
// Each item in the slice is a copy of the original item.
func (r BlockRoots) Slice() [][]byte {
	if r == nil {
		return nil
	}
	bRoots := make([][]byte, len(r))
	for i, root := range r {
		tmp := root
		bRoots[i] = tmp[:]
	}
	return bRoots
}
