package customtypes

import (
	"fmt"

	fssz "github.com/prysmaticlabs/fastssz"
)

var _ fssz.HashRoot = (HistoricalRoots)([][32]byte{})
var _ fssz.Marshaler = (*HistoricalRoots)(nil)
var _ fssz.Unmarshaler = (*HistoricalRoots)(nil)

// HistoricalRoots represents a 32 bytes HistoricalRoots object in Ethereum beacon chain consensus.
type HistoricalRoots [][32]byte

// HashTreeRoot returns calculated hash root.
func (r HistoricalRoots) HashTreeRoot() ([32]byte, error) {
	return fssz.HashWithDefaultHasher(r)
}

// HashTreeRootWith hashes a HistoricalRoots object with a Hasher from the default HasherPool.
func (r HistoricalRoots) HashTreeRootWith(hh *fssz.Hasher) error {
	index := hh.Index()
	for _, sRoot := range r {
		hh.Append(sRoot[:])
	}
	hh.Merkleize(index)
	return nil
}

// UnmarshalSSZ deserializes the provided bytes buffer into the HistoricalRoots object.
func (r *HistoricalRoots) UnmarshalSSZ(buf []byte) error {
	if len(buf) != r.SizeSSZ() {
		return fmt.Errorf("expected buffer of length %d received %d", r.SizeSSZ(), len(buf))
	}

	mixes := make([][32]byte, len(buf)/32)
	for i := range mixes {
		copy(mixes[i][:], buf[i*32:(i+1)*32])
	}
	*r = mixes
	return nil
}

// MarshalSSZTo marshals HistoricalRoots with the provided byte slice.
func (r HistoricalRoots) MarshalSSZTo(dst []byte) ([]byte, error) {
	marshalled, err := r.MarshalSSZ()
	if err != nil {
		return nil, err
	}
	return append(dst, marshalled...), nil
}

// MarshalSSZ marshals HistoricalRoots into a serialized object.
func (r HistoricalRoots) MarshalSSZ() ([]byte, error) {
	marshalled := make([]byte, len(r)*32)
	for i, r32 := range r {
		for j, rr := range r32 {
			marshalled[i*32+j] = rr
		}
	}
	return marshalled, nil
}

// SizeSSZ returns the size of the serialized object.
func (r HistoricalRoots) SizeSSZ() int {
	return len(r) * 32
}

// Slice converts a customtypes.HistoricalRoots object into a 2D byte slice.
func (r HistoricalRoots) Slice() [][]byte {
	if r == nil {
		return nil
	}
	hRoots := make([][]byte, len(r))
	for i, root := range r {
		tmp := root
		hRoots[i] = tmp[:]
	}
	return hRoots
}
